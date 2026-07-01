mod config;
mod transport;
pub mod resilience;
mod protocol;

pub use config::NetworkConfig;
pub use transport::{client_endpoint, connect_direct, server_endpoint, MessageTransport, QuicTransport};
pub use resilience::{ResilienceConfig, NetworkResilience};
pub use protocol::{Message, WireChange};

/// Default port a sharer listens on for direct viewer connections.
pub const DEFAULT_PORT: u16 = 5800;
