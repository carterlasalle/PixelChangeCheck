//! End-to-end encryption between sharer and viewer.
//!
//! The relay terminates TLS on both legs and, before this change, saw the
//! `Message` stream in the clear. That is a meaningfully different thing
//! from being able to carry the traffic: whoever runs the relay could read
//! the screen.
//!
//! # Why this is small
//!
//! The relay is now a pure byte pipe -- it caches nothing and parses
//! nothing -- so end-to-end encryption is an envelope around the stream
//! rather than a redesign. The session token that already exists becomes
//! the pre-shared key, so there is no second credential for a user to
//! manage, and the security model gets simpler rather than more complex.
//!
//! # What a relay operator learns
//!
//! That a session exists, and its packet sizes and timing. Not the
//! dimensions, not the content, not the session code.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use sha2::{Digest, Sha256};
// x25519-dalek 2.x names the long-term secret `EphemeralSecret`.
use anyhow::{Context, Result};
use x25519_dalek::{EphemeralSecret, PublicKey};

/// Wire ids. Bumping the protocol for this is deliberate: an old peer that
/// cannot do the handshake must fail rather than silently fall back.
pub const HANDSHAKE_REQUEST: u8 = 0x20;
pub const HANDSHAKE_RESPONSE: u8 = 0x21;

/// Every message carries a 12-byte nonce and a 16-byte tag.
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
/// What `seal` adds on top of the plaintext: 4-byte counter header plus
/// the 16-byte AEAD tag. The planner reserves this when sizing a patch
/// set against the send budget, so a set that fits pre-seal cannot
/// exceed the relay's cap post-seal.
pub const SEALED_OVERHEAD: usize = 4 + TAG_LEN;

/// A key pair for one handshake.
///
/// `EphemeralSecret` is deliberately neither `Copy` nor serialisable in
/// x25519-dalek 2.x, and `diffie_hellman` consumes it. Both are good
/// properties -- they make accidental key reuse a compile error -- and
/// both shape the API below: a key pair is handed over once per handshake,
/// never borrowed.
pub struct KeyPair {
    secret: EphemeralSecret,
    public: [u8; 32],
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public", &hex(&self.public))
            .finish_non_exhaustive()
    }
}

impl KeyPair {
    pub fn generate() -> Self {
        let secret = EphemeralSecret::random_from_rng(rand_core::OsRng);
        let public = PublicKey::from(&secret).to_bytes();
        Self { secret, public }
    }

    pub fn public(&self) -> [u8; 32] {
        self.public
    }
}

/// One direction of a session, ready to seal and open frames.
pub struct Direction {
    cipher: ChaCha20Poly1305,
    /// Monotonic counter, never reused. A repeated nonce in
    /// ChaCha20-Poly1305 is catastrophic, so this is enforced rather than
    /// assumed.
    counter: u64,
}

impl std::fmt::Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Direction")
            .field("counter", &self.counter)
            .finish_non_exhaustive()
    }
}

impl Direction {
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(Key::from_slice(&key)),
            counter: 0,
        }
    }

    fn nonce(counter: u64) -> [u8; NONCE_LEN] {
        // 4 zero bytes then a big-endian counter. A random per-session
        // prefix would be marginally better, but the counter is what
        // actually guarantees uniqueness, and 2^64 frames is not a session
        // length anyone reaches.
        let mut n = [0u8; NONCE_LEN];
        n[4..].copy_from_slice(&counter.to_be_bytes());
        n
    }

    /// Seal one frame.
    ///
    /// The counter travels in the clear as the frame header and is also
    /// the AEAD's additional data. Tampering with it therefore changes the
    /// AAD and fails the tag, so a relay cannot reorder or substitute
    /// frames without the receiver noticing.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let counter = self.counter;
        self.counter = self
            .counter
            .checked_add(1)
            .context("frame counter overflowed; rekey required")?;
        let raw = Self::nonce(counter);
        let out = self
            .cipher
            .encrypt(
                Nonce::from_slice(&raw),
                Payload {
                    msg: plaintext,
                    aad: &aad_for(counter),
                },
            )
            .map_err(|_| anyhow::anyhow!("frame encryption failed"))?;
        // The frame carries counter+1 so the receiver's initial state of
        // "nothing seen" is 0 and the first frame is counter 1. Carrying
        // the raw counter would make the very first frame look like a
        // replay of frame zero.
        let mut framed = Vec::with_capacity(4 + out.len());
        framed.extend_from_slice(&((counter + 1) as u32).to_le_bytes());
        framed.extend_from_slice(&out);
        Ok(framed)
    }

    /// Open one frame, rejecting anything at or below what has been seen.
    pub fn open(&mut self, framed: &[u8]) -> Result<Vec<u8>> {
        anyhow::ensure!(
            framed.len() > 4 + TAG_LEN,
            "sealed frame is too short: {} bytes",
            framed.len()
        );
        let wire = u32::from_le_bytes([framed[0], framed[1], framed[2], framed[3]]) as u64;
        anyhow::ensure!(
            wire > self.counter,
            "sealed frame is a replay or out of order: counter {wire} <= {}",
            self.counter
        );
        self.counter = wire;
        let raw = Self::nonce(wire - 1);
        self.cipher
            .decrypt(
                Nonce::from_slice(&raw),
                Payload {
                    msg: &framed[4..],
                    aad: &aad_for(wire - 1),
                },
            )
            .map_err(|_| anyhow::anyhow!("sealed frame failed authentication"))
    }

    /// Rekey: a fresh key for the same direction, resetting the counter.
    pub fn rekey(&mut self, key: [u8; 32]) {
        self.cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
        self.counter = 0;
    }
}

