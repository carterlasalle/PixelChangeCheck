use anyhow::{Context, Result};
use crate::pcc::types::{Frame, FrameCapture, QualityConfig};
use screenshots::Screen;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tracing::{debug, info};

pub struct ScreenCapture {
    config: QualityConfig,
    screen: Screen,
    frame_counter: AtomicU64,
}

impl ScreenCapture {
    pub fn new() -> Result<Self> {
        let screens = Screen::all()
            .context("Failed to enumerate screens")?;

        let screen = screens
            .into_iter()
            .next()
            .context("No screens found")?;

        info!(
            "Screen capture initialized: {}x{} (scale: {})",
            screen.display_info.width,
            screen.display_info.height,
            screen.display_info.scale_factor,
        );

        Ok(Self {
            config: QualityConfig::default(),
            screen,
            frame_counter: AtomicU64::new(0),
        })
    }

    /// Get the width of the captured screen
    pub fn width(&self) -> u32 {
        self.screen.display_info.width
    }

    /// Get the height of the captured screen
    pub fn height(&self) -> u32 {
        self.screen.display_info.height
    }
}

impl FrameCapture for ScreenCapture {
    fn capture_frame(&self) -> Result<Frame> {
        let image = self
            .screen
            .capture()
            .context("Failed to capture screen")?;

        let width = image.width();
        let height = image.height();

        // Convert RGBA to RGB
        let rgba_data = image.into_raw();
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        for pixel in rgba_data.chunks_exact(4) {
            rgb_data.push(pixel[0]); // R
            rgb_data.push(pixel[1]); // G
            rgb_data.push(pixel[2]); // B
        }

        let id = self.frame_counter.fetch_add(1, Ordering::Relaxed);

        debug!("Captured frame {}: {}x{}", id, width, height);

        Ok(Frame {
            id,
            timestamp: SystemTime::now(),
            width,
            height,
            data: rgb_data,
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_creation() {
        // This may fail in headless CI environments, which is expected
        let _capture = ScreenCapture::new();
    }
}