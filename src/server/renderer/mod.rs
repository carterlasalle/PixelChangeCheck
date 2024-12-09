mod buffer;
pub use buffer::FrameBuffer;

use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use tracing::{error, info};

pub struct Renderer {
    buffer: Arc<FrameBuffer>,
    context: Arc<Mutex<ffmpeg::format::context::Output>>,
    stream_index: usize,
    frame_interval: Duration,
}

impl Renderer {
    pub async fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        // Initialize FFmpeg
        ffmpeg::init().context("Failed to initialize FFmpeg")?;
        
        // Create frame buffer
        let buffer = Arc::new(FrameBuffer::new(width, height));
        
        // Create output context
        let mut context = ffmpeg::format::output(&format!("sdl2://PCC Display"))?;
        
        // Create encoder
        let encoder = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)?;
        let stream_index = context.add_stream(encoder)?;
        let mut stream = context.stream_mut(stream_index);
        
        {
            let enc = stream.parameters_mut();
            enc.set_width(width);
            enc.set_height(height);
            enc.set_format(ffmpeg::format::Pixel::RGB24);
            enc.set_frame_rate(Some((fps as i32, 1)));
            enc.set_time_base(Some((1, fps as i32)));
        }
        
        #[cfg(target_os = "macos")]
        {
            let enc = stream.parameters_mut();
            enc.set_codec_id(ffmpeg::codec::Id::H264);
            enc.set_codec_tag(ffmpeg::codec::tag::NONE);
            
            // Set VideoToolbox-specific options
            let mut opts = ffmpeg::Dictionary::new();
            opts.set("allow_sw", "1"); // Allow fallback to software encoding
            opts.set("realtime", "1"); // Enable realtime encoding
            opts.set("profile", "high"); // Use high profile for better quality
            stream.set_parameters_mut().set_options(opts)?;
        }
        
        Ok(Self {
            buffer,
            context: Arc::new(Mutex::new(context)),
            stream_index,
            frame_interval: Duration::from_secs(1) / fps,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Starting renderer");
        
        let mut interval = time::interval(self.frame_interval);
        
        loop {
            interval.tick().await;
            
            // Get next frame from buffer
            if let Some(frame) = self.buffer.next_frame().await? {
                if let Err(e) = self.render_frame(&frame).await {
                    error!("Failed to render frame: {}", e);
                    // Continue rendering next frame instead of breaking
                    continue;
                }
            }
        }
    }
    
    async fn render_frame(&self, frame: &buffer::BufferedFrame) -> Result<()> {
        let mut context = self.context.lock().await;
        let stream = context.stream(self.stream_index);
        
        let mut encoder = stream.parameters().encoder()?;
        let mut packet = ffmpeg::Packet::empty();
        
        // Create video frame
        let mut video_frame = ffmpeg::frame::Video::new(
            ffmpeg::format::Pixel::RGB24,
            frame.width,
            frame.height,
        );
        
        // Copy frame data
        video_frame.data_mut(0).copy_from_slice(&frame.data);
        
        // Encode and write frame
        encoder.send_frame(&video_frame)?;
        while encoder.receive_packet(&mut packet)? {
            context.write(&packet)?;
        }
        
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        let mut context = self.context.lock().await;
        context.flush()?;
        self.buffer.clear().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcc;
    
    #[tokio::test]
    async fn test_renderer_creation() {
        let renderer = Renderer::new(1920, 1080, 30).await;
        assert!(renderer.is_ok());
    }
    
    #[tokio::test]
    async fn test_frame_rendering() {
        let renderer = Renderer::new(1920, 1080, 30).await.unwrap();
        let frame = pcc::Frame {
            id: 1,
            timestamp: std::time::SystemTime::now(),
            width: 1920,
            height: 1080,
            data: vec![0; 1920 * 1080 * 3], // Black frame
        };
        
        renderer.buffer.push_frame(frame).await.unwrap();
        if let Some(buffered_frame) = renderer.buffer.next_frame().await.unwrap() {
            assert!(renderer.render_frame(&buffered_frame).await.is_ok());
        }
    }
} 