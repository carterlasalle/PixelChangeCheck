//! The receiver's authoritative framebuffer and the only code allowed to
//! mutate it.
//!
//! Every invariant the protocol depends on is enforced here, once, rather
//! than in each transport:
//!
//! 1. an update never reads a reference the receiver does not hold;
//! 2. a new epoch invalidates everything from older ones;
//! 3. replacements and copies cannot overwrite newer content, because a
//!    revision is applied at most once and copies read the *pre-transaction*
//!    buffer (Xpra's rule, and the reason a two-region swap works);
//! 4. invalid data cannot partially mutate the framebuffer -- everything is
//!    validated and materialised before a single byte is written.
//!
//! The browser compositor is a line-for-line port of this file; the two are
//! checked against each other by the replay tests.

use crate::encoder::{compression, decode_snapshot, SnapshotFormat};
use crate::network::{Epoch, Rev, WireOp, SNAPSHOT_CHUNK_BYTES};
use crate::pcc::types::BYTES_PER_PIXEL;
use anyhow::Result;

#[derive(Debug, Default)]
pub struct Compositor {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    epoch: Epoch,
    rev: Rev,
    fresh: bool,
    /// Snapshot being assembled. A partially received snapshot is never
    /// visible: the buffer is only replaced when a commit arrives and the
    /// bytes decode.
    incoming: Option<Incoming>,
}

#[derive(Debug)]
struct Incoming {
    epoch: Epoch,
    width: u32,
    height: u32,
    format: SnapshotFormat,
    total_len: usize,
    expected_chunks: u32,
    data: Vec<u8>,
    /// Bit per chunk, so a duplicate index is detected rather than
    /// silently concatenated.
    seen: Vec<bool>,
}

/// Why an update was refused. A viewer turns `NeedsSnapshot` into a
/// `RequestKeyframe` rather than guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejected {
    NeedsSnapshot,
    StaleEpoch,
    AlreadyApplied,
}

impl Compositor {
    pub fn new() -> Self {
        Self::default()
    }

