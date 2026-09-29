use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Frame data is stored as tightly packed RGB triples.
pub const BYTES_PER_PIXEL: usize = 3;

/// Tripwire on the decoded surface a single frame may occupy. 7680x4320
/// (8K RGB) is 99.5 MiB; nothing we support legitimately exceeds that, and
/// refusing larger inputs turns a hostile `width`/`height` pair into a
/// loud error instead of a multi-gigabyte allocation.
pub const MAX_FRAME_BYTES: usize = 100_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: u64,
    pub timestamp: SystemTime,
    /// Microseconds from the session's capture origin to *this* frame's
    /// capture, on a monotonic clock. Zero when nothing is synchronised
    /// against it.
    pub pts_us: u64,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Frame {
    /// Build a frame, rejecting any buffer whose length disagrees with its
    /// dimensions and any surface above `MAX_FRAME_BYTES`.
    pub fn new(id: u64, width: u32, height: u32, data: Vec<u8>) -> Result<Self> {
        Self::with_pts(id, width, height, data, 0)
    }

    /// A frame with an explicit presentation time, for a session that
    /// synchronises against audio.
    pub fn with_pts(id: u64, width: u32, height: u32, data: Vec<u8>, pts_us: u64) -> Result<Self> {
        let expected = rgb_len(width, height)?;
        if data.len() != expected {
            anyhow::bail!(
                "frame buffer length mismatch: {width}x{height} needs {expected} bytes, got {}",
                data.len()
            );
        }
        Ok(Self {
            id,
            timestamp: SystemTime::now(),
            pts_us,
            width,
            height,
            data,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let frame: Self = bincode::deserialize(data)?;
        frame.validate()?;
        Ok(frame)
    }

    /// Re-check the data/dimension agreement after deserializing bytes we
    /// did not construct ourselves.
    pub fn validate(&self) -> Result<()> {
        let expected = rgb_len(self.width, self.height)?;
        if self.data.len() != expected {
            anyhow::bail!(
                "frame buffer length mismatch: {}x{} needs {expected} bytes, got {}",
                self.width,
                self.height,
                self.data.len()
            );
        }
        Ok(())
    }
}

/// Exact pixel replacement over a rectangle. `data` holds precisely
/// `width * height * 3` RGB bytes, row-major and tightly packed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelChange {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl PixelChange {
    /// Checked geometry + exact payload length. Every rectangle that reaches
    /// the wire, and every one that comes off it, passes through here, so a
    /// malformed message can never be partially applied.
    pub fn validate(&self, frame_width: u32, frame_height: u32) -> Result<()> {
        let end_x = self
            .x
            .checked_add(self.width)
            .ok_or_else(|| anyhow::anyhow!("rectangle x overflow: x={}", self.x))?;
        let end_y = self
            .y
            .checked_add(self.height)
            .ok_or_else(|| anyhow::anyhow!("rectangle y overflow: y={}", self.y))?;

        if end_x > frame_width || end_y > frame_height {
            anyhow::bail!(
                "rectangle ({},{}) {}x{} exceeds frame {frame_width}x{frame_height}",
                self.x,
                self.y,
                self.width,
                self.height
            );
        }

        let expected = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|px| px.checked_mul(BYTES_PER_PIXEL))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "rectangle pixel count overflow: {}x{}",
                    self.width,
                    self.height
                )
            })?;
        if self.data.len() != expected {
            anyhow::bail!(
                "rectangle payload length mismatch: {}x{} needs {expected} bytes, got {}",
                self.width,
                self.height,
                self.data.len()
            );
        }
        Ok(())
    }

    pub fn pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Copy these exact pixels out of `src` (which must already be the
    /// authoritative reference) into `dst`.
    pub fn copy_from(&self, src: &[u8], src_width: u32, dst: &mut [u8], dst_width: u32) {
        let row_bytes = self.width as usize * BYTES_PER_PIXEL;
        for row in 0..self.height as usize {
            let from =
                ((self.y as usize + row) * src_width as usize + self.x as usize) * BYTES_PER_PIXEL;
            let to =
                ((self.y as usize + row) * dst_width as usize + self.x as usize) * BYTES_PER_PIXEL;
            dst[to..to + row_bytes].copy_from_slice(&src[from..from + row_bytes]);
        }
    }
}

