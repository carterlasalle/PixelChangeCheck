use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use crate::pcc::QualityConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub struct FrameEncoder {
    context: Arc<Mutex<ffmpeg::codec::Context>>,
    codec: ffmpeg::codec::encoder::Video,
    config: QualityConfig,
}

impl FrameEncoder {
    pub fn new(width: u32, height: u32, config: QualityConfig) -> Result<Self> {
        // Find hardware-accelerated encoder if available
        let codec = Self::find_best_encoder()?;
        let mut context = codec.create()?;
        
        // Configure encoder
        context.set_width(width);
        context.set_height(height);
        context.set_time_base((1, config.target_fps as i32));
        context.set_pixel_format(ffmpeg::format::Pixel::RGB24);
        
        // Set quality parameters
        let mut opts = ffmpeg::Dictionary::new();
        opts.set("preset", "ultrafast"); // Prioritize speed
        opts.set("tune", "zerolatency"); // Minimize latency
        opts.set("crf", &(30.0 * (1.0 - config.quality)).to_string()); // Quality-based encoding
        
        // Open encoder with options
        let encoder = context.encoder().video()?;
        encoder.open_with(opts)?;
        
        Ok(Self {
            context: Arc::new(Mutex::new(context)),
            codec: encoder,
            config,
        })
    }
    
    // Find the best available encoder
    fn find_best_encoder() -> Result<ffmpeg::codec::Codec> {
        let encoders = [
            // Hardware accelerated encoders
            "h264_videotoolbox",  // macOS
            "h264_nvenc",         // NVIDIA
            "h264_amf",           // AMD
            "h264_qsv",           // Intel QuickSync
            "h264_vaapi",         // Linux VA-API
            // Software fallback
            "libx264",
        ];
        
        for encoder in encoders.iter() {
            if let Ok(codec) = ffmpeg::codec::encoder::find_by_name(encoder) {
                info!("Using encoder: {}", encoder);
                return Ok(codec);
            }
        }
        
        // Fallback to first available H.264 encoder
        ffmpeg::codec::encoder::find(ffmpeg::codec::Id::H264)
            .context("No H.264 encoder found")
    }
    
    // Encode a frame with hardware acceleration if available
    pub async fn encode_frame(&self, frame: &ffmpeg::frame::Video) -> Result<Vec<u8>> {
        let mut context = self.context.lock().await;
        let mut encoded = Vec::new();
        
        // Send frame to encoder
        self.codec.send_frame(frame)?;
        
        // Receive encoded packets
        let mut packet = ffmpeg::packet::Packet::empty();
        while self.codec.receive_packet(&mut packet).is_ok() {
            encoded.extend_from_slice(packet.data());
        }
        
        Ok(encoded)
    }
    
    // Reconfigure encoder with new settings
    pub async fn reconfigure(&mut self, config: QualityConfig) -> Result<()> {
        let mut context = self.context.lock().await;
        
        // Update encoder parameters
        let mut opts = ffmpeg::Dictionary::new();
        opts.set("crf", &(30.0 * (1.0 - config.quality)).to_string());
        opts.set("maxrate", &format!("{}k", config.target_fps * 50)); // Rough bitrate estimate
        
        // Reopen encoder with new options
        self.codec = context.encoder().video()?.open_with(opts)?;
        self.config = config;
        
        Ok(())
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