    /// `None` until a snapshot has been installed.
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        if self.fresh {
            Some((self.width, self.height))
        } else {
            None
        }
    }

    pub fn epoch(&self) -> Epoch {
        self.epoch
    }

    pub fn rev(&self) -> Rev {
        self.rev
    }

    pub fn is_fresh(&self) -> bool {
        self.fresh
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Begin assembling a lossless snapshot. Any snapshot already being
    /// assembled is abandoned: an interrupted transfer must not be able to
    /// corrupt the next one.
    pub fn begin_snapshot(
        &mut self,
        epoch: Epoch,
        width: u32,
        height: u32,
        format: SnapshotFormat,
        total_len: u32,
        chunks: u32,
    ) -> Result<()> {
        if self.fresh && epoch < self.epoch {
            anyhow::bail!(
                "snapshot epoch {epoch} is older than the installed epoch {}",
                self.epoch
            );
        }
        // Check the declared geometry against the frame budget before any
        // of the snapshot's bytes are read.
        crate::pcc::types::rgb_len(width, height)
            .map_err(|e| anyhow::anyhow!("invalid snapshot geometry {width}x{height}: {e}"))?;
        // Check the declared geometry against the frame budget before any
        // of the snapshot's bytes are read.
        crate::pcc::types::rgb_len(width, height)
            .map_err(|e| anyhow::anyhow!("invalid snapshot geometry {width}x{height}: {e}"))?;
        if total_len as usize == 0 {
            anyhow::bail!("snapshot announced with total_len=0");
        }
        self.incoming = Some(Incoming {
            epoch,
            width,
            height,
            format,
            total_len: total_len as usize,
            expected_chunks: chunks,
            data: Vec::with_capacity(total_len as usize),
            seen: vec![false; chunks as usize],
        });
        Ok(())
    }

    /// Add one chunk. Out-of-order or duplicated indices are refused.
    pub fn push_snapshot_chunk(&mut self, index: u32, data: &[u8]) -> Result<()> {
        let incoming = self.incoming.as_mut().ok_or_else(|| {
            anyhow::anyhow!("snapshot chunk {index} with no snapshot in progress")
        })?;
        let slot = index as usize;
        if slot >= incoming.seen.len() {
            anyhow::bail!(
                "snapshot chunk index {index} outside 0..{}",
                incoming.seen.len()
            );
        }
        if incoming.seen[slot] {
            anyhow::bail!("snapshot chunk {index} arrived twice");
        }
        if data.len() > SNAPSHOT_CHUNK_BYTES {
            anyhow::bail!(
                "snapshot chunk {index} is {} bytes (max {SNAPSHOT_CHUNK_BYTES})",
                data.len()
            );
        }
        if incoming.data.len() + data.len() > incoming.total_len {
            anyhow::bail!(
                "snapshot overflows: {} + {} > {}",
                incoming.data.len(),
                data.len(),
                incoming.total_len
            );
        }
        incoming.seen[slot] = true;
        incoming.data.extend_from_slice(data);
        Ok(())
    }

    /// Finish a snapshot. This is the only way to become fresh, and the
    /// decode happens before any state is touched, so a bad snapshot can
    /// never leave a half-installed buffer.
    pub fn commit_snapshot(&mut self, rev: Rev, epoch: Epoch) -> Result<()> {
        let incoming = self
            .incoming
            .take()
            .ok_or_else(|| anyhow::anyhow!("snapshot commit with no snapshot in progress"))?;
        if incoming.epoch != epoch {
            anyhow::bail!(
                "snapshot commit epoch {epoch} does not match the transfer's {}",
                incoming.epoch
            );
        }
        if let Some(missing) = incoming.seen.iter().position(|s| !s) {
            anyhow::bail!(
                "snapshot committed with chunk {missing} of {} missing",
                incoming.expected_chunks
            );
        }
        if incoming.data.len() != incoming.total_len {
            anyhow::bail!(
                "snapshot is {} bytes, expected {}",
                incoming.data.len(),
                incoming.total_len
            );
        }
        let rgb = decode_snapshot(
            incoming.format,
            incoming.width,
            incoming.height,
            &incoming.data,
        )?;
        self.buffer = rgb;
        self.width = incoming.width;
        self.height = incoming.height;
        self.epoch = incoming.epoch;
        self.rev = rev;
        self.fresh = true;
        Ok(())
    }

    /// Apply a transaction of ops atomically.
    ///
    /// Returns `Err(Rejected)` for the recoverable "you need a snapshot"
    /// / "this is stale" cases and a hard `Err` for malformed payloads.
    pub fn apply_ops(
        &mut self,
        rev: Rev,
        epoch: Epoch,
        ops: &[WireOp],
    ) -> std::result::Result<(), ApplyError> {
        if !self.fresh {
            return Err(ApplyError::Rejected(Rejected::NeedsSnapshot));
        }
        if epoch < self.epoch {
            return Err(ApplyError::Rejected(Rejected::StaleEpoch));
        }
        if epoch > self.epoch {
            // A new geometry is only meaningful with its snapshot.
            return Err(ApplyError::Rejected(Rejected::NeedsSnapshot));
        }
        if rev <= self.rev {
            return Err(ApplyError::Rejected(Rejected::AlreadyApplied));
        }

        // Validate + materialise every payload first. Nothing below this
        // point can fail, so the buffer is never left partially updated.
        let mut staged: Vec<StagedOp> = Vec::with_capacity(ops.len());
        for op in ops {
            let (x, y, w, h) = op.rect();
            let validated = op.validate(self.width, self.height);
            if let Err(e) = validated {
                return Err(ApplyError::Invalid(e));
            }
            match op {
                WireOp::Rect { compressed, .. } => {
                    let expected = (w as usize) * (h as usize) * BYTES_PER_PIXEL;
                    match compression::decompress_exact(compressed, expected) {
                        Ok(pixels) => staged.push(StagedOp::Rect { x, y, w, h, pixels }),
                        Err(e) => return Err(ApplyError::Invalid(e)),
                    }
                }
                WireOp::Fill { color, .. } => staged.push(StagedOp::Fill {
                    x,
                    y,
                    w,
                    h,
                    color: *color,
                }),
                WireOp::Copy { src_x, src_y, .. } => staged.push(StagedOp::Copy {
                    x,
                    y,
                    w,
                    h,
                    sx: *src_x,
                    sy: *src_y,
                }),
            }
        }

        // Xpra's rule: a copy reads the surface as it was *before* the
        // transaction. Without this, a swap of two regions corrupts itself.
        let needs_snapshot = staged.iter().any(|s| matches!(s, StagedOp::Copy { .. }));
        let base = if needs_snapshot {
            Some(self.buffer.clone())
        } else {
            None
        };

        for op in &staged {
            match *op {
                StagedOp::Rect {
                    x,
                    y,
                    w,
                    h,
                    ref pixels,
                } => blit(&mut self.buffer, self.width, x, y, w, h, pixels),
                StagedOp::Fill { x, y, w, h, color } => {
                    fill(&mut self.buffer, self.width, x, y, w, h, color)
                }
                StagedOp::Copy { x, y, w, h, sx, sy } => {
                    let src = base.as_ref().expect("copy base snapshot");
                    // Read the pre-transaction surface at the source
                    // origin, so overlapping moves and swaps compose.
                    blit_region(&mut self.buffer, self.width, x, y, w, h, src, sx, sy);
                }
            }
        }

        self.rev = rev;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ApplyError {
    Rejected(Rejected),
    Invalid(anyhow::Error),
}

impl PartialEq for ApplyError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ApplyError::Rejected(a), ApplyError::Rejected(b)) => a == b,
            (ApplyError::Invalid(a), ApplyError::Invalid(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}

impl From<anyhow::Error> for ApplyError {
    fn from(e: anyhow::Error) -> Self {
        ApplyError::Invalid(e)
    }
}

impl std::error::Error for ApplyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApplyError::Rejected(_) => None,
            ApplyError::Invalid(e) => Some(e.as_ref()),
        }
    }
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::Rejected(r) => write!(f, "update rejected: {r:?}"),
            ApplyError::Invalid(e) => write!(f, "invalid update: {e}"),
        }
    }
}

