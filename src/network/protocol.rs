//! The one wire format, spoken by all three transports.
//!
//! Explicit little-endian encoding rather than `bincode`: the browser
//! client parses these bytes in JavaScript, and a binary serializer's
//! private layout is not something another runtime should have to know.
//! Every field is bounds-checked while decoding, and every length is checked
//! against a budget *before* the allocation it authorises.

use super::WireOp;
use crate::encoder::{SnapshotFormat, MAX_SNAPSHOT_BYTES};
use crate::pcc::QualityConfig;
use anyhow::{Context, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Bumped for the revision/epoch/handshake protocol (v5), for resume
/// (v6: `Hello` carries the viewer's last applied revision and epoch) and
/// for the cursor/preview/redirect control plane (v7: `CursorMove`,
/// `CursorHide`, `MotionPreview`, `Redirect`). Old builds fail at the
/// version gate instead of mis-parsing.
pub const PROTOCOL_VERSION: u8 = 7;

/// Largest encoded message body we will produce or accept.
///
/// Receipt: a 4K (3840x2160) RGB surface is 24.9 MB raw. A lossless
/// snapshot of one is ~1-3 MB on screen content, and a whole-frame update
/// is by definition the worst case we can emit. 16 MB leaves ~5x headroom
/// over the largest snapshot we intend to send while still capping a single
/// allocation well below anything that would destabilise a viewer.
pub const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;

/// Ceiling on rectangles in one update.
///
/// Receipt: a 1080p frame is 2,040 tiles; a 4K frame is 8,160. Anything
/// above the 4K tile count is not a plausible rectangle set for any
/// supported display and is a sign of a hostile or corrupt message.
pub const MAX_OPS_PER_UPDATE: u32 = 8_192;

/// Ceiling on the token exchanged in `Hello`. 64 bytes is far beyond any
/// code a human types.
pub const MAX_TOKEN_LEN: u16 = 64;

/// Ceiling on a length-prefixed `Error` string.
pub const MAX_ERROR_LEN: u16 = 1024;

/// Bytes per snapshot chunk. Chosen so a chunk plus its envelope always
/// fits under `MAX_MESSAGE_SIZE` with room to spare, and so a viewer
/// commits a snapshot in a handful of reads rather than thousands.
pub const SNAPSHOT_CHUNK_BYTES: usize = 1024 * 1024;

/// Ceiling on snapshot chunks. At 1 MiB each this is a 128 MiB snapshot,
/// twice the largest frame budget, so it can never be the binding limit.
pub const MAX_SNAPSHOT_CHUNKS: u32 = 128;

const K_HELLO: u8 = 0x01;
const K_E2E_OFFER: u8 = 0x12;
const K_E2E_REPLY: u8 = 0x13;
const K_REQUEST_KEYFRAME: u8 = 0x02;
const K_ACK: u8 = 0x03;
const K_SNAPSHOT_BEGIN: u8 = 0x04;
const K_SNAPSHOT_CHUNK: u8 = 0x05;
const K_PARTIAL_UPDATE: u8 = 0x06;
const K_KEEP_ALIVE: u8 = 0x07;
const K_QUALITY: u8 = 0x08;
const K_ERROR: u8 = 0x09;
const K_BYE: u8 = 0x0A;
const K_SNAPSHOT_COMMIT: u8 = 0x0B;
const K_CURSOR_MOVE: u8 = 0x0C;
const K_CURSOR_HIDE: u8 = 0x0D;
const K_MOTION_PREVIEW: u8 = 0x0E;
const K_REDIRECT: u8 = 0x0F;

/// A monotonic, strictly increasing revision of the authoritative surface.
/// `0` means "nothing has been committed yet".
pub type Rev = u64;

/// Identifies one geometry/colour epoch. Any change that makes previously
/// sent rectangles meaningless (a resize, a rotation, a display switch)
/// increments it, and the viewer discards everything from older epochs.
pub type Epoch = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// Viewer -> sharer, first message on every stream. The token is both
    /// the authorization credential and (see `network::config`) the
    /// certificate fingerprint the viewer pinned.
    ///
    /// `resume` is what the viewer last applied before a reconnect: the
    /// sharer replays the ring from there when it still covers it, else
    /// snapshots. `None` (a fresh join) always snapshots. Kept inside
    /// `Hello` so no new handshake round trip is needed and old builds
    /// fail at the version gate instead of mis-parsing.
    Hello {
        token: String,
        resume: Option<(Epoch, Rev)>,
    },
    /// Viewer -> sharer: "I hold nothing usable, send me a fresh snapshot."
    RequestKeyframe,
    /// Viewer -> sharer: an ephemeral X25519 public key and proof of
    /// holding the token. Everything after this is sealed, so the relay
    /// sees only sizes and timing.
    E2eOffer {
        public: [u8; 32],
        proof: [u8; 32],
    },
    /// Sharer -> viewer: the same, in reply.
    E2eReply {
        public: [u8; 32],
        proof: [u8; 32],
    },
    /// Viewer -> sharer: cumulative "I have fully applied up to this rev".
    Ack {
        rev: Rev,
    },
    /// Start of a lossless, **bounded** snapshot. The snapshot is split
    /// into `chunks` equal-ish pieces of `total_len` bytes so no single
    /// message can exceed the frame budget, whatever the display size or
    /// the content's entropy. A viewer is pixel-exact after committing a
    /// snapshot plus every later update.
    SnapshotBegin {
        rev: Rev,
        /// Microseconds on the sharer's capture clock. Zero when the
        /// sharer is not sending audio, because then nothing is
        /// synchronised against it.
        pts_us: u64,
        epoch: Epoch,
        width: u32,
        height: u32,
        format: SnapshotFormat,
        total_len: u32,
        chunks: u32,
    },
    /// One piece of the snapshot announced by `SnapshotBegin`. Every part
    /// of a snapshot carries the same revision, so a viewer that already
    /// holds it discards the whole sequence rather than half of it.
    SnapshotChunk {
        rev: Rev,
        index: u32,
        data: Vec<u8>,
    },
    /// The snapshot is complete at this revision.
    SnapshotCommit {
        rev: Rev,
        /// As on the snapshot that produced it.
        pts_us: u64,
        epoch: Epoch,
    },
    /// Exact replacements against the receiver's current buffer.
    PartialUpdate {
        rev: Rev,
        pts_us: u64,
        epoch: Epoch,
        ops: Vec<WireOp>,
    },
    /// Nothing changed. Carries the newest revision produced so a viewer can
    /// tell "idle" from "lost".
    KeepAlive {
        rev: Rev,
    },
    QualityConfig(QualityConfig),
    /// Sharer -> viewer: the pointer moved. Presentation-only: the
    /// compositor never sees it, so it cannot corrupt the authoritative
    /// surface. Coordinates are in shared-surface pixels; a viewer whose
    /// geometry differs scales them.
    CursorMove {
        x: u32,
        y: u32,
    },
    /// Sharer -> viewer: the pointer left the shared area (or capture
    /// lost it). Hide the overlay until the next `CursorMove`.
    CursorHide,
    /// Sharer -> viewer: cheap interim content while full motion is still
    /// being planned/encoded — currently the changed-area rectangle and
    /// its fraction, so a viewer can show *something* instead of a frozen
    /// frame during a fling. Never authoritative: the exact update or
    /// snapshot that follows still decides the pixels.
    MotionPreview {
        rev: Rev,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        changed_fraction: f32,
    },
    /// Sharer -> viewer: "reconnect at this relay session instead" —
    /// the fanout/migration handoff. The viewer tears this session down
    /// and dials the named relay session with its current resume point.
    Redirect {
        relay: String,
        session: String,
    },
    Error(String),
    Bye,
}

