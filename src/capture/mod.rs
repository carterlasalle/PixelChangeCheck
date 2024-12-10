use anyhow::{Context, Result};
use crate::pcc::types::{Frame, FrameCapture, QualityConfig};
use display_info::DisplayInfo;
use ffmpeg_next as ffmpeg;
use std::{sync::Arc, time::SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub struct ScreenCapture {
    config: QualityConfig,
    display_info: DisplayInfo,
    frame_counter: u64,
    input_context: Arc<Mutex<ffmpeg::format::context::Input>>,
    video_stream_index: usize,
}

impl ScreenCapture {
    pub fn new() -> Result<Self> {
        // Initialize FFmpeg
        ffmpeg::init().context("Failed to initialize FFmpeg")?;
        
        // Get primary display info
        let display_info = DisplayInfo::from_point(0, 0)
            .context("Failed to get primary display info")?;
            
        // Set up FFmpeg capture format
        let mut options = ffmpeg::Dictionary::new();
        options.set("framerate", &format!("{}", QualityConfig::default().target_fps));
        options.set("video_size", &format!("{}x{}", display_info.width, display_info.height));
        
        #[cfg(target_os = "macos")]
        let input_context = ffmpeg::format::input_with_dictionary(&format!("avfoundation:{}:0", display_info.id), options)
            .context("Failed to create input context for macOS screen capture")?;
            
        #[cfg(target_os = "windows")]
        let input_context = ffmpeg::format::input_with_dictionary("gdigrab", options)
            .context("Failed to create input context for Windows screen capture")?;
            
        #[cfg(target_os = "linux")]
        let input_context = ffmpeg::format::input_with_dictionary("x11grab", options)
            .context("Failed to create input context for Linux screen capture")?;
        
        // Find video stream
        let video_stream_index = input_context
            .streams()
            .best(ffmpeg::media::Type::Video)
            .context("No video stream found")?
            .index();
            
        Ok(Self {
            config: QualityConfig::default(),
            display_info,
            frame_counter: 0,
            input_context: Arc::new(Mutex::new(input_context)),
            video_stream_index,
        })
    }
    
    async fn read_frame(&mut self) -> Result<ffmpeg::frame::Video> {
        let mut input = self.input_context.lock().await;
        let codec_params = input
            .stream(self.video_stream_index)
            .context("Failed to get video stream")?
            .parameters();
        let decoder = ffmpeg::codec::decoder::Decoder::from_parameters(codec_params)
            .context("Failed to create decoder")?;
        
        let mut video_decoder = decoder.video()
            .context("Failed to get video decoder")?;
            
        let mut frame = ffmpeg::frame::Video::empty();
        
        while let Some((stream, packet)) = input.packets().next() {
            if stream.index() == self.video_stream_index {
                video_decoder.send_packet(&packet)?;
                while video_decoder.receive_frame(&mut frame).is_ok() {
                    return Ok(frame);
                }
            }
        }
        
        Err(anyhow::anyhow!("End of stream"))
    }
}

impl FrameCapture for ScreenCapture {
    fn capture_frame(&self) -> Result<Frame> {
        // Create async runtime for FFmpeg operations
        let rt = tokio::runtime::Runtime::new()?;
        
        rt.block_on(async {
            let frame = self.read_frame().await?;
            
            // Convert frame data to RGB format
            let mut rgb_frame = ffmpeg::frame::Video::empty();
            let mut converter = ffmpeg::software::scaling::Context::get(
                frame.format(),
                frame.width(),
                frame.height(),
                ffmpeg::format::Pixel::RGB24,
                frame.width(),
                frame.height(),
                ffmpeg::software::scaling::Flags::BILINEAR,
            )?;
            
            converter.run(&frame, &mut rgb_frame)?;
            
            Ok(Frame {
                id: self.frame_counter,
                timestamp: SystemTime::now(),
                width: frame.width() as u32,
                height: frame.height() as u32,
                data: rgb_frame.data(0).to_vec(),
            })
        })
    }
    
    fn supported_configs(&self) -> Vec<QualityConfig> {
        vec![
            QualityConfig {
                target_fps: 30,
                max_fps: 60,
                quality: 0.8,
                compression_level: 6,
            },
            QualityConfig {
                target_fps: 60,
                max_fps: 60,
                quality: 1.0,
                compression_level: 4,
            },
        ]
    }
    
    fn configure(&mut self, config: QualityConfig) -> Result<()> {
        self.config = config;
        
        // Update FFmpeg capture parameters
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut input = self.input_context.lock().await;
            let mut options = ffmpeg::Dictionary::new();
            options.set("framerate", &format!("{}", config.target_fps));
            
            // Recreate input context with new settings
            #[cfg(target_os = "macos")]
            {
                *input = ffmpeg::format::input_with_dictionary(&format!("avfoundation:{}:0", self.display_info.id), options)
                    .context("Failed to update macOS screen capture settings")?;
            }
            
            #[cfg(target_os = "windows")]
            {
                *input = ffmpeg::format::input_with_dictionary("gdigrab", options)
                    .context("Failed to update Windows screen capture settings")?;
            }
            
            #[cfg(target_os = "linux")]
            {
                *input = ffmpeg::format::input_with_dictionary("x11grab", options)
                    .context("Failed to update Linux screen capture settings")?;
            }
            
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_screen_capture_creation() {
        let capture = ScreenCapture::new();
        assert!(capture.is_ok(), "Failed to create screen capture instance");
    }
    
    #[test]
    fn test_capture_frame() {
        let capture = ScreenCapture::new().unwrap();
        let frame = capture.capture_frame();
        assert!(frame.is_ok(), "Failed to capture frame");
        
        let frame = frame.unwrap();
        assert!(frame.width > 0, "Invalid frame width");
        assert!(frame.height > 0, "Invalid frame height");
        assert!(!frame.data.is_empty(), "Empty frame data");
    }
}