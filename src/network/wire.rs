//! The three update operations a viewer knows how to apply, and the cost
//! model that chooses between them.
//!
//! * `Fill`  -- a solid rectangle. Cheapest thing that can happen to a
//!   region, and exactly what a window background, a selection, or a
//!   blinking block cursor is.
//! * `Copy`  -- repeat a region the receiver already holds. What a scroll
//!   is, once verified. A hash match only proposes it; the sender verifies
//!   the pixels and the receiver reads from its pre-transaction buffer, so
//!   overlapping moves and swaps behave the way Xpra specifies.
//! * `Rect`  -- exact replacement pixels, LZ4-compressed. The fallback that
//!   is always correct.

use crate::pcc::types::{PixelChange, BYTES_PER_PIXEL};
use anyhow::Result;

pub const OP_RECT: u8 = 0x01;
pub const OP_FILL: u8 = 0x02;
pub const OP_COPY: u8 = 0x03;

/// Bytes of per-op framing every op shares: x(4) + y(4) + width(4) +
/// height(4) + kind(1) + payload_len(4). A  appends a further 8
/// bytes for its source origin; see [].
pub const OP_HEADER_BYTES: usize = 21;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireOp {
    /// Exact replacement pixels. `compressed` is LZ4 with a prepended size.
    Rect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        compressed: Vec<u8>,
    },
    /// A solid colour across the rectangle. Carries no payload beyond the
    /// three colour bytes.
    Fill {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: [u8; 3],
    },
    /// Write the receiver's own pixels from `src` into `dst`, as they were
    /// before this update was applied. The source is explicit because a
    /// scroll is precisely "put what was elsewhere over here", and an
    /// implicit one would make the op a no-op.
    Copy {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        src_x: u32,
        src_y: u32,
    },
}

impl WireOp {
    pub fn rect(&self) -> (u32, u32, u32, u32) {
        match self {
            WireOp::Rect {
                x,
                y,
                width,
                height,
                ..
            }
            | WireOp::Fill {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, *width, *height),
            WireOp::Copy {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, *width, *height),
        }
    }

    pub fn is_copy(&self) -> bool {
        matches!(self, WireOp::Copy { .. })
    }

    pub fn encode_into(&self, out: &mut Vec<u8>) {
        let (x, y, w, h) = self.rect();
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(&w.to_le_bytes());
        out.extend_from_slice(&h.to_le_bytes());
        match self {
            WireOp::Rect { compressed, .. } => {
                out.push(OP_RECT);
                out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
                out.extend_from_slice(compressed);
            }
            WireOp::Fill { color, .. } => {
                out.push(OP_FILL);
                out.extend_from_slice(&(BYTES_PER_PIXEL as u32).to_le_bytes());
                out.extend_from_slice(color);
            }
            WireOp::Copy { src_x, src_y, .. } => {
                out.push(OP_COPY);
                out.extend_from_slice(&src_x.to_le_bytes());
                out.extend_from_slice(&src_y.to_le_bytes());
            }
        }
    }

    /// Bytes this op will occupy on the wire, including its header. Used
    /// to choose between representations *before* paying for any of them.
    pub fn wire_len(&self) -> usize {
        OP_HEADER_BYTES
            + match self {
                WireOp::Rect { compressed, .. } => compressed.len(),
                WireOp::Fill { .. } => BYTES_PER_PIXEL,
                // A copy carries no payload length; it has two u32s of
                // source origin where a length would be.
                WireOp::Copy { .. } => 8 - 4,
            }
    }

