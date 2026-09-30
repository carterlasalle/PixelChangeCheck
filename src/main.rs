use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use pixel_change_check_client::app::{share, view};
use pixel_change_check_client::network::{SessionToken, DEFAULT_PORT};
use pixel_change_check_client::reach;
use pixel_change_check_client::relay;
use pixel_change_check_client::telemetry::{self, LogFormat, Metrics};
use std::time::Duration;

const DEFAULT_RELAY_PORT: u16 = 5900;
const DEFAULT_WEB_PORT: u16 = 8080;

#[derive(Parser)]
#[command(
    name = "pcc",
    version,
    about = "PixelChangeCheck: exact, lossless desktop replication"
)]
struct Cli {
    /// Minimum log level: error, warn, info, debug, trace.
    /// RUST_LOG overrides this and wins for per-module filtering.
    #[arg(long, global = true, default_value = "info")]
    log_level: String,
    /// How to render log events.
    #[arg(long, global = true, value_enum, default_value_t = LogFormat::Text)]
    log_format: LogFormat,
    /// Write logs here as well, rotating hourly.
    #[arg(long, global = true)]
    log_file: Option<String>,
    /// Print a metrics summary to stderr every N seconds. 0 disables.
    #[arg(long, global = true, default_value_t = 0)]
    stats_interval: u64,
    /// Serve Prometheus metrics on this address.
    #[arg(long, global = true)]
    metrics_listen: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

/// How hard to try for a direct connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum ReachPolicyArg {
    /// Try IPv6, then STUN, then the relay.
    Auto,
    /// Fail rather than use a relay.
    Direct,
    /// Skip discovery.
    Relay,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture your screen and share it: to direct viewers, through a
    /// relay, and/or as a browser stream any device can open.
    Share {
        /// Address to listen on for direct viewer connections, as
        /// host:port. A hostname is resolved.
        #[arg(long, default_value_t = format!("0.0.0.0:{DEFAULT_PORT}"))]
        listen: String,
        /// Disable the direct QUIC listener.
        #[arg(long)]
        no_listen: bool,
        /// Relay server address (host:port), for viewers who cannot
        /// connect to you directly (both behind NAT, no port forwarding).
        #[arg(long)]
        relay: Option<String>,
        /// SHA-256 fingerprint of the relay's certificate, as the relay
        /// printed it when it started. Required with --relay.
        #[arg(long)]
        relay_pin: Option<String>,
        /// Rendezvous code for the relay session. Auto-generated and
        /// printed if omitted. This is not the secret: --token is.
        #[arg(long)]
        session: Option<String>,
        /// Address for the browser viewer, as host:port.
        #[arg(long, default_value_t = format!("0.0.0.0:{DEFAULT_WEB_PORT}"))]
        web: String,
        /// Disable the browser viewer entirely.
        #[arg(long)]
        no_web: bool,
        /// PEM certificate for HTTPS/WSS on the browser port. Required for
        /// a non-loopback --web address; loopback stays plaintext for dev.
        #[arg(long, requires = "web_key")]
        web_cert: Option<String>,
        /// PEM private key matching --web-cert.
        #[arg(long)]
        web_key: Option<String>,
        /// Use a synthetic animated test pattern instead of capturing the
        /// real screen. Useful on headless machines and for local tests.
        #[arg(long)]
        synthetic: bool,
        /// Display index to share (see `pcc diagnose --displays` order).
        /// Default shares the primary display.
        #[arg(long)]
        display: Option<usize>,
        /// Screen region to share as x,y,w,h in display pixels, e.g.
        /// `--region 100,200,1280,720`. Captured at the layer, not cropped
        /// afterwards. Conflicts with --display.
        #[arg(long, conflicts_with = "display")]
        region: Option<String>,
        /// Window to share (title or handle). Platform pickers land here;
        /// today it falls back to the display with a warning.
        #[arg(long)]
        window: Option<String>,
        /// Application to share (name or bundle id). Same fallback as
        /// --window for now.
        #[arg(long)]
        application: Option<String>,
        /// Target frames per second.
        #[arg(long, default_value_t = 30)]
        fps: u32,
        /// Hard ceiling the scheduler may never exceed.
        #[arg(long, default_value_t = 60)]
        max_fps: u32,
        /// Initial JPEG quality for the *lossy browser preview*, 0.0-1.0.
        /// The authoritative stream is lossless and ignores this.
        #[arg(long, default_value_t = 0.8)]
        quality: f32,
        /// Viewers must present this token. Generated and printed if
        /// omitted. Anyone without it cannot see the screen.
        #[arg(long)]
        token: Option<String>,
        /// Seconds between backstop convergence snapshots. Zero uses the
        /// 30s default.
        #[arg(long, default_value_t = 0)]
        repair_secs: u64,
        /// How hard to try for a direct path before falling back to a
        /// relay: auto tries IPv6 then STUN, direct refuses the relay,
        /// relay skips discovery entirely.
        #[arg(long, value_enum, default_value_t = ReachPolicyArg::Auto)]
        reach: ReachPolicyArg,
        /// Also capture audio and send it alongside the screen.
        #[arg(long)]
        audio: bool,
    },
    /// Connect to a shared session and view it.
    View {
        /// Connect directly to a sharer's listen address, as host:port.
        #[arg(long, conflicts_with = "relay")]
        connect: Option<String>,
        /// Connect through a relay instead of directly.
        #[arg(long)]
        relay: Option<String>,
        /// Relay session code (required with --relay).
        #[arg(long)]
        session: Option<String>,
        /// SHA-256 fingerprint of the sharer's certificate. Required:
        /// this is what makes the connection authenticated.
        #[arg(long)]
        pin: String,
        /// The token the sharer printed. Without it the sharer refuses.
        #[arg(long)]
        token: String,
        /// Don't try to open a window; just print periodic status.
        #[arg(long)]
        no_window: bool,
        /// Reconnect after a dropped session instead of exiting.
        #[arg(long)]
        reconnect: bool,
    },
    /// Report what this machine can do, whether audio capture is available,
    /// and which path to the internet it has. Connects to nothing.
    Diagnose {
        /// List available displays for --display/--region instead of the
        /// network report.
        #[arg(long)]
        displays: bool,
    },
    /// Print a single line a viewer can open or paste, carrying the token
    /// and the certificate pin so neither has to be retyped.
    Pair {
        /// Host and port the viewer should connect to, as host:port.
        #[arg(long)]
        listen: String,
        /// Certificate fingerprint the viewer must pin.
        #[arg(long)]
        pin: String,
        /// The viewer token. Generated and printed if omitted.
        #[arg(long)]
        token: Option<String>,
    },
    /// Run a relay so a sharer and a viewer that cannot reach each other
    /// directly can still connect: both sides dial out to this relay.
    Relay {
        /// Address to listen on, as host:port.
        #[arg(long, default_value_t = format!("0.0.0.0:{DEFAULT_RELAY_PORT}"))]
        listen: String,
        /// Viewers and hosts must present this token. Generated and
        /// printed if omitted.
        #[arg(long)]
        token: Option<String>,
    },
}