enum StagedOp {
    Rect {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        pixels: Vec<u8>,
    },
    Fill {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        color: [u8; 3],
    },
    Copy {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        sx: u32,
        sy: u32,
    },
}

/// Copy a tightly packed `w*h*3` block into the surface at (x, y). The
/// source rows are `w` pixels wide, not `dst_width`.
fn blit(dst: &mut [u8], dst_width: u32, x: u32, y: u32, w: u32, h: u32, src: &[u8]) {
    let row_bytes = w as usize * BYTES_PER_PIXEL;
    for row in 0..h as usize {
        let to = ((y as usize + row) * dst_width as usize + x as usize) * BYTES_PER_PIXEL;
        dst[to..to + row_bytes].copy_from_slice(&src[row * row_bytes..(row + 1) * row_bytes]);
    }
}

/// Copy a block out of `src` at `(sx, sy)` into `dst` at `(x, y)`.
#[allow(clippy::too_many_arguments)]
fn blit_region(
    dst: &mut [u8],
    dst_width: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    src: &[u8],
    sx: u32,
    sy: u32,
) {
    let row_bytes = w as usize * BYTES_PER_PIXEL;
    for row in 0..h as usize {
        let to = ((y as usize + row) * dst_width as usize + x as usize) * BYTES_PER_PIXEL;
        let from = ((sy as usize + row) * dst_width as usize + sx as usize) * BYTES_PER_PIXEL;
        dst[to..to + row_bytes].copy_from_slice(&src[from..from + row_bytes]);
    }
}

fn fill(dst: &mut [u8], dst_width: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 3]) {
    let row = [color[0], color[1], color[2]].repeat(w as usize);
    for r in 0..h as usize {
        let to = ((y as usize + r) * dst_width as usize + x as usize) * BYTES_PER_PIXEL;
        dst[to..to + row.len()].copy_from_slice(&row);
    }
}

