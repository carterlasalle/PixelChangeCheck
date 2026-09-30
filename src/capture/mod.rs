use crate::pcc::types::{Frame, FrameCapture, QualityConfig};
use device_query::{DeviceQuery, DeviceState};

/// Where the pointer is, in shared-surface pixels. `None` means the
/// pointer is not over the shared area (or the platform gave no answer),
/// which the sharer sends as `CursorHide` rather than a stale position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorSample {
    pub x: u32,
    pub y: u32,
}

/// The platform half of the cursor plane. Today no cross-platform cursor
/// position API is linked, so the only implementation reports "unknown"
/// and the sharer sends `CursorHide`. This trait is the seam a
/// platform sampler (CGEvent, GetCursorPos, Wayland pointer) fills in —
/// without it a future implementation would have to touch the loop.
pub trait CursorSampler: Send {
    fn sample(&mut self, width: u32, height: u32) -> Option<CursorSample>;
}

/// No sampler linked: always unknown. The share still works; viewers
/// simply never show a remote cursor.
pub struct NoCursorSampler;

impl CursorSampler for NoCursorSampler {
    fn sample(&mut self, _width: u32, _height: u32) -> Option<CursorSample> {
        None
    }
}

/// The live platform pointer via `device_query` (CGEvent on macOS, X11
/// query on Linux, GetCursorPos on Windows). Coordinates are global
/// display pixels; the region offset is subtracted so the sample lands in
/// shared-surface space, and anything outside the shared rectangle reads
/// as "not over the share" (`None` → `CursorHide` downstream).
///
/// `DeviceState` is deliberately *not* stored: on Linux it holds an
/// `Rc<X11Connection>`, which is `!Send`, and the sampler crosses into
/// `capture_loop`'s `Send` future. A fresh state per sample costs one
/// X11 open per frame — the same call the old code made once at
/// startup, now made where the `Send` bound can hold.
pub struct PlatformCursorSampler {
    /// Origin of the shared area in global display pixels.
    origin: (i32, i32),
}

impl PlatformCursorSampler {
    pub fn new(origin: (i32, i32)) -> Self {
        Self { origin }
    }
}

impl CursorSampler for PlatformCursorSampler {
    fn sample(&mut self, width: u32, height: u32) -> Option<CursorSample> {
        let (gx, gy) = DeviceState::new().get_mouse().coords;
        let (x, y) = (gx - self.origin.0, gy - self.origin.1);
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return None;
        }
        Some(CursorSample {
            x: x as u32,
            y: y as u32,
        })
    }
}

/// A scripted sampler for tests and `--synthetic`: a pointer that sweeps
/// diagonally so cursor messages actually flow end to end.
pub struct SweepSampler {
    step: u32,
}

impl SweepSampler {
    pub fn new() -> Self {
        Self { step: 0 }
    }
}

impl Default for SweepSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorSampler for SweepSampler {
    fn sample(&mut self, width: u32, height: u32) -> Option<CursorSample> {
        if width == 0 || height == 0 {
            return None;
        }
        let x = self.step % width;
        let y = (self.step / width.max(1)) % height;
        self.step = self.step.wrapping_add(37);
        Some(CursorSample { x, y })
    }
}
use anyhow::{Context, Result};
use screenshots::Screen;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tracing::{debug, info};

pub struct ScreenCapture {
    screen: Screen,
    frame_counter: AtomicU64,
    /// Sub-rectangle of the display, in display pixels. `None` means the
    /// whole display. Clamped to the display at open so every later frame
    /// has identical geometry, which the epoch logic requires.
    region: Option<(u32, u32, u32, u32)>,
    /// A captured window's current geometry, refreshed per frame. `Some`
    /// exactly when sharing a window or application: the frame comes from
    /// the OS window list rather than the display framebuffer.
    window: Option<WindowShare>,
}