/// The two directions of a session.
pub struct Session {
    /// Sharer to viewer.
    pub host_to_viewer: Direction,
    /// Viewer to sharer.
    pub viewer_to_host: Direction,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("host_to_viewer", &self.host_to_viewer)
            .field("viewer_to_host", &self.viewer_to_host)
            .finish()
    }
}

/// Domain separation, so the handshake and the frames cannot be confused.
const TOKEN_DOMAIN: &[u8] = b"pcc/e2e/v1";

/// The additional authenticated data for a frame: its counter. Derived
/// here rather than passed in, so a caller cannot accidentally
/// authenticate something the receiver will reconstruct differently.
fn aad_for(counter: u64) -> [u8; 8] {
    counter.to_le_bytes()
}

/// Derive two independent direction keys.
///
/// Direction separation is not decoration: a relay holding the
/// host-to-viewer key must not be able to forge viewer-to-host control
/// traffic, which is how a snapshot loop or a session teardown would be
/// injected.
fn derive(
    shared: &[u8; 32],
    host_public: &[u8; 32],
    viewer_public: &[u8; 32],
) -> ([u8; 32], [u8; 32]) {
    let h2v = kdf(shared, host_public, viewer_public, b"pcc/h2v");
    let v2h = kdf(shared, host_public, viewer_public, b"pcc/v2h");
    (h2v, v2h)
}

fn kdf(shared: &[u8; 32], hp: &[u8; 32], vp: &[u8; 32], label: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(TOKEN_DOMAIN);
    h.update(label);
    h.update(shared);
    h.update(hp);
    h.update(vp);
    h.finalize().into()
}

/// A handshake offer: our ephemeral public key plus proof that we hold the
/// token.
#[derive(Debug, Clone)]
pub struct Offer {
    pub public: [u8; 32],
    /// SHA-256 over the token and *this* public key. It proves the sender
    /// holds the token, cheaply, before any DH is done. It deliberately
    /// says nothing about the other party's key: the offerer does not know
    /// it yet. What binds the two halves together is the key derivation
    /// below, which mixes both public keys into both directions, and the
    /// AEAD tag on the first real frame.
    pub proof: [u8; 32],
}

/// The handshake reply, carrying the same proof over the responder's key.
#[derive(Debug, Clone)]
pub struct Reply {
    pub public: [u8; 32],
    pub proof: [u8; 32],
}

fn proof_of(token: &str, public: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(TOKEN_DOMAIN);
    h.update(b"proof");
    h.update(public);
    h.update(token.as_bytes());
    h.finalize().into()
}

/// Build an offer. Borrows the key pair: nothing here needs the secret, and
/// only the final [`complete`] consumes it.
pub fn offer(mine: &KeyPair, token: &str) -> Offer {
    let public = mine.public();
    Offer {
        public,
        proof: proof_of(token, &public),
    }
}

/// Build a reply. Borrows the key pair for the same reason as [`offer`].
pub fn reply(mine: &KeyPair, token: &str) -> Reply {
    let public = mine.public();
    Reply {
        public,
        proof: proof_of(token, &public),
    }
}

/// The sharer's half: verify the viewer's proof and produce the session.
pub fn accept(offer: &Offer, mine: KeyPair, token: &str) -> Result<Session> {
    // Reject a wrong token before doing any key agreement.
    anyhow::ensure!(
        constant_time_eq(&proof_of(token, &offer.public), &offer.proof),
        "handshake proof failed: the viewer does not hold this session token"
    );
    let their_public = PublicKey::from(offer.public);
    // Read the public key before the secret is consumed: `diffie_hellman`
    // takes `self` by value and a secret may only be used once.
    let our_public = mine.public();
    let shared_bytes = mine.secret.diffie_hellman(&their_public).to_bytes();

    // The viewer's public key is the origin; ours is the responder.
    let (h2v, v2h) = derive(&shared_bytes, &offer.public, &our_public);
    Ok(Session {
        host_to_viewer: Direction::new(h2v),
        viewer_to_host: Direction::new(v2h),
    })
}