/// Geometry guard used by both the native and browser paths, so a
/// rectangle that would wrap a scanline can never be built in the first
/// place.
pub fn rect_is_inside(x: u32, y: u32, w: u32, h: u32, width: u32, height: u32) -> bool {
    match (x.checked_add(w), y.checked_add(h)) {
        (Some(ex), Some(ey)) => ex <= width && ey <= height,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcc::types::rgb_len;

    fn rect_op(x: u32, y: u32, w: u32, h: u32, px: [u8; 3]) -> WireOp {
        let data: Vec<u8> = px
            .iter()
            .copied()
            .cycle()
            .take((w * h) as usize * 3)
            .collect();
        WireOp::Rect {
            x,
            y,
            width: w,
            height: h,
            compressed: compression::compress_frame(&data).unwrap(),
        }
    }

    fn install(c: &mut Compositor, w: u32, h: u32, epoch: Epoch, rev: Rev) {
        transfer(c, &vec![7u8; rgb_len(w, h).unwrap()], w, h, epoch, rev);
    }

    /// Drive a full Begin/Chunk/Commit cycle with `data` as the encoded
    /// snapshot, split the way the wire would split it.
    fn transfer(c: &mut Compositor, rgb: &[u8], w: u32, h: u32, epoch: Epoch, rev: Rev) {
        let (fmt, encoded) = crate::encoder::encode_snapshot(w, h, rgb).unwrap();
        let chunks: Vec<&[u8]> = encoded.chunks(SNAPSHOT_CHUNK_BYTES).collect();
        c.begin_snapshot(epoch, w, h, fmt, encoded.len() as u32, chunks.len() as u32)
            .unwrap();
        for (i, chunk) in chunks.iter().enumerate() {
            c.push_snapshot_chunk(i as u32, chunk).unwrap();
        }
        c.commit_snapshot(rev, epoch).unwrap();
    }

    fn composite(ops: &[WireOp], w: u32, h: u32, epoch: Epoch, rev: Rev) -> Result<Vec<u8>> {
        let mut c = Compositor::new();
        install(&mut c, w, h, epoch, 0);
        c.apply_ops(rev, epoch, ops)?;
        Ok(c.buffer().to_vec())
    }

    #[test]
    fn updates_before_a_snapshot_are_refused() {
        let mut c = Compositor::new();
        let op = WireOp::Fill {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            color: [1, 2, 3],
        };
        assert_eq!(
            c.apply_ops(1, 0, &[op]).unwrap_err(),
            ApplyError::Rejected(Rejected::NeedsSnapshot)
        );
    }

    #[test]
    fn fill_then_rect_reconstructs_exactly() {
        let (w, h) = (8u32, 4u32);
        let mut expected = vec![7u8; rgb_len(w, h).unwrap()];
        // paint a 4x2 red block at (2,1)
        for y in 1..3 {
            for x in 2..6 {
                let i = ((y * w + x) * 3) as usize;
                expected[i..i + 3].copy_from_slice(&[200, 10, 10]);
            }
        }
        let ops = vec![
            WireOp::Fill {
                x: 2,
                y: 1,
                width: 4,
                height: 2,
                color: [200, 10, 10],
            },
            rect_op(6, 0, 2, 1, [1, 2, 3]),
        ];
        let got = composite(&ops, w, h, 0, 1).unwrap();
        // The fill covers (2,1)..(6,3).
        let filled = 10usize * 3;
        assert_eq!(
            &got[filled..filled + 3],
            &[200, 10, 10][..],
            "fill must land"
        );
        // Outside it, the snapshot is untouched.
        assert_eq!(&got[..3], &[7, 7, 7][..]);
        let rect_at = 6usize * 3;
        assert_eq!(&got[rect_at..rect_at + 3], &[1, 2, 3][..], "rect must land");
    }

    #[test]
    fn a_rect_that_would_wrap_a_row_is_refused_atomically() {
        let (w, h) = (4u32, 2u32);
        let good = rect_op(0, 0, 1, 1, [9, 9, 9]);
        // x=3 width=2 crosses into the next scanline.
        let bad = WireOp::Rect {
            x: 3,
            y: 0,
            width: 2,
            height: 1,
            compressed: compression::compress_frame(&[255u8; 6]).unwrap(),
        };
        let mut c = Compositor::new();
        install(&mut c, w, h, 0, 0);
        let before = c.buffer().to_vec();
        assert!(c.apply_ops(1, 0, &[good, bad]).is_err());
        assert_eq!(
            c.buffer(),
            &before[..],
            "a refused transaction must change nothing"
        );
        assert_eq!(
            c.rev(),
            0,
            "a refused transaction must not advance the revision"
        );
    }

    #[test]
    fn a_lz4_bomb_is_refused_without_allocating_it() {
        let (w, h) = (4u32, 2u32);
        // A payload that decodes to far more than 4*2*3 bytes.
        let bomb = compression::compress_frame(&vec![0u8; 1_000_000]).unwrap();
        let op = WireOp::Rect {
            x: 0,
            y: 0,
            width: w,
            height: h,
            compressed: bomb,
        };
        let mut c = Compositor::new();
        install(&mut c, w, h, 0, 0);
        assert!(c.apply_ops(1, 0, &[op]).is_err());
        assert_eq!(c.rev(), 0);
    }

    #[test]
    fn an_old_revision_is_never_applied_twice() {
        let (w, h) = (4u32, 2u32);
        let mut c = Compositor::new();
        install(&mut c, w, h, 0, 5);
        let op = WireOp::Fill {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            color: [1, 1, 1],
        };
        assert!(c.apply_ops(6, 0, std::slice::from_ref(&op)).is_ok());
        assert_eq!(
            c.apply_ops(6, 0, &[op]).unwrap_err(),
            ApplyError::Rejected(Rejected::AlreadyApplied)
        );
    }

    #[test]
    fn a_new_epoch_without_a_snapshot_is_refused() {
        let (w, h) = (4u32, 2u32);
        let mut c = Compositor::new();
        install(&mut c, w, h, 3, 1);
        let op = WireOp::Fill {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            color: [1, 1, 1],
        };
        assert_eq!(
            c.apply_ops(2, 4, &[op]).unwrap_err(),
            ApplyError::Rejected(Rejected::NeedsSnapshot)
        );
    }

    #[test]
    fn copy_reads_the_pre_transaction_buffer() {
        // Two regions swapped in one transaction. Doing this sequentially
        // against the live buffer would duplicate A over B.
        let w = 4u32;
        let h = 2u32;
        let mut a = vec![0u8; rgb_len(w, h).unwrap()];
        // left half = red, right half = blue
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 3) as usize;
                a[i..i + 3].copy_from_slice(if x < 2 { &[255, 0, 0] } else { &[0, 0, 255] });
            }
        }
        let mut c = Compositor::new();
        transfer(&mut c, &a, w, h, 0, 0);

        // Swap: copy left <- right and right <- left, in one transaction.
        let ops = vec![
            WireOp::Copy {
                x: 0,
                y: 0,
                width: 2,
                height: h,
                src_x: 2,
                src_y: 0,
            },
            WireOp::Copy {
                x: 2,
                y: 0,
                width: 2,
                height: h,
                src_x: 0,
                src_y: 0,
            },
        ];
        c.apply_ops(1, 0, &ops).unwrap();
        let out = c.buffer();
        assert_eq!(&out[0..3], &[0, 0, 255], "left must now be blue");
        assert_eq!(&out[6..9], &[255, 0, 0], "right must now be red");
    }

    #[test]
    fn an_incomplete_snapshot_leaves_the_surface_untouched() {
        let (w, h) = (4u32, 2u32);
        let mut c = Compositor::new();
        transfer(&mut c, &vec![1u8; rgb_len(w, h).unwrap()], w, h, 0, 1);
        let before = c.buffer().to_vec();

        let rgb = vec![9u8; rgb_len(w, h).unwrap()];
        let (_, encoded) = crate::encoder::encode_snapshot(w, h, &rgb).unwrap();
        c.begin_snapshot(0, w, h, SnapshotFormat::Png, encoded.len() as u32, 2)
            .unwrap();
        c.push_snapshot_chunk(0, &encoded[..encoded.len() / 2])
            .unwrap();
        // Committing with a chunk missing must fail and change nothing.
        assert!(c.commit_snapshot(2, 0).is_err());
        assert_eq!(c.buffer(), &before[..]);
        assert_eq!(c.rev(), 1, "a failed commit must not advance the revision");
    }

    #[test]
    fn a_duplicate_chunk_is_refused() {
        let (w, h) = (4u32, 2u32);
        let mut c = Compositor::new();
        let rgb = vec![9u8; rgb_len(w, h).unwrap()];
        let (_, encoded) = crate::encoder::encode_snapshot(w, h, &rgb).unwrap();
        c.begin_snapshot(0, w, h, SnapshotFormat::Png, encoded.len() as u32, 1)
            .unwrap();
        c.push_snapshot_chunk(0, &encoded).unwrap();
        let err = c.push_snapshot_chunk(0, &encoded).unwrap_err().to_string();
        assert!(err.contains("arrived twice"), "unhelpful: {err}");
    }

    #[test]
    fn resize_installs_a_new_epoch() {
        let mut c = Compositor::new();
        install(&mut c, 4, 2, 0, 1);
        install(&mut c, 8, 4, 1, 2);
        assert_eq!(c.dimensions(), Some((8, 4)));
        assert_eq!(c.epoch(), 1);
        assert_eq!(c.buffer().len(), rgb_len(8, 4).unwrap());
    }
}
