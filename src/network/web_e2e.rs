//! End-to-end encryption for the browser viewer.
//!
//! # Why a different primitive from the native path
//!
//! The native client uses X25519 and ChaCha20-Poly1305. A browser cannot:
//! `crypto.subtle` has no X25519 in the versions most people run, but
//! ECDH on P-256, HKDF and AES-GCM are universally available. So the
//! browser gets a construction built entirely from primitives it
//! actually has, rather than one that only works on the newest Chrome.
//!
//! The *protocol* is the same shape: an ephemeral key exchange, a proof of
//! holding the session token, and then every frame sealed. Only the
//! primitives differ, and both sides of the wire agree because the browser
//! implementation is the matching JavaScript.
//!
//! # What a network attacker learns
//!
//! That a session exists, and its frame sizes and timing. Not the
//! dimensions, not the content, not the session code.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{Context, Result};
use hkdf::Hkdf;
use p256::ecdh::diffie_hellman;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey, SecretKey};
use sha2::Sha256;

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// Wire ids for the browser handshake. Distinct from the native ones so a
/// cross-wiring fails loudly instead of producing garbage keys.
pub const K_BROWSER_OFFER: u8 = 0x22;
pub const K_BROWSER_REPLY: u8 = 0x23;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserOffer {
    /// Uncompressed P-256 point, 65 bytes.
    pub public: [u8; 65],
    pub proof: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserReply {
    pub public: [u8; 65],
    pub proof: [u8; 32],
}

/// A browser key pair for one handshake.
pub struct BrowserKeyPair {
    secret: SecretKey,
    public: [u8; 65],
}

impl std::fmt::Debug for BrowserKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserKeyPair")
            .field("public_len", &self.public.len())
            .finish_non_exhaustive()
    }
}

impl BrowserKeyPair {
    pub fn generate() -> Self {
        let secret = SecretKey::random(&mut rand_core::OsRng);
        let point = secret.public_key().to_encoded_point(false);
        let mut public = [0u8; 65];
        public.copy_from_slice(point.as_bytes());
        Self { secret, public }
    }
}

/// Derive two direction keys from the ECDH shared secret.
///
/// `info` binds the keys to this protocol and version, so a key derived
/// for one cannot be replayed into another.
fn derive(
    shared: &[u8],
    offerer_public: &[u8; 65],
    responder_public: &[u8; 65],
    token: &str,
) -> ([u8; 32], [u8; 32]) {
    // HKDF-Extract then two Expand steps, so the two directions never
    // share a key even though they share a secret.
    let hk = Hkdf::<Sha256>::new(Some(b"pcc/web/v1".as_slice()), shared);
    let mut offer_to_respond = [0u8; 32];
    let mut respond_to_offer = [0u8; 32];
    hk.expand(b"o2r", &mut offer_to_respond)
        .expect("32 bytes fits the SHA-256 output");
    hk.expand(b"r2o", &mut respond_to_offer)
        .expect("32 bytes fits the SHA-256 output");
    let _ = (offerer_public, responder_public, token);
    (offer_to_respond, respond_to_offer)
}

/// Proof that the holder of `public` also holds the token.
///
/// Cheap on purpose: a wrong token fails here, before any key agreement
/// is done on the browser's behalf.
/// The same function, exposed so the JavaScript implementation can be
/// checked against a fixed vector rather than by inspection.
pub fn proof_of_public(public: &[u8; 65], token: &str) -> [u8; 32] {
    proof_of(public, token)
}

fn proof_of(public: &[u8; 65], token: &str) -> [u8; 32] {
    use sha2::Digest;
    let mut h = Sha256::new();
    h.update(b"pcc/web/v1");
    h.update(b"proof");
    h.update(public);
    h.update(token.as_bytes());
    h.finalize().into()
}

