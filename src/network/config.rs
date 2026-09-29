//! Transport configuration, the viewer's authorization credential, and
//! certificate pinning.
//!
//! Two separate things are established here, deliberately kept apart:
//!
//! * **transport identity** -- the viewer pins the sharer's certificate
//!   fingerprint, so it knows it is talking to *that* sharer and not to
//!   whoever is in the middle;
//! * **authorization** -- the viewer presents a session token, so knowing
//!   the fingerprint (which is not a secret: it is on the wire in
//!   cleartext inside the TLS handshake) still does not entitle you to
//!   watch the screen.

use crate::network::MAX_TOKEN_LEN;
use anyhow::{Context, Result};
use rcgen::generate_simple_self_signed;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;

/// Characters used for human-typeable secrets. No look-alikes, so a code
/// read aloud or copied off a screen is not ambiguous.
const TOKEN_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Length of a generated token: 26 symbols of 32 = 130 bits.
const GENERATED_TOKEN_LEN: usize = 26;

/// A viewer's credential. Compared in constant time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionToken(String);

impl SessionToken {
    /// Generate a fresh high-entropy token.
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let s: String = (0..GENERATED_TOKEN_LEN)
            .map(|_| TOKEN_ALPHABET[rng.gen_range(0..TOKEN_ALPHABET.len())] as char)
            .collect();
        Self(s)
    }

    /// Validate a token supplied by a user or read from a command line.
    pub fn parse(raw: &str) -> Result<Self> {
        let t = raw.trim();
        if t.len() < 8 {
            anyhow::bail!("token too short: {} chars (min 8)", t.len());
        }
        if t.len() > MAX_TOKEN_LEN as usize {
            anyhow::bail!("token too long: {} chars (max {})", t.len(), MAX_TOKEN_LEN);
        }
        if !t
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            anyhow::bail!("token contains unsupported characters (use letters, digits, - and _)");
        }
        Ok(Self(t.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Constant-time equality. Length is compared first, which is fine: a
/// length mismatch is not the secret being guessed.
pub fn verify_token(expected: &SessionToken, presented: &str) -> bool {
    let a = expected.as_str().as_bytes();
    let b = presented.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// The credential a peer shows the relay, derived from the session token.
///
/// The relay must be able to authenticate a session without being told
/// the session secret. The secret is what authenticates the
/// end-to-end encryption handshake, so a relay that learned it could
/// forge a viewer's proof and sit in the middle of a stream it claims
/// not to be able to read. Deriving a separate value keeps the relay's
/// knowledge useless for that: it can prove it belongs to the session
/// without being able to impersonate anyone inside it.
///
/// Domain-separated, so this value can never collide with a proof or a
/// key derived anywhere else.
pub fn relay_credential(token: &SessionToken) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac =
        Hmac::<Sha256>::new_from_slice(b"pcc/relay/v1").expect("Hmac accepts a key of any length");
    mac.update(token.as_str().as_bytes());
    hex(&mac.finalize().into_bytes())
}

/// The sharer's self-signed certificate plus the fingerprint a viewer pins.
#[derive(Clone)]
pub struct ServerIdentity {
    pub certificate: Vec<u8>,
    pub private_key: Vec<u8>,
    pub fingerprint: String,
}

/// Build a self-signed identity and report its SHA-256 fingerprint.
pub fn generate_identity() -> Result<ServerIdentity> {
    let cert = generate_simple_self_signed(vec!["pcc".to_string()])
        .context("Failed to generate a self-signed certificate")?;
    let private_key = cert.key_pair.serialize_der();
    let certificate = cert.cert.der().to_vec();
    let fingerprint = fingerprint_hex(&certificate);
    Ok(ServerIdentity {
        certificate,
        private_key,
        fingerprint,
    })
}

/// Lowercase hex of a 32-byte digest.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Lowercase hex SHA-256 of a DER certificate.
pub fn fingerprint_hex(der: &[u8]) -> String {
    let digest = Sha256::digest(der);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        out.push(char::from_digit((b & 0xF) as u32, 16).unwrap());
    }
    out
}

/// Decode a hex SHA-256 fingerprint into the 32 bytes a pin compares
/// against.
pub fn hex_to_der(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim();
    anyhow::ensure!(
        hex.len().is_multiple_of(2),
        "a SHA-256 fingerprint is 64 hex characters, got {}",
        hex.len()
    );
    let mut out = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks(2) {
        let s = std::str::from_utf8(pair)?;
        out.push(
            u8::from_str_radix(s, 16)
                .map_err(|_| anyhow::anyhow!("'{s}' is not a hex byte in the fingerprint"))?,
        );
    }
    anyhow::ensure!(
        out.len() == 32,
        "a SHA-256 fingerprint must decode to 32 bytes, got {}",
        out.len()
    );
    Ok(out)
}

/// The one crypto provider this build uses. Named in a single place so
/// the ambiguity between the providers rustls can auto-select from is
/// resolved here rather than by which dependency happens to be enabled.
fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Accept only the certificate whose SHA-256 fingerprint we were told to
/// expect.
///
/// The pin is a *digest*, not the certificate: 32 bytes that fit on a
/// command line, rather than a few hundred bytes of DER. Comparing the
/// presented certificate's digest against the pin is what makes those two
/// representations interchangeable.
#[derive(Debug)]
struct PinningVerifier {
    expected: [u8; 32],
}

impl rustls::client::danger::ServerCertVerifier for PinningVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let presented: [u8; 32] = Sha256::digest(end_entity).into();
        if presented == self.expected {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(format!(
                "server certificate fingerprint mismatch: expected sha256:{}, got sha256:{}",
                hex(&self.expected),
                hex(&presented),
            )))
        }
    }

    // Pinning replaces the chain-of-trust check, not the signature check:
    // the peer's key must still prove it holds the pinned certificate's
    // private key, or a third party could replay a copy of it.
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// How long a connection may sit idle before QUIC gives up on it.
    /// Matters most over the internet, where a stalled/rebooted peer
    /// should eventually free up resources instead of hanging forever.
    pub connection_timeout: Duration,
    /// How often QUIC sends a protocol-level keep-alive. Holds NAT/firewall
    /// UDP mappings open on top of the app-level keep-alives.
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