/// One shared window: matched by title substring at open, re-resolved per
/// frame by the same match. Re-resolving (not caching a handle) is what
/// keeps the share alive across window close/reopen and what lets the
/// geometry follow a moved window — at the cost of one enumeration per
/// frame, which is a window-list walk, not a capture.
#[derive(Debug, Clone)]
struct WindowShare {
    /// Lowercase substring matched against window titles.
    needle: String,
    /// Application match as well (bundle id or process name fragment).
    /// `None` for a pure `--window` share.
    app: Option<String>,
    /// Open-time geometry, used only for the frozen-frame fallback
    /// when the window vanishes. Live geometry always comes from the
    /// captured image (physical pixels — 2x the window-list points on
    /// Retina); the share loop's epoch logic absorbs the open-time vs
    /// first-frame difference exactly like a display resize.
    last: (u32, u32),
}

/// What to capture. Display and Region are backed by the OS screen
/// enumeration today; Window and Application are the seam for the
/// platform pickers (ScreenCaptureKit / WGC / portal) and currently fall
/// back to the full display with a warning rather than failing the share.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureTarget {
    /// Whole display by index into `Screen::all()` order (0 = primary).
    Display(usize),
    /// Sub-rectangle of display 0, as `x,y,w,h` in display pixels.
    Region {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// OS window handle or title. Falls back to the display for now.
    Window(String),
    /// Application name or bundle id. Falls back to the display for now.
    Application(String),
}

impl CaptureTarget {
    /// Parse `--region x,y,w,h`. Rejects zero sizes up front: a 0-wide
    /// capture would poison every geometry check downstream.
    pub fn parse_region(spec: &str) -> Result<(u32, u32, u32, u32)> {
        let parts: Vec<&str> = spec.split(',').collect();
        let nums: Result<Vec<u32>, _> = parts.iter().map(|p| p.trim().parse()).collect();
        let nums = nums.map_err(|_| {
            anyhow::anyhow!("--region must be x,y,w,h in display pixels, got '{spec}'")
        })?;
        if nums.len() != 4 || nums[2] == 0 || nums[3] == 0 {
            anyhow::bail!("--region must be x,y,w,h with nonzero size, got '{spec}'");
        }
        Ok((nums[0], nums[1], nums[2], nums[3]))
    }
}

impl ScreenCapture {
    pub fn new() -> Result<Self> {
        Self::open_target(&CaptureTarget::Display(0))
    }

    /// Open one display by index, or the primary when out of range.
    pub fn open_display(index: usize) -> Result<Self> {
        Self::open_target(&CaptureTarget::Display(index))
    }

