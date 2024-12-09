mod buffer;
pub use buffer::FrameBuffer;

use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use tracing::{debug, error, info};

pub struct Renderer {
    buffer: Arc<FrameBuffer>,
    context: Arc<Mutex<ffmpeg::format::context::Output>>,
    stream: ffmpeg::format::stream::Stream,
    frame_interval: Duration,
}

impl Renderer {
    pub async fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        // Initialize FFmpeg with all available formats and codecs
        ffmpeg::init().context("Failed to initialize FFmpeg")?;
        
        // Create frame buffer
        let buffer = Arc::new(FrameBuffer::new(width, height));
        
        // Set up output context for display
        let context = ffmpeg::format::output(&format!("sdl2://PCC Display"))
            .context("Failed to create FFmpeg output context")?;
        let stream = context.add_stream()
            .context("Failed to add video stream")?;
        
        // Configure stream
        let mut codec = stream.codec().encoder().video()
            .context("Failed to create video encoder")?;
            
        // Set codec parameters
        codec.set_width(width);
        codec.set_height(height);
        codec.set_format(ffmpeg::format::Pixel::RGB24);
        codec.set_frame_rate(Some((fps as i32, 1)));
        codec.set_time_base(Some((1, fps as i32)));
        
        // Configure hardware acceleration based on platform
        #[cfg(target_os = "macos")]
        {
            // Use VideoToolbox for hardware acceleration on macOS
            codec.set_encoder("h264_videotoolbox")
                .context("Failed to set VideoToolbox encoder")?;
            
            // Set VideoToolbox-specific options
            let mut opts = ffmpeg::Dictionary::new();
            opts.set("allow_sw", "1"); // Allow fallback to software encoding
            opts.set("realtime", "1"); // Enable realtime encoding
            opts.set("profile", "high"); // Use high profile for better quality
            codec.set_options(opts)
                .context("Failed to set VideoToolbox options")?;
        }
        
        #[cfg(target_os = "linux")]
        {
            // Use VAAPI for hardware acceleration on Linux
            codec.set_encoder("h264_vaapi")
                .context("Failed to set VAAPI encoder")?;
            
            // Set VAAPI-specific options
            let mut opts = ffmpeg::Dictionary::new();
            opts.set("low_power", "1"); // Use low power mode if available
            opts.set("idr_interval", "30"); // Set IDR frame interval
            codec.set_options(opts)
                .context("Failed to set VAAPI options")?;
        }
        
        // Open codec
        codec.open_as(codec.id())
            .context("Failed to open video codec")?;
        
        Ok(Self {
            buffer: buffer.clone(),
            context: Arc::new(Mutex::new(context)),
            stream,
            frame_interval: Duration::from_secs(1) / fps,
        })
    }
    
    // Start the rendering loop
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
    
    // Render a single frame
    async fn render_frame(&self, frame: &buffer::BufferedFrame) -> Result<()> {
        let mut context = self.context.lock().await;
        
        // Create FFmpeg frame
        let mut video_frame = ffmpeg::frame::Video::new(
            ffmpeg::format::Pixel::RGB24,
            frame.width,
            frame.height,
        );
        
        // Copy frame data
        video_frame.data_mut(0).copy_from_slice(&frame.data);
        
        // Write frame to output
        let mut packet = ffmpeg::packet::Packet::empty();
        self.stream.codec().encoder().video()?
            .send_frame(&video_frame)
            .context("Failed to send frame to encoder")?;
            
        while self.stream.codec().encoder().video()?
            .receive_packet(&mut packet)
            .is_ok() 
        {
            context.write_packet(&packet)
                .context("Failed to write packet")?;
        }
        
        Ok(())
    }
    
    // Clean up resources
    pub async fn shutdown(&self) -> Result<()> {
        let mut context = self.context.lock().await;
        context.flush()?;
        self.buffer.clear().await;
        Ok(())
    }
}

// Tests
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