/// Exact byte count of a tightly packed RGB surface, with the overflow and
/// budget checks every decode path needs.
pub fn rgb_len(width: u32, height: u32) -> Result<usize> {
    let bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|px| px.checked_mul(BYTES_PER_PIXEL))
        .ok_or_else(|| {
            anyhow::anyhow!("frame dimensions overflow: {width}x{height} ({} pixels)", {
                (width as u64) * (height as u64)
            })
        })?;
    if bytes > MAX_FRAME_BYTES {
        anyhow::bail!(
            "frame budget exceeded: max_frame_bytes={MAX_FRAME_BYTES}, requested={bytes} \
             ({width}x{height})"
        );
    }
    Ok(bytes)
}

/// Rate/quality policy shared by the capture scheduler, the encoder and the
/// viewer notification.
///
/// `compression_level` was carried here for years without any codec reading
/// it; LZ4 has no such knob. It is gone rather than kept as a lie.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Rate the scheduler aims for.
    pub target_fps: u32,
    /// Hard ceiling the scheduler may never exceed, even while healthy.
    pub max_fps: u32,
    /// JPEG quality for the explicitly-lossy browser preview and for
    /// screenshots. The authoritative surface is lossless and ignores this.
    pub quality: f32,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            target_fps: 30,
            max_fps: 60,
            quality: 0.8,
        }
    }
}

/// Change detection over two frames of identical geometry.
pub trait PixelChangeDetector {
    /// Exact pixel replacements turning `previous` into `current`.
    fn detect_changes(&self, previous: &Frame, current: &Frame) -> Result<Vec<PixelChange>>;

    /// Channel-delta below which a pixel counts as unchanged.
    fn set_threshold(&mut self, threshold: u8);

    /// Tile edge length used for the coarse scan and the merge pass.
    fn set_block_size(&mut self, block_size: u32) -> Result<()>;
}

/// A source of frames.
pub trait FrameCapture {
    fn capture_frame(&self) -> Result<Frame>;

    /// Configurations this backend can actually honour.
    fn supported_configs(&self) -> Vec<QualityConfig>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32) -> Frame {
        Frame::new(0, w, h, vec![0; rgb_len(w, h).unwrap()]).unwrap()
    }

    #[test]
    fn rect_rejects_row_wrapping() {
        // x=3 width=2 on a 4-wide frame crosses into the next scanline.
        // The flat-buffer check cannot see this; the geometry check must.
        let f = frame(4, 2);
        let bad = PixelChange {
            x: 3,
            y: 0,
            width: 2,
            height: 1,
            data: vec![255; 6],
        };
        let err = bad.validate(f.width, f.height).unwrap_err().to_string();
        assert!(err.contains("exceeds frame"), "unhelpful error: {err}");
    }

    #[test]
    fn rect_rejects_wrong_payload_length() {
        let f = frame(4, 2);
        let bad = PixelChange {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
            data: vec![255; 6], // needs 12
        };
        let err = bad.validate(f.width, f.height).unwrap_err().to_string();
        assert!(err.contains("payload length mismatch"), "unhelpful: {err}");
    }

    #[test]
    fn rect_rejects_coordinate_overflow() {
        let f = frame(4, 2);
        let bad = PixelChange {
            x: u32::MAX,
            y: 0,
            width: 1,
            height: 1,
            data: vec![0; 3],
        };
        assert!(bad.validate(f.width, f.height).is_err());
    }

    #[test]
    fn frame_new_rejects_mismatched_buffer() {
        assert!(Frame::new(0, 4, 4, vec![0; 10]).is_err());
    }

    #[test]
    fn frame_budget_is_reported_with_numbers() {
        let err = rgb_len(20_000, 20_000).unwrap_err().to_string();
        assert!(
            err.contains("max_frame_bytes=100000000"),
            "unhelpful: {err}"
        );
        assert!(err.contains("requested=1200000000"), "unhelpful: {err}");
    }

    #[test]
    fn copy_from_blits_exact_rect() {
        let src = frame(4, 2);
        let mut src = src.data;
        for b in src.iter_mut() {
            *b = 9;
        }
        let mut dst = vec![0u8; 24];
        let change = PixelChange {
            x: 1,
            y: 1,
            width: 2,
            height: 1,
            data: vec![],
        };
        change.copy_from(&src, 4, &mut dst, 4);
        // row 1, x=1..3 => bytes 15..21
        assert!(dst[12..15].iter().all(|&b| b == 0));
        assert!(dst[15..21].iter().all(|&b| b == 9));
        assert!(dst[21..].iter().all(|&b| b == 0));
    }
}
