use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use rustls::{self, client::ServerCertVerified, client::ServerCertVerifier};
use rcgen::generate_simple_self_signed;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// How long a connection may sit idle before QUIC gives up on it.
    /// Matters most over the internet, where a stalled/rebooted peer
    /// should eventually free up resources instead of hanging forever.
    pub connection_timeout: Duration,
    /// How often QUIC sends a protocol-level keep-alive. Helps hold NAT/
    /// firewall UDP mappings open on top of the app-level keep-alives we
    /// already send once per capture frame.
    pub keepalive_interval: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
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

    /// Build the QUIC transport-level config (idle timeout, keep-alive)
    /// shared by both client and server endpoints.
    pub fn transport_config(&self) -> Arc<quinn::TransportConfig> {
        let mut transport = quinn::TransportConfig::default();
        if let Ok(idle_timeout) = quinn::IdleTimeout::try_from(self.connection_timeout) {
            transport.max_idle_timeout(Some(idle_timeout));
        }
        transport.keep_alive_interval(Some(self.keepalive_interval));
        Arc::new(transport)
    }
}
