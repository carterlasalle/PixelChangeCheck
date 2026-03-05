use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use rustls::{self, client::ServerCertVerified, client::ServerCertVerifier};
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

/// A certificate verifier that accepts any certificate.
/// WARNING: This skips TLS certificate verification and should ONLY be used
/// for localhost testing and development. Do not use in production.
struct SkipServerVerification;

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

impl NetworkConfig {
    pub fn client_crypto_config(&self) -> rustls::ClientConfig {
        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        config.alpn_protocols = vec![b"pcc".to_vec()];
        config
    }

    pub fn server_crypto_config(&self) -> rustls::ServerConfig {
        // Generate a self-signed certificate for testing
        let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let key_der = cert.serialize_private_key_der();
        let cert_der = cert.serialize_der().unwrap();

        let mut server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::Certificate(cert_der)],
                rustls::PrivateKey(key_der),
            )
            .unwrap();

        server_crypto.alpn_protocols = vec![b"pcc".to_vec()];
        server_crypto
    }
} 