use serde::{Deserialize, Serialize};
use std::time::Duration;
use rustls::{ClientConfig, ServerConfig};
use rcgen::generate_simple_self_signed;

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
    pub fn client_crypto_config(&self) -> ClientConfig {
        ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth()
    }

    pub fn server_crypto_config(&self) -> ServerConfig {
        // Generate a self-signed certificate for testing
        let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let key_der = cert.serialize_private_key_der();
        let cert_der = cert.serialize_der().unwrap();
        
        let mut server_crypto = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![rustls::Certificate(cert_der)], rustls::PrivateKey(key_der))
            .unwrap();
            
        server_crypto.alpn_protocols = vec![b"pcc".to_vec()];
        server_crypto
    }
} 