    /// Open a capture target. Window/Application fall back to the display
    /// with a warning: sharing the wrong rectangle is recoverable, failing
    /// the share is not. Region is clamped to the display so geometry is
    /// stable across frames.
    pub fn open_target(target: &CaptureTarget) -> Result<Self> {
        let screens = Screen::all().context("Failed to enumerate screens")?;
        let (screen, region) = match target {
            CaptureTarget::Display(i) => (
                screens.into_iter().nth(*i).context(format!(
                    "display {i} not found ({} display(s) available)",
                    Screen::all().map(|s| s.len()).unwrap_or(0)
                ))?,
                None,
            ),
            CaptureTarget::Region {
                x,
                y,
                width,
                height,
            } => {
                let screen = screens.into_iter().next().context("No screens found")?;
                let (dw, dh) = (screen.display_info.width, screen.display_info.height);
                // Clamp the origin into the display, then clamp the size
                // into what remains. Minimum 1x1 so a fully-offscreen
                // request still yields stable geometry instead of an error
                // mid-stream; the epoch logic requires identical geometry
                // on every frame.
                let x = (*x).min(dw.saturating_sub(1));
                let y = (*y).min(dh.saturating_sub(1));
                let width = (*width).min(dw.saturating_sub(x)).max(1);
                let height = (*height).min(dh.saturating_sub(y)).max(1);
                // Some backends enforce their own minimum (this
                // machine's returns 2x2 for a 1x1 request). Geometry comes
                // from the returned image on every frame, so store what
                // the platform actually honors — never what was asked. A
                // backend that ignores the region here would silently
                // share the whole screen below, which is a privacy
                // violation, not a sizing quirk.
                let probe = screen
                    .capture_area(x as i32, y as i32, width, height)
                    .context("verifying the platform honors region capture")?;
                let (width, height) = (probe.width(), probe.height());
                if width == 0 || height == 0 {
                    anyhow::bail!(
                        "platform returned an empty image for region {x},{y}: region capture unsupported here"
                    );
                }
                (screen, Some((x, y, width, height)))
            }
            CaptureTarget::Window(id) => {
                let screen = screens.into_iter().next().context("No screens found")?;
                let needle = id.to_ascii_lowercase();
                // Fail fast when nothing matches: sharing the display
                // instead would leak pixels the user explicitly excluded.
                let (w, h) = find_window(&needle, None)
                    .context(format!("no visible window matches '{id}'"))?;
                info!("Sharing window matching '{id}' ({w}x{h})");
                return Ok(Self {
                    screen,
                    frame_counter: AtomicU64::new(0),
                    region: None,
                    window: Some(WindowShare {
                        needle,
                        app: None,
                        last: (w, h),
                    }),
                });
            }
            CaptureTarget::Application(id) => {
                let screen = screens.into_iter().next().context("No screens found")?;
                let needle = id.to_ascii_lowercase();
                let (w, h) = find_window(&needle, Some(&needle))
                    .context(format!("no visible window matches application '{id}'"))?;
                info!("Sharing application matching '{id}' ({w}x{h})");
                return Ok(Self {
                    screen,
                    frame_counter: AtomicU64::new(0),
                    region: None,
                    window: Some(WindowShare {
                        needle: needle.clone(),
                        app: Some(needle),
                        last: (w, h),
                    }),
                });
            }
        };

        info!(
            "Screen capture initialized: {}x{} (scale: {}){}",
            screen.display_info.width,
            screen.display_info.height,
            screen.display_info.scale_factor,
            region
                .map(|(x, y, w, h)| format!(" region {x},{y} {w}x{h}"))
                .unwrap_or_default(),
        );

        Ok(Self {
            screen,
            frame_counter: AtomicU64::new(0),
            region,
            window: None,
        })
    }

    /// Origin of the shared area in global display pixels: the region
    /// offset, or the display's own offset in a multi-monitor layout.
    /// The cursor sampler subtracts it so overlay coordinates land in
    /// shared-surface space.
    pub fn origin(&self) -> (i32, i32) {
        // Window shares report (0,0): the window image is already cropped
        // to the window, so cursor coordinates need no offset. (A live
        // window position would be better; xcap reports per-window x/y
        // but they are screen-space only on some platforms, so this stays
        // conservative rather than confidently wrong.)
        if self.window.is_some() {
            return (0, 0);
        }
        match self.region {
            Some((x, y, _, _)) => (x as i32, y as i32),
            None => (self.screen.display_info.x, self.screen.display_info.y),
        }
    }

    /// List available displays as (index, width, height) for the picker.
    pub fn list_displays() -> Vec<(usize, u32, u32)> {
        Screen::all()
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.display_info.width, s.display_info.height))
            .collect()
    }

    /// Get the width of the captured area (region or whole screen).
    /// Width of the shared area. For window shares this is the last
    /// known window width: the live match may resize, and the epoch
    /// logic in the share loop handles the change like any geometry
    /// change. Open-time size is only the starting point.
    pub fn width(&self) -> u32 {
        if let Some(w) = &self.window {
            return w.last.0;
        }
        self.region
            .map(|(_, _, w, _)| w)
            .unwrap_or(self.screen.display_info.width)
    }

    /// Get the height of the captured area (region, window, or screen).
    pub fn height(&self) -> u32 {
        if let Some(w) = &self.window {
            return w.last.1;
        }
        self.region
            .map(|(_, _, _, h)| h)
            .unwrap_or(self.screen.display_info.height)
    }
}

