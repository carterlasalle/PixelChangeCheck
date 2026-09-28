use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use pixel_change_check_client::app::{share, view};
use pixel_change_check_client::network::{SessionToken, DEFAULT_PORT};
use pixel_change_check_client::relay;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

const DEFAULT_RELAY_PORT: u16 = 5900;
const DEFAULT_WEB_PORT: u16 = 8080;

#[derive(Parser)]
#[command(
    name = "pcc",
    version,
    about = "PixelChangeCheck: exact, lossless desktop replication"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        /// PEM certificate for HTTPS/WSS on the browser port. Without it
        /// the browser path is plaintext (the token still authenticates).
        #[arg(long, requires = "web_key")]
        web_cert: Option<String>,
        /// PEM private key matching --web-cert.
        #[arg(long)]
        web_key: Option<String>,
        /// Use a synthetic animated test pattern instead of capturing the
        /// real screen. Useful on headless machines and for local tests.
        #[arg(long)]
        synthetic: bool,
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

fn init_logging() {
    let _ = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .compact()
        .try_init();
}

fn token_from(arg: Option<&String>, what: &str) -> Result<SessionToken> {
    match arg {
        Some(t) => SessionToken::parse(t).with_context(|| format!("Invalid {what}")),
        None => {
            let t = SessionToken::generate();
            info!("Generated {what}: {}", t.as_str());
            Ok(t)
        }
    }
}

fn main() -> Result<()> {
    init_logging();
    let cli = Cli::parse();

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
            fps,
            max_fps,
            quality,
            token,
            repair_secs,
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
                fps,
                max_fps,
                quality,
                token,
                repair_interval: Duration::from_secs(repair_secs),
            };
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(share::run_share(args))
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
            view::run_view(args)
        }
        Commands::Relay { listen, token } => {
            let token = token_from(token.as_ref(), "relay token")?;
            println!("relay token: {}", token.as_str());
            let identity = std::sync::Arc::new(
                pixel_change_check_client::network::generate_identity()
                    .context("Failed to create the relay identity")?,
            );
            let rt = tokio::runtime::Runtime::new()?;
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