    /// Geometry and payload checks, with the numbers in the message.
    pub fn validate(&self, frame_width: u32, frame_height: u32) -> Result<()> {
        let (x, y, w, h) = self.rect();
        if w == 0 || h == 0 {
            anyhow::bail!("empty op: {w}x{h} at ({x},{y})");
        }
        let end_x = x
            .checked_add(w)
            .ok_or_else(|| anyhow::anyhow!("op x overflow: x={x} width={w}"))?;
        let end_y = y
            .checked_add(h)
            .ok_or_else(|| anyhow::anyhow!("op y overflow: y={y} height={h}"))?;
        if end_x > frame_width || end_y > frame_height {
            anyhow::bail!("op ({x},{y}) {w}x{h} exceeds frame {frame_width}x{frame_height}");
        }
        if let WireOp::Rect { width, height, .. } = self {
            let expected = (*width as u64) * (*height as u64) * BYTES_PER_PIXEL as u64;
            if expected > crate::pcc::types::MAX_FRAME_BYTES as u64 {
                anyhow::bail!(
                    "op too large: {width}x{height} = {expected} raw bytes \
                     (max_frame_bytes={})",
                    crate::pcc::types::MAX_FRAME_BYTES
                );
            }
        }
        // A copy reads the receiver's own surface, so its source has to be
        // inside it too.
        if let WireOp::Copy {
            src_x,
            src_y,
            width,
            height,
            ..
        } = self
        {
            let sx1 = src_x
                .checked_add(*width)
                .ok_or_else(|| anyhow::anyhow!("copy source x overflow: {src_x}+{width}"))?;
            let sy1 = src_y
                .checked_add(*height)
                .ok_or_else(|| anyhow::anyhow!("copy source y overflow: {src_y}+{height}"))?;
            if sx1 > frame_width || sy1 > frame_height {
                anyhow::bail!(
                    "copy source ({src_x},{src_y}) {width}x{height} exceeds frame \
                     {frame_width}x{frame_height}"
                );
            }
        }
        Ok(())
    }

    /// Total rectangle area, so overlapping ops can be accounted for.
    pub fn area(&self) -> u64 {
        let (_, _, w, h) = self.rect();
        w as u64 * h as u64
    }
}

impl From<PixelChange> for WireOp {
    /// Wrap exact replacement pixels, LZ4-compressing them.
    fn from(change: PixelChange) -> Self {
        let compressed =
            crate::encoder::compression::compress_frame(&change.data).expect("lz4 cannot fail");
        WireOp::Rect {
            x: change.x,
            y: change.y,
            width: change.width,
            height: change.height,
            compressed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_len_matches_encoded_length() {
        let ops = vec![
            WireOp::Rect {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
                compressed: vec![0xAA; 17],
            },
            WireOp::Fill {
                x: 0,
                y: 0,
                width: 5,
                height: 5,
                color: [1, 2, 3],
            },
            WireOp::Copy {
                x: 6,
                y: 7,
                width: 8,
                height: 9,
                src_x: 20,
                src_y: 21,
            },
        ];
        for op in ops {
            let mut buf = Vec::new();
            op.encode_into(&mut buf);
            assert_eq!(buf.len(), op.wire_len(), "wire_len disagrees with encoding");
        }
    }

    #[test]
    fn out_of_bounds_ops_are_rejected_with_numbers() {
        let op = WireOp::Fill {
            x: 30,
            y: 0,
            width: 10,
            height: 2,
            color: [0, 0, 0],
        };
        let err = op.validate(32, 32).unwrap_err().to_string();
        assert!(
            err.contains("(30,0) 10x2 exceeds frame 32x32"),
            "unhelpful: {err}"
        );
    }

    #[test]
    fn empty_op_is_rejected() {
        let op = WireOp::Copy {
            x: 0,
            y: 0,
            width: 0,
            height: 4,
            src_x: 0,
            src_y: 0,
        };
        assert!(op.validate(32, 32).is_err());
    }

    #[test]
    fn a_copy_whose_source_is_outside_the_frame_is_refused() {
        let op = WireOp::Copy {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            src_x: 30,
            src_y: 0,
        };
        let err = op.validate(32, 32).unwrap_err().to_string();
        assert!(err.contains("copy source (30,0)"), "unhelpful: {err}");
    }
}
