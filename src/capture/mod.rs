use crate::pcc::types::{Frame, FrameCapture, QualityConfig};
use anyhow::{Context, Result};
use screenshots::Screen;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tracing::{debug, info};

pub struct ScreenCapture {
    screen: Screen,
    frame_counter: AtomicU64,
}

impl ScreenCapture {
    pub fn new() -> Result<Self> {
        let screens = Screen::all().context("Failed to enumerate screens")?;

        let screen = screens.into_iter().next().context("No screens found")?;

        info!(
            "Screen capture initialized: {}x{} (scale: {})",
            screen.display_info.width, screen.display_info.height, screen.display_info.scale_factor,
        );

        Ok(Self {
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
        let image = self.screen.capture().context("Failed to capture screen")?;

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
            // The origin is set by the session, not by the capture
            // source, so a frame captured before that has no meaningful
            // presentation time.
            pts_us: 0,
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
            },
            QualityConfig {
                target_fps: 60,
                max_fps: 60,
                quality: 1.0,
            },
        ]
    }
}

/// A deterministic, animated test pattern used when there's no real display
/// to capture from (headless servers, CI, containers) or when explicitly
/// requested for local testing. Draws a moving bar plus a bouncing box so
/// PCC has something meaningful to diff between frames.
pub struct SyntheticCapture {
    width: u32,
    height: u32,
    frame_counter: AtomicU64,
}

impl SyntheticCapture {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            frame_counter: AtomicU64::new(0),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl FrameCapture for SyntheticCapture {
    fn capture_frame(&self) -> Result<Frame> {
        let id = self.frame_counter.fetch_add(1, Ordering::Relaxed);
        let (width, height) = (self.width, self.height);
        let mut data = vec![30u8; (width * height * 3) as usize];

        // A static-ish gradient background (so most of the frame is
        // unchanged between calls, letting PCC skip most of it).
        for y in 0..height {
            let shade = (y * 40 / height.max(1)) as u8;
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                data[idx] = 20 + shade;
                data[idx + 1] = 20 + shade / 2;
                data[idx + 2] = 40;
            }
        }

        // A bouncing box that actually moves each frame, so PCC has real
        // changed regions to detect and transmit.
        let box_size: u32 = (width.min(height) / 8).max(10);
        let period_x = (width - box_size).max(1);
        let period_y = (height - box_size).max(1);
        let t = id % (2 * period_x.max(period_y) as u64).max(1);
        let bx = triangle_wave(t, period_x as u64) as u32;
        let by = triangle_wave(t, period_y as u64) as u32;

        for dy in 0..box_size.min(height) {
            for dx in 0..box_size.min(width) {
                let x = (bx + dx).min(width - 1);
                let y = (by + dy).min(height - 1);
                let idx = ((y * width + x) * 3) as usize;
                data[idx] = 220;
                data[idx + 1] = 60;
                data[idx + 2] = 60;
            }
        }

        debug!("Generated synthetic frame {}: {}x{}", id, width, height);

        Ok(Frame {
            id,
            timestamp: SystemTime::now(),
            pts_us: 0,
            width,
            height,
            data,
        })
    }

    fn supported_configs(&self) -> Vec<QualityConfig> {
        vec![QualityConfig::default()]
    }
}

/// A triangle wave in `[0, period]` used to bounce the synthetic test box
/// back and forth without any floating point drift.
fn triangle_wave(t: u64, period: u64) -> u64 {
    let period = period.max(1);
    let cycle = 2 * period;
    let phase = t % cycle;
    if phase <= period {
        phase
    } else {
        cycle - phase
    }
}

/// Either a real screen or the synthetic test pattern, chosen at startup.
/// Falling back automatically means `share` still runs (and can be tested
/// end-to-end) on headless machines with no attached display.
pub enum CaptureSource {
    Screen(ScreenCapture),
    Synthetic(SyntheticCapture),
}

impl CaptureSource {
    /// Open the requested capture source. If `force_synthetic` is false and
    /// no real screen is available, transparently falls back to synthetic.
    pub fn open(force_synthetic: bool) -> Result<Self> {
        if force_synthetic {
            info!("Using synthetic test-pattern capture (requested)");
            return Ok(Self::Synthetic(SyntheticCapture::new(1280, 720)));
        }

        match ScreenCapture::new() {
            Ok(capture) => Ok(Self::Screen(capture)),
            Err(e) => {
                info!("No real display available ({e}); falling back to synthetic test pattern");
                Ok(Self::Synthetic(SyntheticCapture::new(1280, 720)))
            }
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Self::Screen(c) => c.width(),
            Self::Synthetic(c) => c.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::Screen(c) => c.height(),
            Self::Synthetic(c) => c.height(),
        }
    }

    pub fn capture_frame(&self) -> Result<Frame> {
        match self {
            Self::Screen(c) => c.capture_frame(),
            Self::Synthetic(c) => c.capture_frame(),
        }
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

    #[test]
    fn test_synthetic_capture_produces_correctly_sized_frames() {
        let capture = SyntheticCapture::new(64, 48);
        let frame = capture.capture_frame().unwrap();
        assert_eq!(frame.data.len(), 64 * 48 * 3);
    }

    #[test]
    fn test_synthetic_capture_changes_between_frames() {
        let capture = SyntheticCapture::new(64, 48);
        let frame1 = capture.capture_frame().unwrap();
        let frame2 = capture.capture_frame().unwrap();
        assert_ne!(frame1.data, frame2.data, "the bouncing box should move");
    }

    #[test]
    fn test_capture_source_falls_back_when_forced() {
        let source = CaptureSource::open(true).unwrap();
        assert_eq!(source.width(), 1280);
        assert_eq!(source.height(), 720);
    }
}