// ---------------------------------------------------------------- encoding

impl Message {
    pub fn encode_into(&self, out: &mut Vec<u8>) {
        out.push(PROTOCOL_VERSION);
        let len_at = out.len();
        out.extend_from_slice(&[0u8; 4]);
        self.encode_body(out);
        let len = (out.len() - len_at - 4) as u32;
        out[len_at..len_at + 4].copy_from_slice(&len.to_le_bytes());
    }

    /// Encode with the size budget applied. The sender calls this once per
    /// update and shares the bytes with every viewer.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(1024);
        out.push(PROTOCOL_VERSION);
        let len_at = out.len();
        out.extend_from_slice(&[0u8; 4]);
        self.encode_body(&mut out);
        let len = (out.len() - len_at - 4) as u64;
        if len > MAX_MESSAGE_SIZE as u64 {
            anyhow::bail!("Message too large: {len} bytes (max_message_size={MAX_MESSAGE_SIZE})");
        }
        let len = len as u32;
        out[len_at..len_at + 4].copy_from_slice(&len.to_le_bytes());
        Ok(out)
    }

    /// Encoded size without allocating, so the planner can choose between a
    /// patch set and a snapshot before paying for either.
    pub fn encoded_len(&self) -> Result<usize> {
        let mut buf = Vec::new();
        self.encode_body(&mut buf);
        let len = buf.len();
        drop(buf);
        if len as u64 > MAX_MESSAGE_SIZE as u64 {
            anyhow::bail!("Message too large: {len} bytes (max_message_size={MAX_MESSAGE_SIZE})");
        }
        Ok(len + 5)
    }

    fn encode_body(&self, out: &mut Vec<u8>) {
        match self {
            Message::Hello { token, resume } => {
                out.push(K_HELLO);
                let bytes = token.as_bytes();
                out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
                out.extend_from_slice(bytes);
                // One flag byte, then the pair only when resuming. Old
                // builds never emit this and fail the version gate first.
                match resume {
                    Some((epoch, rev)) => {
                        out.push(1);
                        out.extend_from_slice(&epoch.to_le_bytes());
                        out.extend_from_slice(&rev.to_le_bytes());
                    }
                    None => out.push(0),
                }
            }
            Message::RequestKeyframe => out.push(K_REQUEST_KEYFRAME),
            Message::E2eOffer { public, proof } => {
                out.push(K_E2E_OFFER);
                out.extend_from_slice(public);
                out.extend_from_slice(proof);
            }
            Message::E2eReply { public, proof } => {
                out.push(K_E2E_REPLY);
                out.extend_from_slice(public);
                out.extend_from_slice(proof);
            }
            Message::Ack { rev } => {
                out.push(K_ACK);
                out.extend_from_slice(&rev.to_le_bytes());
            }
            Message::SnapshotBegin {
                rev,
                pts_us,
                epoch,
                width,
                height,
                format,
                total_len,
                chunks,
            } => {
                out.push(K_SNAPSHOT_BEGIN);
                out.extend_from_slice(&rev.to_le_bytes());
                out.extend_from_slice(&pts_us.to_le_bytes());
                out.extend_from_slice(&epoch.to_le_bytes());
                out.extend_from_slice(&width.to_le_bytes());
                out.extend_from_slice(&height.to_le_bytes());
                out.push(u8::from(*format));
                out.extend_from_slice(&total_len.to_le_bytes());
                out.extend_from_slice(&chunks.to_le_bytes());
            }
            Message::SnapshotChunk { rev, index, data } => {
                out.push(K_SNAPSHOT_CHUNK);
                out.extend_from_slice(&rev.to_le_bytes());
                out.extend_from_slice(&index.to_le_bytes());
                out.extend_from_slice(&(data.len() as u32).to_le_bytes());
                out.extend_from_slice(data);
            }
            Message::SnapshotCommit { rev, pts_us, epoch } => {
                out.push(K_SNAPSHOT_COMMIT);
                out.extend_from_slice(&rev.to_le_bytes());
                out.extend_from_slice(&pts_us.to_le_bytes());
                out.extend_from_slice(&epoch.to_le_bytes());
            }
            Message::PartialUpdate {
                rev,
                pts_us,
                epoch,
                ops,
            } => {
                out.push(K_PARTIAL_UPDATE);
                out.extend_from_slice(&rev.to_le_bytes());
                out.extend_from_slice(&pts_us.to_le_bytes());
                out.extend_from_slice(&epoch.to_le_bytes());
                out.extend_from_slice(&(ops.len() as u32).to_le_bytes());
                for op in ops {
                    op.encode_into(out);
                }
            }
            Message::KeepAlive { rev } => {
                out.push(K_KEEP_ALIVE);
                out.extend_from_slice(&rev.to_le_bytes());
            }
            Message::QualityConfig(cfg) => {
                out.push(K_QUALITY);
                out.extend_from_slice(&cfg.target_fps.to_le_bytes());
                out.extend_from_slice(&cfg.max_fps.to_le_bytes());
                out.extend_from_slice(&cfg.quality.to_le_bytes());
            }
            Message::Error(text) => {
                out.push(K_ERROR);
                let bytes = text.as_bytes();
                let n = bytes.len().min(MAX_ERROR_LEN as usize);
                out.extend_from_slice(&(n as u16).to_le_bytes());
                out.extend_from_slice(&bytes[..n]);
            }
            Message::Bye => out.push(K_BYE),
            Message::CursorMove { x, y } => {
                out.push(K_CURSOR_MOVE);
                out.extend_from_slice(&x.to_le_bytes());
                out.extend_from_slice(&y.to_le_bytes());
            }
            Message::CursorHide => out.push(K_CURSOR_HIDE),
            Message::MotionPreview {
                rev,
                x,
                y,
                width,
                height,
                changed_fraction,
            } => {
                out.push(K_MOTION_PREVIEW);
                out.extend_from_slice(&rev.to_le_bytes());
                out.extend_from_slice(&x.to_le_bytes());
                out.extend_from_slice(&y.to_le_bytes());
                out.extend_from_slice(&width.to_le_bytes());
                out.extend_from_slice(&height.to_le_bytes());
                out.extend_from_slice(&changed_fraction.to_le_bytes());
            }
            Message::Redirect { relay, session } => {
                out.push(K_REDIRECT);
                let rb = relay.as_bytes();
                let sb = session.as_bytes();
                out.extend_from_slice(&(rb.len() as u16).to_le_bytes());
                out.extend_from_slice(rb);
                out.extend_from_slice(&(sb.len() as u16).to_le_bytes());
                out.extend_from_slice(sb);
            }
        }
    }

    /// Decode a full framed message (version + length + body).
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            anyhow::bail!("Message too short: {} bytes (need >= 5)", bytes.len());
        }
        let version = bytes[0];
        if version != PROTOCOL_VERSION {
            anyhow::bail!("Protocol version mismatch: expected {PROTOCOL_VERSION}, got {version}");
        }
        let len = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
        if len > MAX_MESSAGE_SIZE {
            anyhow::bail!(
                "Framed message too large: {len} bytes (max_message_size={MAX_MESSAGE_SIZE})"
            );
        }
        let payload = &bytes[5..];
        if payload.len() != len as usize {
            anyhow::bail!(
                "Message length mismatch: header says {len}, got {}",
                payload.len()
            );
        }
        let mut cur = Cursor::new(payload);
        let msg = cur.message()?;
        if !cur.is_empty() {
            anyhow::bail!("{} trailing bytes after message body", cur.remaining());
        }
        Ok(msg)
    }

    /// The revision this message carries, if it carries one.
    pub fn rev(&self) -> Option<Rev> {
        match self {
            Message::SnapshotCommit { rev, .. }
            | Message::PartialUpdate { rev, .. }
            | Message::KeepAlive { rev }
            | Message::Ack { rev } => Some(*rev),
            _ => None,
        }
    }

    pub async fn write_framed<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let encoded = self.encode()?;
        // The envelope length is implied by the write on a stream, but the
        // 4-byte prefix is what the relay caches and forwards verbatim.
        writer
            .write_all(&(encoded.len() as u32).to_le_bytes())
            .await?;
        writer.write_all(&encoded).await?;
        writer.flush().await?;
        Ok(())
    }

    /// Read one length-prefixed envelope, without decoding it. A sealed
    /// session reads the ciphertext through here.
    pub async fn read_envelope<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = read_len_prefix(&len_buf)?;
        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload).await?;
        Ok(payload)
    }

    /// Read one length-prefixed message. The cap is applied to the prefix
    /// *before* the buffer exists, so a hostile peer cannot make us
    /// allocate a 4 GiB `Vec` by sending the bytes `FF FF FF FF`.
    pub async fn read_framed<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        Message::decode(&Message::read_envelope(reader).await?)
    }
}

