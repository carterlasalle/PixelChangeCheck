mod buffer;
pub use buffer::FrameBuffer;

use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use tracing::{error, info};

pub struct Renderer {
    buffer: Arc<FrameBuffer>,
    input_context: Arc<Mutex<ffmpeg::format::context::Input>>,
    output_context: Arc<Mutex<ffmpeg::format::context::Output>>,
    decoder: Arc<Mutex<ffmpeg::codec::decoder::video::Video>>,
    encoder: Arc<Mutex<ffmpeg::codec::encoder::video::Video>>,
    video_stream_index: usize,
    stream_index: usize,
    frame_interval: Duration,
}

impl Renderer {
    pub async fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        // Initialize FFmpeg and register devices
        ffmpeg::init().context("Failed to initialize FFmpeg")?;
        ffmpeg::device::register_all();
        
        // Create frame buffer
        let buffer = Arc::new(FrameBuffer::new(width, height));
        
        // Initialize avfoundation input for screen capture
        #[cfg(target_os = "macos")]
        let mut input_context = {
            // Set input format to avfoundation
            let mut input_format = ffmpeg::format::input_format("avfoundation")
                .context("Failed to find avfoundation input format")?;
            
            // Configure input options for screen capture
            let mut options = ffmpeg::Dictionary::new();
            options.set("framerate", &fps.to_string())?;
            options.set("capture_cursor", "1")?;
            options.set("pixel_format", "rgb24")?;
            
            // Get primary display coordinates (0, 0)
            let display_info = DisplayInfo::from_point(0, 0)
                .context("Failed to get display info")?;
            
            // Open input context with screen capture device
            ffmpeg::format::input_with_dictionary(&format!("avfoundation:{}:{}", display_info.index, 0), options)
                .context("Failed to open screen capture device")?
        };
        
        // Find and setup video stream decoder
        let video_stream_index = input_context.streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| anyhow::anyhow!("No video stream found"))?
            .index();
        
        let input_stream = input_context.stream(video_stream_index)
            .context("Failed to get input stream")?;
        let codec_params = input_stream.parameters();
        
        let mut decoder = codec_params.decoder()
            .video()
            .context("Failed to create video decoder")?;
        
        decoder.set_format(ffmpeg::format::Pixel::RGB24);
        
        // Create output format context with SDL2 output
        let mut output_format = ffmpeg::format::output_format("sdl2")
            .context("Failed to find SDL2 output format")?;
        let mut output_context = ffmpeg::format::output_with_format("PCC Display", output_format)
            .context("Failed to create output context")?;
        
        // Find H264 encoder
        let encoder_codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
            .ok_or_else(|| anyhow::anyhow!("H.264 encoder not found"))?;
        
        // Add video stream
        let mut stream = output_context.add_stream(encoder_codec)?;
        let stream_index = stream.index();
        
        // Configure stream parameters
        {
            let enc = stream.parameters_mut();
            enc.set_width(width);
            enc.set_height(height);
            enc.set_format(ffmpeg::format::Pixel::RGB24);
            enc.set_frame_rate(Some((fps as i32, 1)));
            enc.set_time_base(Some((1, fps as i32)));
        }
        
        // Create and configure encoder context
        let codec_id = stream.parameters().codec_id();
        let codec = ffmpeg::encoder::find(codec_id)
            .ok_or_else(|| anyhow::anyhow!("Could not find encoder"))?;
        
        let mut encoder = codec.video()
            .ok_or_else(|| anyhow::anyhow!("Failed to create video encoder"))?;
        
        // Configure encoder
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(ffmpeg::format::Pixel::RGB24);
        
        #[cfg(target_os = "macos")]
        {
            // Set VideoToolbox hardware acceleration options
            encoder.set_option("allow_sw", "1")?;
            encoder.set_option("realtime", "1")?;
            encoder.set_option("profile", "high")?;
        }
        
        // Open encoder with codec
        encoder.open_as(codec)?;
        
        // Write output format header
        output_context.write_header()
            .context("Failed to write output format header")?;
        
        Ok(Self {
            buffer,
            input_context: Arc::new(Mutex::new(input_context)),
            output_context: Arc::new(Mutex::new(output_context)),
            decoder: Arc::new(Mutex::new(decoder)),
            encoder: Arc::new(Mutex::new(encoder)),
            video_stream_index,
            stream_index,
            frame_interval: Duration::from_secs(1) / fps,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Starting renderer");
        
        let mut interval = time::interval(self.frame_interval);
        
        loop {
            interval.tick().await;
            
            // Capture frame from input device
            let frame = self.capture_frame().await?;
            
            if let Some(frame) = frame {
                if let Err(e) = self.render_frame(&frame).await {
                    error!("Failed to render frame: {}", e);
                    continue;
                }
            }
        }
    }
    
    async fn capture_frame(&self) -> Result<Option<buffer::BufferedFrame>> {
        let mut input_context = self.input_context.lock().await;
        let mut decoder = self.decoder.lock().await;
        let mut packet = ffmpeg::packet::Packet::empty();
        
        // Read packets until we get a video frame
        while input_context.read_packet(&mut packet)? {
            if packet.stream() == self.video_stream_index {
                decoder.send_packet(&packet)?;
                
                let mut frame = ffmpeg::frame::Video::empty();
                if decoder.receive_frame(&mut frame)? {
                    let frame_data = frame.data(0).to_vec();
                    return Ok(Some(buffer::BufferedFrame {
                        id: packet.pts().unwrap_or(0) as u64,
                        timestamp: std::time::SystemTime::now(),
                        data: frame_data,
                        width: frame.width() as u32,
                        height: frame.height() as u32,
                    }));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn render_frame(&self, frame: &buffer::BufferedFrame) -> Result<()> {
        let mut context = self.output_context.lock().await;
        let mut encoder = self.encoder.lock().await;
        
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
        
        let mut packet = ffmpeg::packet::Packet::empty();
        while encoder.receive_packet(&mut packet)? {
            // Write packet with proper interleaving
            context.write_interleaved(&mut packet)
                .context("Failed to write packet")?;
        }
        
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        let mut context = self.output_context.lock().await;
        let mut encoder = self.encoder.lock().await;
        let mut decoder = self.decoder.lock().await;
        
        // Flush decoder
        decoder.send_eof()?;
        let mut frame = ffmpeg::frame::Video::empty();
        while decoder.receive_frame(&mut frame)? {}
        
        // Flush encoder
        encoder.send_eof()?;
        let mut packet = ffmpeg::packet::Packet::empty();
        while encoder.receive_packet(&mut packet)? {
            context.write_interleaved(&mut packet)?;
        }
        
        // Write trailer and clean up
        context.write_trailer()
            .context("Failed to write output format trailer")?;
        
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