fn token_from(arg: Option<&String>, what: &str) -> Result<SessionToken> {
    match arg {
        Some(t) => SessionToken::parse(t).with_context(|| format!("Invalid {what}")),
        None => {
            let t = SessionToken::generate();
            tracing::info!("Generated {what}: {}", t.as_str());
            Ok(t)
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // The guard must outlive everything, or the last events are dropped.
    let _log_guard =
        telemetry::logging::init(&cli.log_level, cli.log_format, cli.log_file.as_deref())?;
    // One runtime for the whole process. The telemetry tasks below are
    // spawned before the subcommand's own runtime exists, so they need a
    // reactor already in place.
    let rt = tokio::runtime::Runtime::new()?;
    let metrics = Metrics::shared();
    if cli.stats_interval > 0 {
        telemetry::spawn_interval_reporter(
            rt.handle(),
            metrics.clone(),
            std::time::Duration::from_secs(cli.stats_interval),
        );
    }
    if let Some(addr) = &cli.metrics_listen {
        let addr = addr.to_string();
        let metrics = metrics.clone();
        rt.block_on(async move {
            let addr = pixel_change_check_client::network::resolve(&addr)
                .await
                .with_context(|| format!("Invalid --metrics-listen address '{addr}'"))?;
            tokio::spawn(async move {
                if let Err(e) = telemetry::logging::serve_metrics(addr, metrics).await {
                    tracing::error!("Metrics endpoint stopped: {e}");
                }
            });
            Ok::<(), anyhow::Error>(())
        })?;
    }

    match cli.command {
        Commands::Share {
            listen,
            no_listen,
            relay: relay_addr,
            relay_pin,
            session,
            web,
            no_web,
            web_cert,
            web_key,
            synthetic,
            display,
            region,
            window,
            application,
            fps,
            max_fps,
            quality,
            token,
            repair_secs,
            reach,
            audio,
        } => {
            let token = token_from(token.as_ref(), "viewer token")?;
            let args = share::ShareArgs {
                listen: if no_listen { None } else { Some(listen) },
                relay: relay_addr,
                relay_pin: relay_pin.unwrap_or_default(),
                session,
                web: if no_web { None } else { Some(web) },
                web_cert: match (web_cert, web_key) {
                    (Some(c), Some(k)) => Some((c, k)),
                    (None, None) => None,
                    _ => anyhow::bail!("--web-cert and --web-key must be given together"),
                },
                synthetic,
                capture_target: match (&display, &region, &window, &application) {
                    (None, None, None, None) => {
                        pixel_change_check_client::capture::CaptureTarget::Display(0)
                    }
                    (Some(i), None, None, None) => {
                        pixel_change_check_client::capture::CaptureTarget::Display(*i)
                    }
                    (None, Some(r), None, None) => {
                        let (x, y, w, h) =
                            pixel_change_check_client::capture::CaptureTarget::parse_region(r)
                                .with_context(|| format!("Invalid --region '{r}'"))?;
                        pixel_change_check_client::capture::CaptureTarget::Region {
                            x,
                            y,
                            width: w,
                            height: h,
                        }
                    }
                    (None, None, Some(w), None) => {
                        pixel_change_check_client::capture::CaptureTarget::Window(w.clone())
                    }
                    (None, None, None, Some(a)) => {
                        pixel_change_check_client::capture::CaptureTarget::Application(a.clone())
                    }
                    _ => anyhow::bail!(
                        "pick one capture source: --display, --region, --window, or --application"
                    ),
                },
                fps,
                max_fps,
                quality,
                token,
                repair_interval: Duration::from_secs(repair_secs),
                reach: match reach {
                    ReachPolicyArg::Auto => reach::ReachPolicy::TryDirect,
                    ReachPolicyArg::Direct => reach::ReachPolicy::DirectOnly,
                    ReachPolicyArg::Relay => reach::ReachPolicy::RelayOnly,
                },
                audio,
            };
            rt.block_on(share::run_share(args, metrics))
        }
        Commands::View {
            connect,
            relay: relay_addr,
            session,
            pin,
            token,
            no_window,
            reconnect,
        } => {
            let args = view::ViewArgs {
                connect,
                relay: relay_addr,
                session,
                pin,
                token: SessionToken::parse(&token).context("Invalid --token")?,
                show_window: !no_window,
                reconnect,
            };
            view::run_view(args, metrics)
        }
        Commands::Pair {
            listen,
            pin,
            token: pair_token,
        } => {
            let t = token_from(pair_token.as_ref(), "viewer token")?;
            println!("{}", reach::pair_url(&listen, &pin, t.as_str()));
            Ok(())
        }
        Commands::Diagnose { displays } => {
            if displays {
                for (i, w, h) in pixel_change_check_client::capture::ScreenCapture::list_displays()
                {
                    println!("display {i}: {w}x{h}");
                }
                return Ok(());
            }
            // Answers "do I need a relay?" without contacting anyone. The
            // STUN probe is a single UDP round trip to a public server,
            // which is the only rung that can be tested without a peer.
            let report = rt.block_on(pixel_change_check_client::reach::diagnose(None));
            print!("{}", pixel_change_check_client::reach::render(&report));
            Ok(())
        }
        Commands::Relay { listen, token } => {
            let token = token_from(token.as_ref(), "relay token")?;
            println!("relay token: {}", token.as_str());
            let identity = std::sync::Arc::new(
                pixel_change_check_client::network::generate_identity()
                    .context("Failed to create the relay identity")?,
            );
            rt.block_on(async move {
                let addr = pixel_change_check_client::network::resolve(&listen)
                    .await
                    .with_context(|| format!("Invalid --listen address '{listen}'"))?;
                let listener = tokio::net::TcpListener::bind(addr)
                    .await
                    .with_context(|| format!("Failed to bind relay on {addr}"))?;
                let bound = listener.local_addr()?;
                println!("relay listening on {bound}");
                relay::run_relay_server(listener, identity, token).await
            })
        }
    }
}