/// Validate a 4-byte length prefix against the envelope budget and return it
/// as a `usize`. Shared by the relay and every other framing entry point.
/// The revision a message carries, without decoding the rest.
///
/// Every kind that has one leads with it, so this is a fixed read.
pub fn peek_rev(envelope: &[u8]) -> Option<Rev> {
    if envelope.len() < 5 || envelope[0] != PROTOCOL_VERSION {
        return None;
    }
    let body = &envelope[5..];
    match *body.first()? {
        // SnapshotBegin, SnapshotChunk, PartialUpdate, KeepAlive,
        // SnapshotCommit and MotionPreview all start with a u64 revision.
        K_SNAPSHOT_BEGIN | K_SNAPSHOT_CHUNK | K_PARTIAL_UPDATE | K_KEEP_ALIVE
        | K_SNAPSHOT_COMMIT | K_MOTION_PREVIEW => {
            if body.len() < 9 {
                return None;
            }
            Some(u64::from_le_bytes(body[1..9].try_into().ok()?))
        }
        _ => None,
    }
}

pub fn read_len_prefix(len_buf: &[u8; 4]) -> Result<usize> {
    read_len_prefix_with_slack(len_buf, 0)
}

/// The same cap with room for `slack` extra bytes inside the outer length.
/// The relay's host leg carries a 4-byte peer id on top of the largest
/// envelope a viewer may send, so it reads with slack 4. Every other
/// caller stays at slack 0.
pub fn read_len_prefix_with_slack(len_buf: &[u8; 4], slack: usize) -> Result<usize> {
    let len = u32::from_le_bytes(*len_buf);
    let cap = MAX_MESSAGE_SIZE + 5 + slack as u32;
    if len > cap {
        anyhow::bail!(
            "Framed message too large: {len} bytes (max_message_size={MAX_MESSAGE_SIZE})"
        );
    }
    Ok(len as usize)
}