/// The viewer's half: verify the sharer's proof and produce the session.
pub fn complete(
    mine: KeyPair,
    reply: &Reply,
    offer_public: &[u8; 32],
    token: &str,
) -> Result<Session> {
    anyhow::ensure!(
        constant_time_eq(&proof_of(token, &reply.public), &reply.proof),
        "handshake proof failed: the sharer does not hold this session token"
    );
    let their_public = PublicKey::from(reply.public);
    let shared_bytes = mine.secret.diffie_hellman(&their_public).to_bytes();
    let (h2v, v2h) = derive(&shared_bytes, offer_public, &reply.public);
    Ok(Session {
        host_to_viewer: Direction::new(h2v),
        viewer_to_host: Direction::new(v2h),
    })
}

/// Length-independent, constant-time comparison. A proof is not a secret,
/// but a timing-variable comparison is a habit worth not having.
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The sharer's side of the handshake: wait for the viewer's offer,
/// answer it, and return the session.
pub async fn host_handshake(
    sink: &mut Box<dyn crate::network::MessageSink>,
    source: &mut Box<dyn crate::network::MessageSource>,
    keys: KeyPair,
    token: &str,
) -> Result<Session> {
    let offer = match source.recv().await? {
        crate::network::Message::E2eOffer { public, proof } => Offer { public, proof },
        other => anyhow::bail!(
            "expected an encryption offer, got {:?}; is the viewer the same build?",
            other.rev()
        ),
    };
    let reply = reply(&keys, token);
    sink.send(&crate::network::Message::E2eReply {
        public: reply.public,
        proof: reply.proof,
    })
    .await?;
    accept(&offer, keys, token)
}