fn shared_secret(secret: &SecretKey, their_public: &[u8; 65]) -> Result<[u8; 32]> {
    let their = PublicKey::from_sec1_bytes(their_public)
        .map_err(|e| anyhow::anyhow!("peer sent a point that is not on the curve: {e}"))?;
    let dh = diffie_hellman(secret.to_nonzero_scalar(), their.as_affine());
    let mut out = [0u8; 32];
    out.copy_from_slice(dh.raw_secret_bytes());
    Ok(out)
}

fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Build an offer.
pub fn offer(mine: &BrowserKeyPair, token: &str) -> BrowserOffer {
    BrowserOffer {
        public: mine.public,
        proof: proof_of(&mine.public, token),
    }
}

/// Answer an offer, producing the session and the reply to send back.
pub fn accept(
    offer: &BrowserOffer,
    mine: &BrowserKeyPair,
    token: &str,
) -> Result<(BrowserReply, BrowserSession)> {
    anyhow::ensure!(
        constant_time_eq(&proof_of(&offer.public, token), &offer.proof),
        "browser handshake proof failed: the viewer does not hold this session token"
    );
    let shared = shared_secret(&mine.secret, &offer.public)?;
    let (o2r, r2o) = derive(&shared, &offer.public, &mine.public, token);
    Ok((
        BrowserReply {
            public: mine.public,
            proof: proof_of(&mine.public, token),
        },
        // The sharer is the responder, so it sends with r2o and opens
        // what arrives on o2r. Getting this backwards produces a
        // session that cannot open its own first frame.
        BrowserSession {
            send: Direction::new(r2o),
            receive: Direction::new(o2r),
        },
    ))
}

/// Complete an offer into a session, given the reply.
pub fn complete(
    mine: &BrowserKeyPair,
    offer: &BrowserOffer,
    reply: &BrowserReply,
    token: &str,
) -> Result<BrowserSession> {
    anyhow::ensure!(
        constant_time_eq(&proof_of(&reply.public, token), &reply.proof),
        "browser handshake proof failed: the sharer does not hold this session token"
    );
    let shared = shared_secret(&mine.secret, &reply.public)?;
    let (o2r, r2o) = derive(&shared, &offer.public, &reply.public, token);
    Ok(BrowserSession {
        send: Direction::new(o2r),
        receive: Direction::new(r2o),
    })
}

pub struct Direction {
    cipher: Aes256Gcm,
    counter: u64,
}

impl Direction {
    fn new(key: [u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new_from_slice(&key).expect("AES-256 takes 32 bytes"),
            counter: 0,
        }
    }

    /// The counter travels in the clear and is also the AAD, so a proxy
    /// rewriting it fails the tag.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let counter = self.counter;
        self.counter = self
            .counter
            .checked_add(1)
            .context("frame counter overflowed; rekey required")?;
        let nonce_bytes = nonce_for(counter);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let mut out = Vec::with_capacity(4 + plaintext.len() + TAG_LEN);
        out.extend_from_slice(&(counter as u32).to_le_bytes());
        out.extend_from_slice(
            &self
                .cipher
                .encrypt(
                    nonce,
                    Payload {
                        msg: plaintext,
                        aad: &counter.to_le_bytes(),
                    },
                )
                .map_err(|_| anyhow::anyhow!("browser frame encryption failed"))?,
        );
        Ok(out)
    }

    pub fn open(&mut self, framed: &[u8]) -> Result<Vec<u8>> {
        anyhow::ensure!(
            framed.len() > 4 + TAG_LEN,
            "sealed frame is too short: {} bytes",
            framed.len()
        );
        let counter = u32::from_le_bytes([framed[0], framed[1], framed[2], framed[3]]) as u64;
        anyhow::ensure!(
            counter >= self.counter,
            "sealed frame is a replay: counter {counter} < {}",
            self.counter
        );
        self.counter = counter + 1;
        let nonce_bytes = nonce_for(counter);
        let nonce = Nonce::from_slice(&nonce_bytes);
        self.cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &framed[4..],
                    aad: &counter.to_le_bytes(),
                },
            )
            .map_err(|_| anyhow::anyhow!("sealed browser frame failed authentication"))
    }
}

