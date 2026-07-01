use anyhow::Result;
use clap::{Parser, Subcommand};
use pixel_change_check_client::app::{share, view};
use pixel_change_check_client::network::DEFAULT_PORT;
use pixel_change_check_client::relay;
use std::net::SocketAddr;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const DEFAULT_RELAY_PORT: u16 = 5900;
const DEFAULT_WEB_PORT: u16 = 8080;

#[derive(Parser)]
#[command(name = "pcc", version, about = "PixelChangeCheck: an efficient screen-sharing tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture your screen and share it: to direct viewers, through a
    /// relay, and/or as a browser stream any device (incl. iPhone) can open.
    Share {
        /// Address to listen on for direct viewer connections.
        #[arg(long, default_value_t = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT)))]
        listen: SocketAddr,
        /// Disable the direct QUIC listener.
        #[arg(long)]
        no_listen: bool,
        /// Relay server address (host:port), for viewers who can't connect
        /// to you directly (e.g. you're both behind NAT / on the internet
        /// without port-forwarding).
        #[arg(long)]
        relay: Option<SocketAddr>,
        /// Relay session code to use. Auto-generated (and printed) if omitted.
        #[arg(long)]
        session: Option<String>,
        /// Port for the browser-based MJPEG viewer (open http://<this-host>:<port>/
        /// from any browser, including an iPhone's Safari, to watch).
        #[arg(long, default_value_t = DEFAULT_WEB_PORT)]
        web_port: u16,
        /// Disable the browser viewer entirely.
        #[arg(long)]
        no_web: bool,
        /// Use a synthetic animated test pattern instead of capturing the
        /// real screen. Useful on headless machines / for local testing.
        #[arg(long)]
        synthetic: bool,
        /// Target frames per second.
        #[arg(long, default_value_t = 30)]
        fps: u32,
        /// Initial JPEG quality, 0.0-1.0.
        #[arg(long, default_value_t = 0.8)]
        quality: f32,
    },
    /// Connect to a shared session and view it in a native window.
    View {
        /// Connect directly to a sharer's listen address.
        #[arg(long)]
        connect: Option<SocketAddr>,
        /// Connect through a relay instead of directly.
        #[arg(long)]
        relay: Option<SocketAddr>,
        /// Relay session code (required with --relay).
        #[arg(long)]
        session: Option<String>,
        /// Don't try to open a window; just print periodic status.
        #[arg(long)]
        no_window: bool,
    },
    /// Run a relay server, so sharers and viewers that can't reach each
    /// other directly (both behind NAT, over the internet) can still
    /// connect: both sides dial out to this relay.
    Relay {
        #[arg(long, default_value_t = SocketAddr::from(([0, 0, 0, 0], DEFAULT_RELAY_PORT)))]
        listen: SocketAddr,
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

fn main() -> Result<()> {
    init_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Share {
            listen,
            no_listen,
            relay: relay_addr,
            session,
            web_port,
            no_web,
            synthetic,
            fps,
            quality,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            let args = share::ShareArgs {
                listen: if no_listen { None } else { Some(listen) },
                relay: relay_addr,
                session,
                web: if no_web {
                    None
                } else {
                    Some(SocketAddr::from(([0, 0, 0, 0], web_port)))
                },
                synthetic,
                fps,
                quality,
            };
            rt.block_on(share::run_share(args))
        }
        Commands::View {
            connect,
            relay: relay_addr,
            session,
            no_window,
        } => {
            let args = view::ViewArgs {
                connect,
                relay: relay_addr,
                session,
                show_window: !no_window,
            };
            view::run_view(args)
        }
        Commands::Relay { listen } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(relay::run_relay_server(listen))
        }
    }
}
