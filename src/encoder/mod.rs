use anyhow::{Context, Result};
use crate::pcc::QualityConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use jpeg_encoder::{Encoder, ColorType};

pub struct FrameEncoder {
    config: QualityConfig,
    width: u32,
    height: u32,
}

impl FrameEncoder {
    pub fn new(width: u32, height: u32, config: QualityConfig) -> Result<Self> {
        Ok(Self {
            config,
            width,
            height,
        })
    }
    
    // Encode a frame using optimized JPEG compression
    pub async fn encode_frame(&self, frame: &[u8]) -> Result<Vec<u8>> {
        // Log compression start for performance tracking
        let start = std::time::Instant::now();
        
        let mut output = Vec::new();
        let quality = (self.config.quality * 100.0) as u8;
        let mut encoder = Encoder::new(&mut output, quality);
        encoder.encode(
            frame,
            self.width as u16,
            self.height as u16,
            ColorType::Rgb,
        )?;
        
        // Log compression stats
        let duration = start.elapsed();
        let compression_ratio = frame.len() as f32 / output.len() as f32;
        debug!(
            "Frame encoded: {}x{} in {:?}, ratio: {:.2}:1",
            self.width, self.height, duration, compression_ratio
        );
        
        Ok(output)
    }
    
    // Reconfigure encoder with new settings
    pub async fn reconfigure(&mut self, config: QualityConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

// Frame compression utilities for small regions
pub mod compression {
    use super::*;
    use lz4_flex::compress_prepend_size;
    use lz4_flex::decompress_size_prepended;
    
    pub fn compress_frame(frame: &[u8], _quality: f32) -> Result<Vec<u8>> {
        let start = std::time::Instant::now();
        let compressed = compress_prepend_size(frame);
        let duration = start.elapsed();
        
        debug!(
            "Region compressed: {} -> {} bytes in {:?}, ratio: {:.2}:1",
            frame.len(),
            compressed.len(),
            duration,
            frame.len() as f32 / compressed.len() as f32
        );
        
        Ok(compressed)
    }
    
    pub fn decompress_frame(compressed: &[u8]) -> Result<Vec<u8>> {
        Ok(decompress_size_prepended(compressed)?)
    }
}
