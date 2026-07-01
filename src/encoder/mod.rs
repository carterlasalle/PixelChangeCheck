use anyhow::{Context, Result};
use crate::pcc::QualityConfig;
use tracing::debug;
use jpeg_encoder::{Encoder, ColorType};

pub struct FrameEncoder {
    config: QualityConfig,
    width: u32,
    height: u32,
}

/// Encode raw RGB24 pixel data to JPEG. Synchronous helper shared by the
/// async capture pipeline and the plain-threaded web (MJPEG) server.
pub fn encode_jpeg(width: u32, height: u32, quality: f32, frame: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let quality = (quality.clamp(0.0, 1.0) * 100.0) as u8;
    let encoder = Encoder::new(&mut output, quality);
    encoder.encode(frame, width as u16, height as u16, ColorType::Rgb)?;
    Ok(output)
}

/// Decode a JPEG buffer back into raw RGB24 pixel data.
pub fn decode_jpeg(jpeg_data: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
    let img = image::load_from_memory_with_format(jpeg_data, image::ImageFormat::Jpeg)
        .context("Failed to decode JPEG frame")?
        .to_rgb8();
    let (width, height) = (img.width(), img.height());
    Ok((width, height, img.into_raw()))
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
        let start = std::time::Instant::now();

        let output = encode_jpeg(self.width, self.height, self.config.quality, frame)?;

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

    pub fn quality(&self) -> f32 {
        self.config.quality
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
