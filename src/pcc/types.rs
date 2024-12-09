use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: u64,
    pub timestamp: SystemTime,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Frame {
    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelChange {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameUpdate {
    pub frame_id: u64,
    pub timestamp: SystemTime,
    pub changes: Vec<PixelChange>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QualityConfig {
    pub target_fps: u32,
    pub max_fps: u32,
    pub quality: f32,          // 0.0-1.0
    pub compression_level: u8,  // 0-9
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            target_fps: 30,
            max_fps: 60,
            quality: 0.8,
            compression_level: 6,
        }
    }
}

/// Trait for implementing pixel change detection
pub trait PixelChangeDetector {
    /// Detect changes between two frames
    fn detect_changes(&self, previous: &Frame, current: &Frame) -> Result<Vec<PixelChange>>;
    
    /// Configure the detector
    fn configure(&mut self, config: QualityConfig) -> Result<()>;
}

/// Trait for frame capture implementations
pub trait FrameCapture {
    /// Capture a new frame
    fn capture_frame(&self) -> Result<Frame>;
    
    /// Get supported capture configurations
    fn supported_configs(&self) -> Vec<QualityConfig>;
    
    /// Configure the capture
    fn configure(&mut self, config: QualityConfig) -> Result<()>;
} 