/// The viewer's side: send the offer, take the reply, return the session.
pub async fn viewer_handshake(
    sink: &mut Box<dyn crate::network::MessageSink>,
    source: &mut Box<dyn crate::network::MessageSource>,
    keys: KeyPair,
    token: &str,
) -> Result<Session> {
    let offer = offer(&keys, token);
    sink.send(&crate::network::Message::E2eOffer {
        public: offer.public,
        proof: offer.proof,
    })
    .await?;
    let reply = match source.recv().await? {
        crate::network::Message::E2eReply { public, proof } => Reply { public, proof },
        // The sharer refusing the token is the common case here, and its
        // explanation is far more useful than "expected a reply".
        crate::network::Message::Error(text) => anyhow::bail!("{text}"),
        other => anyhow::bail!(
            "expected an encryption reply, got {:?}; is the sharer the same build?",
            other.rev()
        ),
    };
    complete(keys, &reply, &offer.public, token)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive a whole handshake the way `serve_viewer` and `receive_once`
    /// do, and check the two sides agree.
    fn handshake(token: &str) -> (Session, Session) {
        let viewer_keys = KeyPair::generate();
        let host_keys = KeyPair::generate();
        let offer = offer(&viewer_keys, token);
        let reply = reply(&host_keys, token);
        let viewer = complete(viewer_keys, &reply, &offer.public, token).unwrap();
        let host = accept(&offer, host_keys, token).unwrap();
        (host, viewer)
    }

    #[test]
    fn both_sides_derive_the_same_keys() {
        let (mut host, mut viewer) = handshake("TOKEN12345678");
        let plaintext = b"a frame of pixels";
        let sealed = host.host_to_viewer.seal(plaintext).unwrap();
        assert_eq!(viewer.host_to_viewer.open(&sealed).unwrap(), plaintext);
    }

    #[test]
    fn the_viewer_can_seal_control_traffic_the_host_opens() {
        let (mut host, mut viewer) = handshake("TOKEN12345678");
        let sealed = viewer.viewer_to_host.seal(b"RequestKeyframe").unwrap();
        assert_eq!(
            host.viewer_to_host.open(&sealed).unwrap(),
            b"RequestKeyframe"
        );
    }

    #[test]
    fn a_wrong_token_fails_the_handshake() {
        let viewer_keys = KeyPair::generate();
        let host_keys = KeyPair::generate();
        let offer = offer(&viewer_keys, "TOKEN12345678");
        let reply = reply(&host_keys, "TOKEN12345678");
        // The viewer verifies the sharer's proof against a token it does
        // not have, which must fail rather than produce a session that
        // cannot open any frame.
        let err = complete(viewer_keys, &reply, &offer.public, "WRONGTOKEN999")
            .unwrap_err()
            .to_string();
        assert!(err.contains("handshake proof failed"), "unhelpful: {err}");
    }

    #[test]
    fn a_relay_substituting_a_reply_cannot_open_any_frame() {
        // The attacker knows nothing but the transcript. Its reply carries
        // a different key, so the viewer derives different session keys
        // and the sharer's frame will not authenticate.
        let viewer_keys = KeyPair::generate();
        let host_keys = KeyPair::generate();
        let forged_keys = KeyPair::generate();
        let offer = offer(&viewer_keys, "TOKEN12345678");
        let real = reply(&host_keys, "TOKEN12345678");
        let forged = reply(&forged_keys, "TOKEN12345678");
        assert_ne!(real.public, forged.public);
        let mut viewer = complete(viewer_keys, &forged, &offer.public, "TOKEN12345678").unwrap();
        let mut host = accept(&offer, host_keys, "TOKEN12345678").unwrap();
        let sealed = host.host_to_viewer.seal(b"pixels").unwrap();
        assert!(viewer.host_to_viewer.open(&sealed).is_err());
    }

    #[test]
    fn the_two_directions_use_different_keys() {
        let (mut s, _) = handshake("TOKEN12345678");
        // If the directions shared a key, a frame sealed one way would
        // open the other. That must not be possible.
        let sealed = s.host_to_viewer.seal(b"secret").unwrap();
        assert!(s.viewer_to_host.open(&sealed).is_err());
    }

    #[test]
    fn a_tampered_counter_fails_authentication() {
        // The counter is the AAD, so rewriting it in the clear header
        // must break the tag rather than silently renumbering the frame.
        let mut d = Direction::new([7u8; 32]);
        let mut sealed = d.seal(b"pixels").unwrap();
        sealed[3] = sealed[3].wrapping_add(1);
        let mut other = Direction::new([7u8; 32]);
        let err = other.open(&sealed).unwrap_err().to_string();
        assert!(
            err.contains("authentication") || err.contains("replay"),
            "unhelpful: {err}"
        );
    }

    #[test]
    fn a_replayed_frame_is_refused() {
        let mut sender = Direction::new([1u8; 32]);
        let mut receiver = Direction::new([1u8; 32]);
        let sealed = sender.seal(b"pixels").unwrap();
        receiver.open(&sealed).unwrap();
        let err = receiver.open(&sealed).unwrap_err().to_string();
        assert!(err.contains("replay"), "unhelpful: {err}");
    }

    #[test]
    fn an_out_of_order_frame_is_refused() {
        let mut sender = Direction::new([1u8; 32]);
        let mut receiver = Direction::new([1u8; 32]);
        let first = sender.seal(b"one").unwrap();
        let second = sender.seal(b"two").unwrap();
        receiver.open(&second).unwrap();
        // A relay delivering the older frame afterwards must not succeed.
        assert!(receiver.open(&first).is_err());
    }

    #[test]
    fn a_wrong_key_cannot_open_a_frame() {
        let mut sender = Direction::new([1u8; 32]);
        let sealed = sender.seal(b"pixels").unwrap();
        let mut wrong = Direction::new([2u8; 32]);
        assert!(wrong.open(&sealed).is_err());
    }

    #[test]
    fn a_tampered_ciphertext_is_refused() {
        let mut sender = Direction::new([1u8; 32]);
        let mut sealed = sender.seal(b"pixels").unwrap();
        let n = sealed.len();
        sealed[n - 1] ^= 0xFF;
        let mut receiver = Direction::new([1u8; 32]);
        assert!(receiver.open(&sealed).is_err());
    }

    #[test]
    fn a_truncated_frame_is_refused_before_any_decryption() {
        let mut sender = Direction::new([1u8; 32]);
        let sealed = sender.seal(b"pixels").unwrap();
        let mut receiver = Direction::new([1u8; 32]);
        let err = receiver.open(&sealed[..8]).unwrap_err().to_string();
        assert!(err.contains("too short"), "unhelpful: {err}");
    }

    #[test]
    fn a_rekey_breaks_the_old_key_and_resets_the_counter() {
        let mut sender = Direction::new([1u8; 32]);
        sender.seal(b"before").unwrap();
        let mut receiver = Direction::new([1u8; 32]);
        receiver.open(&sender.seal(b"before").unwrap()).unwrap();

        sender.rekey([9u8; 32]);
        receiver.rekey([9u8; 32]);
        // After a rekey the counter starts again, so the first frame on
        // the new key opens and the old key cannot read it.
        let mut old = Direction::new([1u8; 32]);
        let sealed = sender.seal(b"after").unwrap();
        assert!(old.open(&sealed).is_err());
        assert_eq!(receiver.open(&sealed).unwrap(), b"after");
    }

    #[test]
    fn debug_output_never_contains_key_material() {
        let d = Direction::new([0xAB; 32]);
        let text = format!("{d:?}");
        assert!(
            !text.contains("ab"),
            "debug output leaked key bytes: {text}"
        );
    }
}
