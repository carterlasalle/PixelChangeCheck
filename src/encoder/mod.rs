use anyhow::{Context, Result};
use crate::pcc::QualityConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use vpx_encode::{Config, Encoder, Frame, VideoFormat};

pub struct FrameEncoder {
    encoder: Arc<Mutex<Encoder>>,
    config: QualityConfig,
    width: u32,
    height: u32,
}

impl FrameEncoder {
    pub fn new(width: u32, height: u32, config: QualityConfig) -> Result<Self> {
        // Configure VP9 encoder
        let mut vpx_config = Config::new(width, height)
            .context("Failed to create VP9 config")?;
            
        // Set encoding parameters
        vpx_config
            .set_threads(num_cpus::get() as u32)
            .set_timebase(1, config.target_fps as u32)
            .set_target_bitrate((width * height * config.target_fps / 100) as u32) // Rough estimate
            .set_speed(8) // Faster encoding
            .set_video_format(VideoFormat::I420);
            
        // Create encoder
        let encoder = Encoder::new(&vpx_config)
            .context("Failed to create VP9 encoder")?;
            
        Ok(Self {
            encoder: Arc::new(Mutex::new(encoder)),
            config,
            width,
            height,
        })
    }
    
    // Encode a frame
    pub async fn encode_frame(&self, frame: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = self.encoder.lock().await;
        
        // Convert RGB to I420
        let yuv = Self::rgb_to_i420(frame, self.width, self.height)?;
        
        // Create VP9 frame
        let mut vpx_frame = Frame::new(self.width, self.height);
        vpx_frame.data.copy_from_slice(&yuv);
        
        // Encode frame
        let packet = encoder.encode(&vpx_frame, true)?;
        
        Ok(packet.data)
    }
    
    // Reconfigure encoder with new settings
    pub async fn reconfigure(&mut self, config: QualityConfig) -> Result<()> {
        let mut encoder = self.encoder.lock().await;
        
        // Update bitrate based on quality
        let target_bitrate = (self.width * self.height * config.target_fps / 100) as u32;
        encoder.control().set_target_bitrate(target_bitrate)?;
        
        self.config = config;
        Ok(())
    }
    
    // Convert RGB to I420 (YUV420) color space
    fn rgb_to_i420(rgb: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let pixels = width * height;
        let mut yuv = vec![0u8; (pixels * 3 / 2) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let rgb_idx = ((y * width + x) * 3) as usize;
                let y_idx = (y * width + x) as usize;
                let u_idx = (pixels + (y / 2 * width / 2 + x / 2)) as usize;
                let v_idx = (pixels + pixels / 4 + (y / 2 * width / 2 + x / 2)) as usize;
                
                let r = rgb[rgb_idx] as f32;
                let g = rgb[rgb_idx + 1] as f32;
                let b = rgb[rgb_idx + 2] as f32;
                
                // RGB to YUV conversion
                let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                let u_val = (128.0 + (-0.169 * r - 0.331 * g + 0.5 * b)) as u8;
                let v_val = (128.0 + (0.5 * r - 0.419 * g - 0.081 * b)) as u8;
                
                yuv[y_idx] = y_val;
                if x % 2 == 0 && y % 2 == 0 {
                    yuv[u_idx] = u_val;
                    yuv[v_idx] = v_val;
                }
            }
        }
        
        Ok(yuv)
    }
}

// Frame compression utilities
pub mod compression {
    use super::*;
    use flate2::{write::ZlibEncoder, read::ZlibDecoder, Compression};
    use std::io::prelude::*;
    
    pub fn compress_frame(frame: &[u8], quality: f32) -> Result<Vec<u8>> {
        let mut compressed = Vec::new();
        let mut encoder = ZlibEncoder::new(
            &mut compressed,
            Compression::new((9.0 * quality) as u32),
        );
        
        encoder.write_all(frame)?;
        encoder.finish()?;
        
        Ok(compressed)
    }
    
    pub fn decompress_frame(compressed: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        let mut decoder = ZlibDecoder::new(compressed);
        
        decoder.read_to_end(&mut decompressed)?;
        
        Ok(decompressed)
    }
}