/// Find a visible window by title (and optionally app) substring.
/// Returns its current size. Skips minimized and empty windows: a
/// minimized window has no pixels to share, and a 0-size one would poison
/// every geometry check downstream.
fn find_window(needle: &str, app: Option<&str>) -> Result<(u32, u32)> {
    let windows = xcap::Window::all().context("enumerating windows")?;
    for w in &windows {
        let title = w.title().unwrap_or_default().to_ascii_lowercase();
        if !title.contains(needle) {
            continue;
        }
        if let Some(a) = app {
            let name = w.app_name().unwrap_or_default().to_ascii_lowercase();
            if !name.contains(a) && !title.contains(a) {
                continue;
            }
        }
        if w.is_minimized().unwrap_or(false) {
            continue;
        }
        let (width, height) = (w.width().unwrap_or(0), w.height().unwrap_or(0));
        if width == 0 || height == 0 {
            continue;
        }
        return Ok((width, height));
    }
    anyhow::bail!("no visible window matches")
}

/// List windows for the picker: (title, app, w, h), smallest first so a
/// `--window` needle is easy to aim.
pub fn list_windows() -> Vec<(String, String, u32, u32)> {
    let Ok(windows) = xcap::Window::all() else {
        return Vec::new();
    };
    let mut out: Vec<(String, String, u32, u32)> = windows
        .iter()
        .filter(|w| !w.is_minimized().unwrap_or(true))
        .filter_map(|w| {
            let (width, height) = (w.width().unwrap_or(0), w.height().unwrap_or(0));
            if width == 0 || height == 0 {
                return None;
            }
            Some((
                w.title().unwrap_or_default(),
                w.app_name().unwrap_or_default(),
                width,
                height,
            ))
        })
        .collect();
    out.sort_by_key(|(_, _, w, h)| w * h);
    out
}

/// Capture one frame of a shared window. The window is re-resolved by
/// title on every frame; when it vanishes the last geometry repeats with
/// the previous pixels (a freeze, not an error — the epoch logic needs
/// stable geometry, and a close is temporary until the window returns).
fn capture_window_frame(counter: &AtomicU64, share: &WindowShare) -> Result<Frame> {
    // xcap's image type is a different `image` major version than ours,
    // so the two `RgbaImage`s never meet: work on raw bytes only.
    let (width, height, rgba_data): (u32, u32, Vec<u8>) = match find_live_window(share)
        .and_then(|w| w.capture_image().context("capturing window pixels"))
    {
        Ok(image) => (image.width(), image.height(), image.into_raw()),
        Err(_) => {
            // Frozen frame: last size, solid grey. The diff against the
            // live reference still converges the moment the window
            // returns.
            let (w, h) = share.last;
            let mut raw = Vec::with_capacity((w * h * 4) as usize);
            for _ in 0..w * h {
                raw.extend_from_slice(&[128, 128, 128, 255]);
            }
            (w, h, raw)
        }
    };
    let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
    for pixel in rgba_data.as_chunks::<4>().0 {
        rgb_data.push(pixel[0]);
        rgb_data.push(pixel[1]);
        rgb_data.push(pixel[2]);
    }
    let id = counter.fetch_add(1, Ordering::Relaxed);
    Ok(Frame {
        id,
        timestamp: SystemTime::now(),
        pts_us: 0,
        width,
        height,
        data: rgb_data,
    })
}

