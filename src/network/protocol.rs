use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// Protocol version for compatibility checking
const PROTOCOL_VERSION: u8 = 2;

// Maximum message size (a full keyframe JPEG at high resolution comfortably fits under this)
const MAX_MESSAGE_SIZE: u32 = 1024 * 1024 * 16; // 16MB

/// A single changed region, ready to send over the wire (already lz4-compressed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireChange {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// lz4-compressed RGB bytes for this region
    pub compressed_data: Vec<u8>,
}

/// Messages exchanged between a sharer and a viewer (directly over QUIC, or
/// relayed over TCP). Every message is length-prefixed on the wire, so the
/// transport only needs to hand us whole byte buffers.
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// A full keyframe (JPEG-encoded). Sent for the first frame of a session
    /// and periodically thereafter so late-joining viewers can catch up.
    FullFrame {
        frame_id: u64,
        width: u32,
        height: u32,
        jpeg_data: Vec<u8>,
    },
    /// A partial update: only the regions of the frame that changed since
    /// the last frame, per the PCC detector.
    PartialUpdate {
        frame_id: u64,
        changes: Vec<WireChange>,
    },
    /// Sent instead of frame data when nothing has changed, so the viewer
    /// (and any relay/NAT in between) knows the connection is still alive.
    KeepAlive,
    /// Renegotiate capture/encode quality (e.g. after the sharer detects
    /// the network can't keep up).
    QualityConfig(crate::pcc::QualityConfig),
    /// Human-readable error, sent right before a connection is closed.
    Error(String),
    /// Graceful session end.
    Bye,
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(1024);
        buf.push(PROTOCOL_VERSION);

        let serialized = bincode::serialize(self)?;
        if serialized.len() as u64 > MAX_MESSAGE_SIZE as u64 {
            anyhow::bail!("Message too large: {} bytes", serialized.len());
        }

        buf.extend_from_slice(&(serialized.len() as u32).to_le_bytes());
        buf.extend_from_slice(&serialized);
        Ok(buf)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            anyhow::bail!("Message too short");
        }

        let version = bytes[0];
        if version != PROTOCOL_VERSION {
            anyhow::bail!(
                "Protocol version mismatch: expected {}, got {}",
                PROTOCOL_VERSION,
                version
            );
        }

        let len = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as usize;
        let payload = &bytes[5..];
        if payload.len() != len {
            anyhow::bail!(
                "Message length mismatch: header says {}, got {}",
                len,
                payload.len()
            );
        }

        Ok(bincode::deserialize(payload)?)
    }

    /// Write this message to an async byte stream, length-prefixed so the
    /// reader knows exactly how many bytes to read back (QUIC/TCP streams
    /// give no message boundaries on their own).
    pub async fn write_framed<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let encoded = self.serialize()?;
        // 4-byte little-endian frame length, followed by the encoded message
        // (which itself carries the protocol version + its own length header).
        writer.write_all(&(encoded.len() as u32).to_le_bytes()).await?;
        writer.write_all(&encoded).await?;
        writer.flush().await?;
        Ok(())
    }

    /// Read one length-prefixed message from an async byte stream.
    pub async fn read_framed<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        if len as u32 > MAX_MESSAGE_SIZE + 5 {
            anyhow::bail!("Framed message too large: {} bytes", len);
        }

        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload).await?;
        Message::deserialize(&payload)
    }
}
