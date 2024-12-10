mod buffer;
pub use buffer::FrameBuffer;

use anyhow::{Context, Result};
use display_info::DisplayInfo;
use ffmpeg_next as ffmpeg;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};
use tracing::{error, info};

pub struct Renderer {
    buffer: Arc<FrameBuffer>,
    _input_context: Arc<Mutex<ffmpeg::format::context::Input>>,
    output_context: Arc<Mutex<ffmpeg::format::context::Output>>,
    _decoder: Arc<Mutex<ffmpeg::codec::decoder::video::Video>>,
    encoder: Arc<Mutex<ffmpeg::codec::encoder::video::Video>>,
    _video_stream_index: usize,
    _stream_index: usize,
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
            let input_format = ffmpeg::format::input(&format!("avfoundation:{}:0", 0))
                .context("Failed to find avfoundation input format")?;
            
            // Configure input options for screen capture
            let mut options = ffmpeg::Dictionary::new();
            options.set("framerate", &fps.to_string());
            options.set("capture_cursor", "1");
            options.set("pixel_format", "rgb24");
            
            // Get primary display coordinates (0, 0)
            let display_info = DisplayInfo::from_point(0, 0)
                .context("Failed to get display info")?;
            
            // Open input context with screen capture device
            ffmpeg::format::input_with_dictionary(&format!("avfoundation:{}:0", display_info.id), options)
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
        
        let mut decoder = ffmpeg::codec::decoder::Decoder::from_parameters(codec_params)
            .context("Failed to create video decoder")?
            .video()
            .context("Failed to create video decoder")?;
        
        decoder.set_format(ffmpeg::format::Pixel::RGB24);
        
        // Create output format context with SDL2 output
        let output_format = ffmpeg::format::output("sdl2", "")
            .context("Failed to find SDL2 output format")?
            .format();
        let output_context = ffmpeg::format::output("PCC Display")
            .context("Failed to create output context")?;
        
        // Find H264 encoder
        let encoder_codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
            .ok_or_else(|| anyhow::anyhow!("H.264 encoder not found"))?;
        
        // Add video stream
        let mut stream = output_context.add_stream(encoder_codec)?;
        let stream_index = stream.index();
        
        // Configure stream parameters
        stream.parameters_mut().set_width(width);
        stream.parameters_mut().set_height(height);
        stream.parameters_mut().set_format(ffmpeg::format::Pixel::RGB24);
        stream.parameters_mut().set_codec_tag(0);
        stream.set_time_base((1, fps as i32));
        
        // Create and configure encoder context
        let codec_id = stream.parameters().codec_id();
        let encoder_codec = ffmpeg::encoder::find(codec_id)
            .ok_or_else(|| anyhow::anyhow!("Could not find encoder"))?;
        let mut encoder = ffmpeg::encoder::video(codec_id)
            .ok_or_else(|| anyhow::anyhow!("Failed to create video encoder"))?;
        
        // Configure encoder
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(ffmpeg::format::Pixel::RGB24);
        encoder.set_time_base((1, fps as i32));
        
        #[cfg(target_os = "macos")]
        {
            // Set VideoToolbox hardware acceleration options
            encoder.set_option("allow_sw", "1")?;
            encoder.set_option("realtime", "1")?;
            encoder.set_option("profile", "high")?;
        }
        
        // Open encoder with codec
        let mut encoder_context = encoder.open_as(encoder_codec)?;
        
        #[cfg(target_os = "macos")]
        {
            encoder_context.set_option("allow_sw", "1")?;
            encoder_context.set_option("realtime", "1")?;
            encoder_context.set_option("profile", "high")?;
        }
        
        stream.set_parameters(&encoder_context);
        
        // Write output format header
        output_context.write_header()
            .context("Failed to write output format header")?;
        
        Ok(Self {
            buffer,
            _input_context: Arc::new(Mutex::new(input_context)),
            output_context: Arc::new(Mutex::new(output_context)),
            _decoder: Arc::new(Mutex::new(decoder)),
            encoder: Arc::new(Mutex::new(encoder_context)),
            _video_stream_index: video_stream_index,
            _stream_index: stream_index,
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
        if let Some(buffered_frame) = self.buffer.next_frame().await? {
            Ok(Some(buffered_frame))
        } else {
            Ok(None)
        }
    }
    
    async fn render_frame(&self, frame: &buffer::BufferedFrame) -> Result<()> {
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
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self._stream_index);
            
            // Write packet with proper interleaving
            {
                let mut context = self.output_context.lock().await;
                context.write_interleaved(&packet)
                    .context("Failed to write packet")?;
            }
        }
        
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        let mut encoder = self.encoder.lock().await;
        
        // Flush encoder
        encoder.send_eof()?;
        let mut packet = ffmpeg::packet::Packet::empty();
        while encoder.receive_packet(&mut packet).is_ok() {
            {
                let mut context = self.output_context.lock().await;
                context.write_interleaved(&packet)?;
            }
        }
        
        // Write trailer and clean up
        {
            let mut context = self.output_context.lock().await;
            context.write_trailer()
                .context("Failed to write output format trailer")?;
        }
        
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