// ---------------------------------------------------------------- decoding

pub struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// A reader over an encoded body. The web server uses it to parse a
    /// browser handshake, which is a message like any other.
    pub fn new(body: &'a [u8]) -> Self {
        Self { buf: body, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.remaining() {
            anyhow::bail!(
                "Truncated message: need {n} more bytes, have {}",
                self.remaining()
            );
        }
        let out = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    /// A length that is about to authorise an allocation.
    fn alloc_len(&mut self, max: u64, what: &str) -> Result<usize> {
        let n = self.u32()? as u64;
        if n > max {
            anyhow::bail!("{what} too large: {n} bytes (max={max})");
        }
        Ok(n as usize)
    }

    fn message(&mut self) -> Result<Message> {
        let kind = self.u8()?;
        let msg = match kind {
            K_HELLO => {
                let n = self.u16()? as usize;
                if n > MAX_TOKEN_LEN as usize {
                    anyhow::bail!("token too large: {n} bytes (max={MAX_TOKEN_LEN})");
                }
                let token = std::str::from_utf8(self.take(n)?)
                    .context("token is not valid UTF-8")?
                    .to_string();
                // Matches the encoder: one flag byte, then the pair.
                let resume = match self.u8()? {
                    0 => None,
                    1 => Some((self.u32()?, self.u64()?)),
                    f => anyhow::bail!("hello resume flag {f} is not 0 or 1"),
                };
                Message::Hello { token, resume }
            }
            K_REQUEST_KEYFRAME => Message::RequestKeyframe,
            K_E2E_OFFER => {
                let mut public = [0u8; 32];
                let mut proof = [0u8; 32];
                public.copy_from_slice(self.take(32)?);
                proof.copy_from_slice(self.take(32)?);
                Message::E2eOffer { public, proof }
            }
            K_E2E_REPLY => {
                let mut public = [0u8; 32];
                let mut proof = [0u8; 32];
                public.copy_from_slice(self.take(32)?);
                proof.copy_from_slice(self.take(32)?);
                Message::E2eReply { public, proof }
            }
            K_ACK => Message::Ack { rev: self.u64()? },
            K_SNAPSHOT_BEGIN => {
                let rev = self.u64()?;
                let pts_us = self.u64()?;
                let epoch = self.u32()?;
                let width = self.u32()?;
                let height = self.u32()?;
                // Budget from the declared geometry before anything is
                // allocated: a 65535x65535 header costs nothing.
                crate::pcc::types::rgb_len(width, height).map_err(|e| {
                    anyhow::anyhow!("invalid snapshot geometry {width}x{height}: {e}")
                })?;
                let format = SnapshotFormat::from_u8(self.u8()?)?;
                let total_len = self.alloc_len(MAX_SNAPSHOT_BYTES as u64, "snapshot")? as u32;
                let chunks = self.u32()?;
                if chunks == 0 || chunks > MAX_SNAPSHOT_CHUNKS {
                    anyhow::bail!(
                        "snapshot chunk count {chunks} outside 1..={MAX_SNAPSHOT_CHUNKS}"
                    );
                }
                Message::SnapshotBegin {
                    rev,
                    pts_us,
                    epoch,
                    width,
                    height,
                    format,
                    total_len,
                    chunks,
                }
            }
            K_SNAPSHOT_CHUNK => {
                let rev = self.u64()?;
                let index = self.u32()?;
                let n = self.alloc_len(SNAPSHOT_CHUNK_BYTES as u64, "snapshot chunk")?;
                Message::SnapshotChunk {
                    rev,
                    index,
                    data: self.take(n)?.to_vec(),
                }
            }
            K_PARTIAL_UPDATE => {
                let rev = self.u64()?;
                let pts_us = self.u64()?;
                let epoch = self.u32()?;
                let count = self.u32()?;
                if count > MAX_OPS_PER_UPDATE {
                    anyhow::bail!(
                        "too many ops in one update: {count} (max_ops_per_update={MAX_OPS_PER_UPDATE})"
                    );
                }
                let mut ops = Vec::with_capacity((count as usize).min(1024));
                for _ in 0..count {
                    ops.push(self.op()?);
                }
                Message::PartialUpdate {
                    rev,
                    pts_us,
                    epoch,
                    ops,
                }
            }
            K_KEEP_ALIVE => Message::KeepAlive { rev: self.u64()? },
            K_QUALITY => Message::QualityConfig(QualityConfig {
                target_fps: self.u32()?,
                max_fps: self.u32()?,
                quality: self.f32()?,
            }),
            K_ERROR => {
                // The length is written as u16, so it is read as u16.
                let n = self.u16()?;
                if n > MAX_ERROR_LEN {
                    anyhow::bail!("Error message too large: {n} bytes (max={MAX_ERROR_LEN})");
                }
                let text = std::str::from_utf8(self.take(n as usize)?)
                    .context("Error message is not valid UTF-8")?
                    .to_string();
                Message::Error(text)
            }
            K_SNAPSHOT_COMMIT => Message::SnapshotCommit {
                rev: self.u64()?,
                pts_us: self.u64()?,
                epoch: self.u32()?,
            },
            K_BYE => Message::Bye,
            K_CURSOR_MOVE => Message::CursorMove {
                x: self.u32()?,
                y: self.u32()?,
            },
            K_CURSOR_HIDE => Message::CursorHide,
            K_MOTION_PREVIEW => {
                let rev = self.u64()?;
                let x = self.u32()?;
                let y = self.u32()?;
                let width = self.u32()?;
                let height = self.u32()?;
                let changed_fraction = self.f32()?;
                if !(0.0..=1.0).contains(&changed_fraction) {
                    anyhow::bail!("motion preview fraction {changed_fraction} outside 0..=1");
                }
                Message::MotionPreview {
                    rev,
                    x,
                    y,
                    width,
                    height,
                    changed_fraction,
                }
            }
            K_REDIRECT => {
                let rn = self.u16()? as usize;
                if rn == 0 || rn > 256 {
                    anyhow::bail!("redirect relay address length {rn} outside 1..=256");
                }
                let relay = std::str::from_utf8(self.take(rn)?)
                    .context("redirect relay is not valid UTF-8")?
                    .to_string();
                let sn = self.u16()? as usize;
                if sn == 0 || sn > 64 {
                    anyhow::bail!("redirect session code length {sn} outside 1..=64");
                }
                let session = std::str::from_utf8(self.take(sn)?)
                    .context("redirect session is not valid UTF-8")?
                    .to_string();
                Message::Redirect { relay, session }
            }
            other => anyhow::bail!("Unknown message kind: 0x{other:02x}"),
        };
        Ok(msg)
    }

    fn op(&mut self) -> Result<WireOp> {
        let x = self.u32()?;
        let y = self.u32()?;
        let width = self.u32()?;
        let height = self.u32()?;
        let kind = self.u8()?;
        let op = match kind {
            crate::network::OP_RECT => {
                let expected = (width as u64) * (height as u64) * 3;
                if expected > crate::pcc::types::MAX_FRAME_BYTES as u64 {
                    anyhow::bail!(
                        "rect too large: {width}x{height} = {expected} raw bytes \
                         (max_frame_bytes={})",
                        crate::pcc::types::MAX_FRAME_BYTES
                    );
                }
                // LZ4's size-prepended frame can only be as large as the
                // input plus its own header; refuse anything larger so a
                // compression bomb cannot be handed to the decompressor.
                let n = self.alloc_len(expected + 16, "rect payload")?;
                let compressed = self.take(n)?.to_vec();
                WireOp::Rect {
                    x,
                    y,
                    width,
                    height,
                    compressed,
                }
            }
            crate::network::OP_FILL => {
                let n = self.alloc_len(3, "fill colour")?;
                if n != 3 {
                    anyhow::bail!("fill colour must be 3 bytes, got {n}");
                }
                let b = self.take(3)?;
                WireOp::Fill {
                    x,
                    y,
                    width,
                    height,
                    color: [b[0], b[1], b[2]],
                }
            }
            crate::network::OP_COPY => {
                let src_x = self.u32()?;
                let src_y = self.u32()?;
                WireOp::Copy {
                    x,
                    y,
                    width,
                    height,
                    src_x,
                    src_y,
                }
            }
            other => anyhow::bail!("Unknown op kind: 0x{other:02x}"),
        };
        Ok(op)
    }
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(msg: &Message) -> Message {
        let bytes = msg.encode().unwrap();
        Message::decode(&bytes).unwrap()
    }

    #[test]
    fn every_message_roundtrips() {
        let msgs = vec![
            Message::Hello {
                token: "ABC123".into(),
                resume: None,
            },
            Message::Hello {
                token: "ABC123".into(),
                resume: Some((2, 9301)),
            },
            Message::RequestKeyframe,
            Message::Ack { rev: 42 },
            Message::SnapshotBegin {
                rev: 7,
                pts_us: 0,
                epoch: 1,
                width: 2,
                height: 2,
                format: SnapshotFormat::Raw,
                total_len: 12,
                chunks: 1,
            },
            Message::SnapshotChunk {
                rev: 7,
                index: 0,
                data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            },
            Message::SnapshotCommit {
                rev: 7,
                pts_us: 0,
                epoch: 1,
            },
            Message::PartialUpdate {
                rev: 8,
                pts_us: 0,
                epoch: 1,
                ops: vec![
                    WireOp::Rect {
                        x: 1,
                        y: 1,
                        width: 1,
                        height: 1,
                        compressed: vec![7, 7, 7],
                    },
                    WireOp::Fill {
                        x: 0,
                        y: 0,
                        width: 4,
                        height: 4,
                        color: [9, 8, 7],
                    },
                    WireOp::Copy {
                        x: 2,
                        y: 2,
                        width: 4,
                        height: 4,
                        src_x: 40,
                        src_y: 40,
                    },
                ],
            },
            Message::KeepAlive { rev: 99 },
            Message::CursorMove { x: 10, y: 20 },
            Message::CursorHide,
            Message::MotionPreview {
                rev: 8,
                x: 0,
                y: 0,
                width: 64,
                height: 48,
                changed_fraction: 0.5,
            },
            Message::Redirect {
                relay: "127.0.0.1:5900".into(),
                session: "ABC123".into(),
            },
            Message::QualityConfig(QualityConfig::default()),
            Message::Error("boom".into()),
            Message::Bye,
        ];
        for msg in msgs {
            assert_eq!(roundtrip(&msg), msg, "roundtrip failed for {msg:?}");
        }
    }

    #[test]
    fn hostile_preview_fraction_is_rejected() {
        let msg = Message::MotionPreview {
            rev: 1,
            x: 0,
            y: 0,
            width: 8,
            height: 8,
            changed_fraction: 2.0,
        };
        let err = Message::decode(&msg.encode().unwrap())
            .unwrap_err()
            .to_string();
        assert!(err.contains("fraction"), "unhelpful: {err}");
    }

    #[test]
    fn version_mismatch_is_rejected() {
        let mut bytes = Message::Bye.encode().unwrap();
        bytes[0] = 2;
        let err = Message::decode(&bytes).unwrap_err().to_string();
        assert!(err.contains("version mismatch"), "unhelpful: {err}");
    }

    #[test]
    fn hostile_length_prefix_never_allocates() {
        // 0xFFFF_FFFF would be a 4 GiB allocation if it were trusted.
        let err = read_len_prefix(&[0xFF, 0xFF, 0xFF, 0xFF])
            .unwrap_err()
            .to_string();
        assert!(err.contains("max_message_size"), "unhelpful: {err}");
    }

    #[test]
    fn slack_covers_only_the_peer_id_overhead() {
        // The largest envelope plus the 4-byte peer id parses with slack 4
        // and is rejected with slack 0; the plain envelope parses either way.
        let at_cap = (MAX_MESSAGE_SIZE + 5).to_le_bytes();
        assert!(read_len_prefix(&at_cap).is_ok());
        let with_id = (MAX_MESSAGE_SIZE + 9).to_le_bytes();
        assert!(read_len_prefix(&with_id).is_err());
        assert!(read_len_prefix_with_slack(&with_id, 4).is_ok());
        assert!(read_len_prefix_with_slack(&at_cap, 4).is_ok());
    }

    #[test]
    fn truncated_body_is_rejected() {
        let bytes = Message::Ack { rev: 5 }.encode().unwrap();
        let err = Message::decode(&bytes[..bytes.len() - 3])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("Truncated") || err.contains("mismatch"),
            "got: {err}"
        );
    }

    #[test]
    fn unknown_kind_is_rejected() {
        let mut bytes = Message::Bye.encode().unwrap();
        bytes[5] = 0xEE;
        assert!(Message::decode(&bytes).is_err());
    }

    #[test]
    fn snapshot_begin_with_impossible_geometry_is_rejected() {
        let msg = Message::SnapshotBegin {
            rev: 1,
            pts_us: 0,
            epoch: 0,
            width: 60_000,
            height: 60_000,
            format: SnapshotFormat::Png,
            total_len: 10,
            chunks: 1,
        };
        let bytes = msg.encode().unwrap();
        let err = Message::decode(&bytes).unwrap_err().to_string();
        assert!(err.contains("max_frame_bytes"), "unhelpful: {err}");
    }

    #[test]
    fn snapshot_begin_with_zero_chunks_is_rejected() {
        let msg = Message::SnapshotBegin {
            rev: 1,
            pts_us: 0,
            epoch: 0,
            width: 2,
            height: 2,
            format: SnapshotFormat::Raw,
            total_len: 12,
            chunks: 0,
        };
        let bytes = msg.encode().unwrap();
        assert!(Message::decode(&bytes).is_err());
    }

    #[test]
    fn an_oversized_chunk_is_rejected_before_allocation() {
        let mut payload = vec![K_SNAPSHOT_CHUNK];
        payload.extend_from_slice(&1u64.to_le_bytes());
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&(SNAPSHOT_CHUNK_BYTES as u32 + 1).to_le_bytes());
        let mut bytes = vec![PROTOCOL_VERSION];
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        let err = Message::decode(&bytes).unwrap_err().to_string();
        assert!(err.contains("snapshot chunk"), "unhelpful: {err}");
    }

    #[test]
    fn oversized_op_count_is_rejected() {
        let mut payload = vec![K_PARTIAL_UPDATE];
        payload.extend_from_slice(&1u64.to_le_bytes()); // rev
        payload.extend_from_slice(&0u64.to_le_bytes()); // pts
        payload.extend_from_slice(&0u32.to_le_bytes()); // epoch
        payload.extend_from_slice(&(MAX_OPS_PER_UPDATE + 1).to_le_bytes());
        let mut bytes = vec![PROTOCOL_VERSION];
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        let err = Message::decode(&bytes).unwrap_err().to_string();
        assert!(err.contains("max_ops_per_update"), "unhelpful: {err}");
    }
}