/// The live window handle matching a share, or an error when it is gone.
fn find_live_window(share: &WindowShare) -> Result<xcap::Window> {
    let windows = xcap::Window::all().context("enumerating windows")?;
    for w in windows {
        let title = w.title().unwrap_or_default().to_ascii_lowercase();
        if !title.contains(&share.needle) {
            continue;
        }
        if let Some(a) = &share.app {
            let name = w.app_name().unwrap_or_default().to_ascii_lowercase();
            if !name.contains(a) && !title.contains(a) {
                continue;
            }
        }
        if w.is_minimized().unwrap_or(false) {
            continue;
        }
        if w.width().unwrap_or(0) == 0 || w.height().unwrap_or(0) == 0 {
            continue;
        }
        return Ok(w);
    }
    anyhow::bail!("shared window is not visible right now")
}

impl FrameCapture for ScreenCapture {
    fn capture_frame(&self) -> Result<Frame> {
        // Window shares re-resolve every frame: the handle from open may
        // be a closed window by now, and geometry follows the live match.
        if let Some(share) = &self.window {
            return capture_window_frame(&self.frame_counter, share);
        }
        // Region capture happens at the capture layer, not by cropping
        // afterwards: `capture_area` reads only the requested rectangle.
        let image = match self.region {
            Some((x, y, w, h)) => self
                .screen
                .capture_area(x as i32, y as i32, w, h)
                .context("Failed to capture screen region")?,
            None => self.screen.capture().context("Failed to capture screen")?,
        };

        let width = image.width();
        let height = image.height();

        // Convert RGBA to RGB
        let rgba_data = image.into_raw();
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        for pixel in rgba_data.as_chunks::<4>().0 {
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
        Self::open_target(force_synthetic, &CaptureTarget::Display(0))
    }

    /// Open a capture target (display index or region). Synthetic mode
    /// ignores the target: the test pattern is display-independent.
    pub fn open_target(force_synthetic: bool, target: &CaptureTarget) -> Result<Self> {
        if force_synthetic {
            info!("Using synthetic test-pattern capture (requested)");
            return Ok(Self::Synthetic(SyntheticCapture::new(1280, 720)));
        }

        match ScreenCapture::open_target(target) {
            Ok(capture) => Ok(Self::Screen(capture)),
            Err(e) => {
                // Window/application misses propagate: falling back to a
                // test pattern would silently share *something* when the
                // user explicitly excluded everything else. Display/region
                // failures (headless machines) still fall back.
                if matches!(
                    target,
                    CaptureTarget::Window(_) | CaptureTarget::Application(_)
                ) {
                    return Err(e);
                }
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

    /// Origin of the shared area for the cursor sampler. Synthetic has no
    /// display behind it, so (0,0) — the sweep coordinates already are
    /// surface space.
    pub fn origin(&self) -> (i32, i32) {
        match self {
            Self::Screen(c) => c.origin(),
            Self::Synthetic(_) => (0, 0),
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
    fn window_share_fails_fast_on_no_match() {
        let err = ScreenCapture::open_target(&CaptureTarget::Window("no such window xyz".into()))
            .err()
            .map(|e| e.to_string())
            .expect("bogus title must fail");
        assert!(err.contains("no visible window"), "unhelpful: {err}");
        let err = ScreenCapture::open_target(&CaptureTarget::Application("no such app xyz".into()))
            .err()
            .map(|e| e.to_string())
            .expect("bogus app must fail");
        assert!(err.contains("no visible window"), "unhelpful: {err}");
    }

    #[test]
    fn window_list_and_live_capture_agree() {
        let wins = list_windows();
        // This machine has windows; CI may not. Empty list skips.
        let Some((title, _, w, h)) = wins.first() else {
            return;
        };
        assert!(*w > 0 && *h > 0);
        // Share it by a title fragment and capture one frame.
        let needle = title.chars().take(4).collect::<String>();
        if needle.trim().is_empty() {
            return;
        }
        let cap = ScreenCapture::open_target(&CaptureTarget::Window(needle));
        // The smallest window may have closed between listing and open;
        // only assert when the open succeeded.
        if let Ok(cap) = cap {
            let frame = cap.capture_frame().unwrap();
            // Geometry comes from the captured image (physical pixels on
            // Retina), not the window list (logical points).
            assert_eq!(
                frame.data.len(),
                frame.width as usize * frame.height as usize * 3
            );
            assert!(frame.width > 0 && frame.height > 0);
        }
    }

    #[test]
    fn platform_sampler_never_returns_out_of_bounds() {
        let mut s = PlatformCursorSampler::new((0, 0));
        for _ in 0..50 {
            if let Some(c) = s.sample(1512, 982) {
                assert!(c.x < 1512 && c.y < 982);
            }
        }
        // Far-away origin puts the real pointer outside the share.
        let mut far = PlatformCursorSampler::new((1_000_000, 1_000_000));
        assert!(far.sample(64, 48).is_none());
    }

    #[test]
    fn sweep_sampler_walks_inside_the_frame() {
        let mut s = SweepSampler::new();
        for _ in 0..200 {
            let c = s.sample(64, 48).expect("nonempty frame has a cursor");
            assert!(c.x < 64 && c.y < 48);
        }
        assert!(SweepSampler::new().sample(0, 48).is_none());
        let mut n = NoCursorSampler;
        assert!(n.sample(64, 48).is_none());
    }

    #[test]
    fn region_spec_parses_and_rejects_garbage() {
        assert_eq!(
            CaptureTarget::parse_region("100,200,1280,720").unwrap(),
            (100, 200, 1280, 720)
        );
        assert_eq!(
            CaptureTarget::parse_region(" 0, 0, 64, 48 ").unwrap(),
            (0, 0, 64, 48)
        );
        for bad in ["", "1,2,3", "1,2,3,4,5", "a,b,c,d", "0,0,0,10", "0,0,10,0"] {
            assert!(
                CaptureTarget::parse_region(bad).is_err(),
                "{bad:?} must not parse"
            );
        }
    }

    #[test]
    fn synthetic_mode_ignores_the_target() {
        // The test pattern is display-independent, so --display/--region
        // must not change synthetic geometry.
        for target in [
            CaptureTarget::Display(3),
            CaptureTarget::Region {
                x: 10,
                y: 10,
                width: 64,
                height: 48,
            },
            CaptureTarget::Window("x".into()),
        ] {
            let source = CaptureSource::open_target(true, &target).unwrap();
            assert_eq!((source.width(), source.height()), (1280, 720));
        }
    }

    #[test]
    fn region_clamps_to_the_display() {
        // Headless CI has no display: skip rather than fail.
        let Ok(screens) = screenshots::Screen::all() else {
            return;
        };
        let Some(first) = screens.first() else {
            return;
        };
        let (dw, dh) = (first.display_info.width, first.display_info.height);
        let cap = ScreenCapture::open_target(&CaptureTarget::Region {
            x: dw + 1000,
            y: dh + 1000,
            width: 10_000,
            height: 10_000,
        })
        .unwrap();
        // Whatever survives clamping (possibly a backend minimum above
        // 1x1) is what every frame carries; the point is stability and
        // containment, not exact numbers.
        assert!(cap.width() >= 1 && cap.width() <= dw.max(2));
        assert!(cap.height() >= 1 && cap.height() <= dh.max(2));
        let frame = cap.capture_frame().unwrap();
        // Geometry comes from the returned image: what the platform
        // honors at open is what every frame carries.
        assert_eq!((frame.width, frame.height), (cap.width(), cap.height()));
        assert_eq!(
            frame.data.len(),
            frame.width as usize * frame.height as usize * 3
        );
    }

    #[test]
    fn test_capture_source_falls_back_when_forced() {
        let source = CaptureSource::open(true).unwrap();
        assert_eq!(source.width(), 1280);
        assert_eq!(source.height(), 720);
    }
}