fn nonce_for(counter: u64) -> [u8; NONCE_LEN] {
    let mut n = [0u8; NONCE_LEN];
    n[4..].copy_from_slice(&counter.to_be_bytes());
    n
}

/// One browser session, with independent keys per direction.
#[derive(Debug)]
pub struct BrowserSession {
    /// Sharer to viewer.
    pub send: Direction,
    /// Viewer to sharer.
    pub receive: Direction,
}

impl std::fmt::Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Direction")
            .field("counter", &self.counter)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handshake() -> (BrowserSession, BrowserSession) {
        let viewer = BrowserKeyPair::generate();
        let sharer = BrowserKeyPair::generate();
        let token = "WEBTOKEN1234567";
        let o = offer(&viewer, token);
        let (r, host) = accept(&o, &sharer, token).unwrap();
        let client = complete(&viewer, &o, &r, token).unwrap();
        (host, client)
    }

    #[test]
    fn both_ends_agree_on_the_keys() {
        let (mut host, mut client) = handshake();
        let msg = b"a frame of pixels";
        let sealed = host.send.seal(msg).unwrap();
        assert_eq!(client.receive.open(&sealed).unwrap(), msg);
    }

    #[test]
    fn the_two_directions_use_different_keys() {
        let (mut host, mut client) = handshake();
        let sealed = host.send.seal(b"host to viewer").unwrap();
        assert!(
            client.send.open(&sealed).is_err(),
            "viewer-to-host must not open a host-to-viewer frame"
        );
    }

    #[test]
    fn a_wrong_token_fails_before_any_key_agreement() {
        let viewer = BrowserKeyPair::generate();
        let sharer = BrowserKeyPair::generate();
        let o = offer(&viewer, "RIGHTTOKEN1234");
        let err = accept(&o, &sharer, "WRONGTOKEN999")
            .unwrap_err()
            .to_string();
        assert!(err.contains("handshake proof failed"), "unhelpful: {err}");
    }

    #[test]
    fn a_reply_from_a_different_key_cannot_open_anything() {
        let viewer = BrowserKeyPair::generate();
        let sharer = BrowserKeyPair::generate();
        let attacker = BrowserKeyPair::generate();
        let token = "WEBTOKEN1234567";
        let o = offer(&viewer, token);
        let (real, mut host) = accept(&o, &sharer, token).unwrap();
        let forged = BrowserReply {
            public: attacker.public,
            proof: proof_of(&attacker.public, token),
        };
        let mut client = complete(&viewer, &o, &forged, token).unwrap();
        let sealed = host.send.seal(b"pixels").unwrap();
        assert!(client.receive.open(&sealed).is_err());
        assert_ne!(real.public, forged.public);
    }

    #[test]
    fn a_tampered_counter_fails_the_tag() {
        let (mut host, mut client) = handshake();
        let mut sealed = host.send.seal(b"pixels").unwrap();
        sealed[3] = sealed[3].wrapping_add(1);
        assert!(client.receive.open(&sealed).is_err());
    }

    #[test]
    fn a_replay_is_refused() {
        let (mut host, mut client) = handshake();
        let sealed = host.send.seal(b"pixels").unwrap();
        client.receive.open(&sealed).unwrap();
        assert!(client.receive.open(&sealed).is_err());
    }

    #[test]
    fn a_point_off_the_curve_is_refused_rather_than_accepted() {
        let mut bad = [0u8; 65];
        bad[0] = 0x04;
        bad[64] = 0xFF;
        let secret = BrowserKeyPair::generate();
        let err = shared_secret(&secret.secret, &bad).unwrap_err().to_string();
        assert!(
            err.contains("not on the curve") || err.contains("malformed"),
            "unhelpful: {err}"
        );
    }
}
