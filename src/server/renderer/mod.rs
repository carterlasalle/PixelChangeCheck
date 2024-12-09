mod buffer;
pub use buffer::FrameBuffer;

use anyhow::Result;
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
        // Initialize FFmpeg
        ffmpeg::init()?;
        
        // Create frame buffer
        let buffer = Arc::new(FrameBuffer::new(width, height));
        
        // Set up output context for display
        let context = ffmpeg::format::output(&format!("sdl2://PCC Display"))?;
        let stream = context.add_stream()?;
        
        // Configure stream
        let mut codec = stream.codec().encoder().video()?;
        codec.set_width(width);
        codec.set_height(height);
        codec.set_format(ffmpeg::format::Pixel::RGB24);
        codec.set_frame_rate(Some((fps as i32, 1)));
        codec.set_time_base(Some((1, fps as i32)));
        
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
                self.render_frame(&frame).await?;
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
        self.stream.codec().encoder().video()?.send_frame(&video_frame)?;
        while self.stream.codec().encoder().video()?.receive_packet(&mut packet).is_ok() {
            context.write_packet(&packet)?;
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