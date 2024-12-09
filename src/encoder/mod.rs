use anyhow::{Context, Result};
use crate::pcc::QualityConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use jpeg_encoder::{Encoder, ColorType, EncodingConfig};

pub struct FrameEncoder {
    config: QualityConfig,
    width: u32,
    height: u32,
    encoder_config: EncodingConfig,
}

impl FrameEncoder {
    pub fn new(width: u32, height: u32, config: QualityConfig) -> Result<Self> {
        let mut encoder_config = EncodingConfig::new();
        encoder_config.quality = (config.quality * 100.0) as u8;
        encoder_config.optimize_huffman_tables = true;
        
        Ok(Self {
            config,
            width,
            height,
            encoder_config,
        })
    }
    
    // Encode a frame using optimized JPEG compression
    pub async fn encode_frame(&self, frame: &[u8]) -> Result<Vec<u8>> {
        let encoder = Encoder::new_with_config(self.encoder_config.clone());
        
        // Log compression start for performance tracking
        let start = std::time::Instant::now();
        
        let encoded = encoder.encode(
            frame,
            self.width as u16,
            self.height as u16,
            ColorType::Rgb,
        )?;
        
        // Log compression stats
        let duration = start.elapsed();
        let compression_ratio = frame.len() as f32 / encoded.len() as f32;
        debug!(
            "Frame encoded: {}x{} in {:?}, ratio: {:.2}:1",
            self.width, self.height, duration, compression_ratio
        );
        
        Ok(encoded)
    }
    
    // Reconfigure encoder with new settings
    pub async fn reconfigure(&mut self, config: QualityConfig) -> Result<()> {
        self.config = config;
        self.encoder_config.quality = (config.quality * 100.0) as u8;
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
