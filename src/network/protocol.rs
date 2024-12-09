use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// Protocol version for compatibility checking
const PROTOCOL_VERSION: u8 = 1;

// Maximum message sizes
const MAX_FRAME_SIZE: usize = 1024 * 1024 * 4; // 4MB
const MAX_MESSAGE_SIZE: usize = 1024 * 64; // 64KB

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    // Frame-related messages
    FrameData {
        frame_id: u64,
        timestamp: SystemTime,
        data: Bytes,
    },
    FrameAck {
        frame_id: u64,
    },
    
    // Control messages
    KeepAlive,
    QualityConfig(crate::pcc::QualityConfig),
    Error(String),
}

impl Message {
    // Serialize message to bytes
    pub fn serialize(&self) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(1024);
        
        // Write protocol version
        buf.put_u8(PROTOCOL_VERSION);
        
        // Serialize message
        let serialized = bincode::serialize(self)?;
        if serialized.len() > MAX_MESSAGE_SIZE {
            anyhow::bail!("Message too large: {} bytes", serialized.len());
        }
        
        // Write message length and data
        buf.put_u32_le(serialized.len() as u32);
        buf.extend_from_slice(&serialized);
        
        Ok(buf.freeze())
    }
    
    // Deserialize message from bytes
    pub fn deserialize(mut bytes: Bytes) -> Result<Self> {
        if bytes.len() < 5 {
            anyhow::bail!("Message too short");
        }
        
        // Read and verify protocol version
        let version = bytes.get_u8();
        if version != PROTOCOL_VERSION {
            anyhow::bail!("Protocol version mismatch: expected {}, got {}", PROTOCOL_VERSION, version);
        }
        
        // Read message length
        let len = bytes.get_u32_le() as usize;
        if len > MAX_MESSAGE_SIZE {
            anyhow::bail!("Message too large: {} bytes", len);
        }
        
        // Deserialize message
        let message: Self = bincode::deserialize(&bytes[..len])?;
        Ok(message)
    }
}

// Frame-specific protocol handling
pub struct FrameProtocol;

impl FrameProtocol {
    // Encode a frame for transmission
    pub fn encode_frame(frame: &crate::pcc::Frame) -> Result<Vec<Bytes>> {
        let mut chunks = Vec::new();
        let data = Bytes::from(frame.data.clone());
        
        // Split large frames into chunks
        for chunk in data.chunks(MAX_FRAME_SIZE) {
            let message = Message::FrameData {
                frame_id: frame.id,
                timestamp: frame.timestamp,
                data: Bytes::copy_from_slice(chunk),
            };
            
            chunks.push(message.serialize()?);
        }
        
        Ok(chunks)
    }
    
    // Decode received frame data
    pub fn decode_frame(messages: Vec<Message>) -> Result<crate::pcc::Frame> {
        let mut frame_data = BytesMut::new();
        let mut frame_id = None;
        let mut timestamp = None;
        
        for message in messages {
            if let Message::FrameData { frame_id: id, timestamp: ts, data } = message {
                if frame_id.is_none() {
                    frame_id = Some(id);
                    timestamp = Some(ts);
                }
                frame_data.extend_from_slice(&data);
            }
        }
        
        if let (Some(id), Some(ts)) = (frame_id, timestamp) {
            Ok(crate::pcc::Frame {
                id,
                timestamp: ts,
                width: 0, // These need to be set by the caller
                height: 0,
                data: frame_data.to_vec(),
            })
        } else {
            anyhow::bail!("Incomplete frame data");
        }
    }
} 