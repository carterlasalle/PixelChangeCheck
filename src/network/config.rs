use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub port: Option<u16>,
    pub max_packet_size: usize,
    pub target_bandwidth: usize,
    pub connection_timeout: Duration,
    pub keepalive_interval: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: None,
            max_packet_size: 1400, // Standard MTU size minus headers
            target_bandwidth: 5_000_000, // 5MB/s
            connection_timeout: Duration::from_secs(10),
            keepalive_interval: Duration::from_secs(5),
        }
    }
}

impl NetworkConfig {
    pub fn crypto_config(&self) -> rustls::ClientConfig {
        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth()
    }
} 