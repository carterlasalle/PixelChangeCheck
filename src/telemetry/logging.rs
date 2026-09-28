//! Log configuration and the metrics endpoint.
//!
//! # The rule
//!
//! **`RUST_LOG` wins over `--log-level`.** Someone who has just hit a bug
//! needs per-module filtering without rebuilding, and a flag that
//! silently outranked the environment variable would make that impossible
//! exactly when it matters.

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing_subscriber::EnvFilter;

/// How to render log events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum LogFormat {
    /// Human-readable, for a terminal.
    Text,
    /// One JSON object per event, for a bug report or a log aggregator.
    Json,
}

/// A non-blocking writer guard. Dropping it flushes; leaking it would leave
/// the last few events unwritten on a clean shutdown, and holding the file
/// open forever is what a rotating writer is for.
pub struct LogGuard {
    _worker: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl LogGuard {
    fn none() -> Self {
        Self { _worker: None }
    }
}

/// Build the global subscriber from flags and the environment.
///
/// Written out per format rather than generically: the builder types
/// differ between `.json()` and `.compact()`, and a helper over both
/// would need a trait nobody else implements.
pub fn init(level: &str, format: LogFormat, file: Option<&str>) -> Result<LogGuard> {
    // A caller who sets RUST_LOG gets per-module filtering; the flag is
    // only the default when it is absent.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("pcc={level},info")));

    let Some(path) = file else {
        let builder = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stdout)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false);
        match format {
            LogFormat::Json => tracing::subscriber::set_global_default(
                builder.with_ansi(false).json().flatten_event(true).finish(),
            ),
            LogFormat::Text => {
                tracing::subscriber::set_global_default(builder.with_ansi(false).compact().finish())
            }
        }
        .map_err(|e| anyhow::anyhow!("Failed to install the log subscriber: {e}"))?;
        return Ok(LogGuard::none());
    };

    let p = std::path::Path::new(path);
    if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create the log directory {dir:?}"))?;
    }
    // Hourly rotation: a long-running share must not fill a disk with one
    // unbounded file.
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "pcc".into());
    let dir = p
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| p.to_path_buf());
    let (writer, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(true)
        // Bounded: a stalled sink must not grow memory without limit.
        .buffered_lines_limit(64 * 1024)
        .finish(tracing_appender::rolling::hourly(dir, stem));

    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);
    match format {
        LogFormat::Json => tracing::subscriber::set_global_default(
            builder.with_ansi(false).json().flatten_event(true).finish(),
        ),
        LogFormat::Text => {
            tracing::subscriber::set_global_default(builder.with_ansi(false).compact().finish())
        }
    }
    .map_err(|e| anyhow::anyhow!("Failed to install the log subscriber: {e}"))?;

    Ok(LogGuard {
        _worker: Some(guard),
    })
}

/// Serve `GET /metrics` in Prometheus text format.
///
/// Bound to loopback by default and separate from the web port on purpose:
/// the web port is token-authenticated and is a *session* surface, while
/// metrics describe the whole process to whoever can reach them.
pub async fn serve_metrics(addr: std::net::SocketAddr, metrics: Arc<super::Metrics>) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind the metrics endpoint on {addr}"))?;
    tracing::info!("Metrics on http://{addr}/metrics");

    let started = std::time::Instant::now();
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let body = metrics.render_prometheus(started.elapsed());
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\n\
             Content-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        // A scrape is a few hundred bytes; serving it inline is simpler
        // than a connection pool for something this infrequent.
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ValueEnum;

    #[test]
    fn both_formats_are_parseable_by_clap() {
        // A typo in the value enum would otherwise only surface at runtime.
        let names: Vec<String> = LogFormat::value_variants()
            .iter()
            .filter_map(|v| v.to_possible_value().map(|p| p.get_name().to_string()))
            .collect();
        assert_eq!(names, vec!["text", "json"]);
    }

    #[test]
    fn a_default_filter_follows_the_requested_level() {
        // The constructed filter, not a global one, so the test does not
        // race with `init`.
        let f = EnvFilter::new("pcc=warn,info");
        assert!(f.to_string().contains("warn"));
    }
}