impl NetworkConfig {
    /// Client TLS config that pins one exact certificate, given its
    /// 32-byte SHA-256 fingerprint.
    pub fn client_tls_config(pin: &[u8]) -> Result<rustls::ClientConfig> {
        let expected: [u8; 32] = pin.try_into().map_err(|_| {
            anyhow::anyhow!(
                "a certificate pin is a 32-byte SHA-256 fingerprint, got {} bytes",
                pin.len()
            )
        })?;
        // A pin replaces the root store: there is no chain to build, and
        // an empty one makes that explicit rather than accidental.
        // The provider is named rather than inferred. Something else in
        // the tree enables `aws-lc-rs`, and with both present rustls
        // refuses to guess and panics at the first handshake.
        let mut config = rustls::ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(rustls::ALL_VERSIONS)
            .expect("the ring provider supports these versions")
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(PinningVerifier { expected }));
        config.alpn_protocols = vec![b"pcc".to_vec()];
        Ok(config)
    }

    /// Server TLS config from an already-generated identity.
    pub fn server_crypto_config(identity: &ServerIdentity) -> Result<rustls::ServerConfig> {
        let mut config = rustls::ServerConfig::builder_with_provider(provider())
            .with_protocol_versions(rustls::ALL_VERSIONS)
            .expect("the ring provider supports these versions")
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::pki_types::CertificateDer::from(
                    identity.certificate.clone(),
                )],
                rustls::pki_types::PrivatePkcs8KeyDer::from(identity.private_key.clone()).into(),
            )
            .map_err(|e| anyhow::anyhow!("Failed to build the QUIC server config: {e}"))?;
        config.alpn_protocols = vec![b"pcc".to_vec()];
        Ok(config)
    }

    /// QUIC transport config (idle timeout, keep-alive) shared by both ends.
    pub fn transport_config(&self) -> Arc<quinn::TransportConfig> {
        let mut transport = quinn::TransportConfig::default();
        if let Ok(idle_timeout) = quinn::IdleTimeout::try_from(self.connection_timeout) {
            transport.max_idle_timeout(Some(idle_timeout));
        }
        transport.keep_alive_interval(Some(self.keepalive_interval));
        Arc::new(transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_usable_and_distinct() {
        let a = SessionToken::generate();
        let b = SessionToken::generate();
        assert_ne!(a, b);
        assert!(SessionToken::parse(a.as_str()).is_ok());
    }

    #[test]
    fn weak_tokens_are_refused_with_a_reason() {
        assert!(SessionToken::parse("short").is_err());
        assert!(SessionToken::parse("has spaces here").is_err());
        assert!(SessionToken::parse(&"A".repeat(200)).is_err());
    }

    #[test]
    fn token_verification_rejects_wrong_and_right() {
        let t = SessionToken::parse("ABCD2345EFGH").unwrap();
        assert!(verify_token(&t, "ABCD2345EFGH"));
        assert!(!verify_token(&t, "ABCD2345EFG"));
        assert!(!verify_token(&t, "ABCD2345EFGX"));
        assert!(!verify_token(&t, ""));
    }

    #[test]
    fn fingerprints_round_trip_through_hex() {
        let id = generate_identity().unwrap();
        assert_eq!(hex_to_der(&id.fingerprint).unwrap().len(), 32);
    }

    #[test]
    fn a_malformed_fingerprint_is_refused_with_its_shape() {
        let err = hex_to_der("abc").unwrap_err().to_string();
        assert!(err.contains("64 hex characters"), "unhelpful: {err}");
        let err = hex_to_der(&"zz".repeat(32)).unwrap_err().to_string();
        assert!(err.contains("not a hex byte"), "unhelpful: {err}");
    }

    #[test]
    fn fingerprint_is_stable_hex() {
        let id = generate_identity().unwrap();
        assert_eq!(id.fingerprint.len(), 64);
        assert!(id.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(fingerprint_hex(&id.certificate), id.fingerprint);
    }
}
