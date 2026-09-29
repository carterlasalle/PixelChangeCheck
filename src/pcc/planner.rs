//! Choosing *what* to send for a frame that changed.
//!
//! Three questions, in order, exactly as the decision should go:
//!
//! 1. is the content already right at the receiver? -> say nothing;
//! 2. can an exact region the receiver already holds reproduce it? -> `Copy`
//!    (this is what a scroll is, once the pixels have been verified);
//! 3. is a solid colour or a lossless patch the cheap representation?
//!
//! and finally, if none of that is smaller than a fresh snapshot, say so
//! rather than shipping 2,000 tiny rectangles.

use super::detector::{
    changed_fraction, tile_hashes, tile_hashes_of, verify_displacement_of, Grid, PCCDetector,
};
use super::types::{Frame, PixelChange, BYTES_PER_PIXEL};
use crate::network::WireOp;
use anyhow::Result;

/// Run the displacement search only when the frame is churning this much.
///
/// Receipt: a static desktop changes well under 1% per frame and a cursor
/// blink under 0.1%. Hashing every tile twice costs roughly a frame's worth
/// of memory traffic, so paying it on a screen that is not moving is waste.
pub const SCROLL_SEARCH_MIN_FRACTION: f64 = 0.10;

/// Largest scroll displacement to look for, in pixels.
///
/// Receipt: a mouse-wheel notch is ~100px, a PageUp a few hundred, and a
/// 60fps smooth-scroll fling peaks in the low thousands. 4096 covers a fast
/// fling while keeping the search linear and bounded.
pub const SCROLL_SHIFT_LIMIT: u32 = 4096;

/// Fraction of tiles that must hash-match before a displacement is even
/// verified byte-for-byte.
pub const SCROLL_MATCH_THRESHOLD: f64 = 0.90;

/// Below this many matched tiles a "scroll" is more likely to be a static
/// region that happens to be uniform. 8 tiles is 256x256px at the default
/// block size.
pub const SCROLL_MIN_TILES: u32 = 8;

/// Bytes of framing an update carries around its ops: kind, revision,
/// epoch, count.
pub const UPDATE_OVERHEAD_BYTES: usize = 1 + 8 + 4 + 4;

/// The ops that turn the reference into the current frame, and the totals a
/// caller needs to decide between patches and a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub ops: Vec<WireOp>,
    /// Encoded size of the ops, headers included.
    pub wire_len: usize,
    /// Area of the frame the ops rewrite, in pixels.
    pub changed_pixels: u64,
}

impl Plan {
    /// A plan that sends nothing.
    pub fn idle() -> Self {
        Self {
            ops: Vec::new(),
            wire_len: 0,
            changed_pixels: 0,
        }
    }

    /// A plan whose patches were measured and found not worth sending; the
    /// caller should ship a snapshot instead. `wire_len` still carries the
    /// measured cost so the decision is visible.
    pub fn prefer_snapshot(measured_wire_len: usize, changed_pixels: u64) -> Self {
        Self {
            ops: Vec::new(),
            wire_len: measured_wire_len,
            changed_pixels,
        }
    }
}

/// A verified displacement: the destination rectangle and the pixel shift
/// its source sits at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Displacement {
    /// (x, y, width, height) of the destination.
    pub region: (u32, u32, u32, u32),
    /// Pixel shift from destination to source, as the receiver sees it.
    pub shift: (i64, i64),
}

#[derive(Debug, Clone, Copy)]
pub struct PlanLimits {
    /// Encoded size of the most recent full snapshot. A patch set bigger
    /// than this is not worth sending.
    pub snapshot_bytes: usize,
    /// Hard ceiling for one update before it must be split or replaced.
    pub max_update_bytes: usize,
}

#[derive(Debug)]
pub struct Planner {
    detector: PCCDetector,
    scroll_enabled: bool,
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            detector: PCCDetector::default(),
            scroll_enabled: true,
        }
    }
}

impl Planner {
    pub fn new(threshold: u8, block_size: u32) -> Result<Self> {
        Ok(Self {
            detector: PCCDetector::new(threshold, block_size)?,
            scroll_enabled: true,
        })
    }

    pub fn set_scroll_search(&mut self, enabled: bool) {
        self.scroll_enabled = enabled;
    }

    pub fn detector(&self) -> &PCCDetector {
        &self.detector
    }
    /// `reference` is the authoritative surface, behind a shared buffer so
    /// a joining viewer can be handed the very same bytes rather than a
    /// copy taken every frame. It advances in step with what is sent.
    pub fn plan(
        &self,
        reference: &mut [u8],
        width: u32,
        height: u32,
        current: &Frame,
        limits: PlanLimits,
    ) -> Result<Plan> {
        if width != current.width || height != current.height {
            anyhow::bail!(
                "frame dimensions do not match: reference {width}x{height} vs current {}x{}",
                current.width,
                current.height
            );
        }
        let changes = self
            .detector
            .detect(reference, &current.data, width, height)?;
        if changes.is_empty() {
            return Ok(Plan::idle());
        }

        let mut ops: Vec<WireOp> = Vec::new();
        let mut wire_len = 0usize;
        let mut changed_pixels = 0u64;

        // 1. Try to explain the churn as a verified displacement.
        let mut remaining: Vec<PixelChange> = changes;
        if self.scroll_enabled
            && changed_fraction(&remaining, width, height) >= SCROLL_SEARCH_MIN_FRACTION
        {
            if let Some(found) = self.find_scroll(reference, current, width, height) {
                let (rx, ry, rw, rh) = found.region;
                let op = WireOp::Copy {
                    x: rx,
                    y: ry,
                    width: rw,
                    height: rh,
                    src_x: (rx as i64 + found.shift.0) as u32,
                    src_y: (ry as i64 + found.shift.1) as u32,
                };
                wire_len += op.wire_len();
                changed_pixels += op.area();
                ops.push(op);
                remaining = subtract_region(&remaining, found.region);
            }
        }

        // 2. Per-rectangle representation choice.
        for change in &remaining {
            let op = self.represent(change);
            wire_len += op.wire_len();
            changed_pixels += op.area();
            ops.push(op);
        }

        // 3. A patch set bigger than a fresh snapshot is not worth sending.
        if wire_len > limits.snapshot_bytes || wire_len > limits.max_update_bytes {
            // The reference is left untouched: nothing shipped, so nothing
            // may be marked as delivered.
            return Ok(Plan::prefer_snapshot(wire_len, changed_pixels));
        }

        // The reference now describes exactly what an up-to-date viewer
        // holds: current's pixels under every op's rectangle, whatever the
        // op was.
        for op in &ops {
            let (x, y, w, h) = op.rect();
            splice_rect(reference, &current.data, width, x, y, w, h);
        }

        Ok(Plan {
            ops,
            wire_len,
            changed_pixels,
        })
    }

    /// Uniform region -> Fill; anything else -> lossless patch. A solid
    /// rectangle is the one case where a general-purpose byte compressor is
    /// clearly beaten by a 24-byte message.
    fn represent(&self, change: &PixelChange) -> WireOp {
        if let Some(color) = uniform_color(change) {
            return WireOp::Fill {
                x: change.x,
                y: change.y,
                width: change.width,
                height: change.height,
                color,
            };
        }
        WireOp::from(change.clone())
    }

    /// Search vertical then horizontal displacements for one that explains
    /// the churn, and verify it byte-for-byte before it is used.
    fn find_scroll(
        &self,
        reference: &[u8],
        current: &Frame,
        width: u32,
        height: u32,
    ) -> Option<Displacement> {
        let grid = Grid::new(width, height, self.detector.block_size());
        if grid.cols < 2 && grid.rows < 2 {
            return None;
        }
        let prev_hashes = tile_hashes_of(reference, width, height, grid);
        let curr_hashes = tile_hashes(current, grid);
        let limit_tiles = SCROLL_SHIFT_LIMIT / grid.block_size;

        // Vertical first: page scroll, terminal scroll, fling. Both
        // directions, because scrolling down and scrolling up are the
        // same displacement with opposite signs.
        let row_span = limit_tiles.min(grid.rows.saturating_sub(1));
        for magnitude in 1..=row_span {
            for sign in [1i64, -1] {
                if let Some(found) = self.check_shift(
                    reference,
                    &current.data,
                    width,
                    height,
                    grid,
                    &prev_hashes,
                    &curr_hashes,
                    0,
                    sign * magnitude as i64,
                ) {
                    return Some(found);
                }
            }
        }
        // Then horizontal: side-by-side panes, wide carousels.
        let col_span = limit_tiles.min(grid.cols.saturating_sub(1));
        for magnitude in 1..=col_span {
            for sign in [1i64, -1] {
                if let Some(found) = self.check_shift(
                    reference,
                    &current.data,
                    width,
                    height,
                    grid,
                    &prev_hashes,
                    &curr_hashes,
                    sign * magnitude as i64,
                    0,
                ) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// One candidate displacement: the overlap region plus whether the
    /// tiles hash-agree and the pixels actually verify.
    #[allow(clippy::too_many_arguments)]
    fn check_shift(
        &self,
        reference: &[u8],
        current: &[u8],
        width: u32,
        height: u32,
        grid: Grid,
        prev_hashes: &[u64],
        curr_hashes: &[u64],
        dx: i64,
        dy: i64,
    ) -> Option<Displacement> {
        let shift_x = dx * grid.block_size as i64;
        let shift_y = dy * grid.block_size as i64;

        // The destination region is the overlap of the frame with itself
        // translated by (dx, dy). For a positive shift the content moved
        // up, so the covered rows are [0, h - shift); for a negative one
        // they are [-shift, h).
        let x0 = (-shift_x).max(0) as u32;
        let y0 = (-shift_y).max(0) as u32;
        let x1 = ((width as i64 - shift_x).min(width as i64)).max(0) as u32;
        let y1 = ((height as i64 - shift_y).min(height as i64)).max(0) as u32;
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let region = (x0, y0, x1 - x0, y1 - y0);

        // Hash agreement proposes; it never proves.
        let mut matched = 0u32;
        let mut total = 0u32;
        for ty in (y0 / grid.block_size)..((y1 - 1) / grid.block_size + 1) {
            for tx in (x0 / grid.block_size)..((x1 - 1) / grid.block_size + 1) {
                let src = (ty as i64 + dy) * grid.cols as i64 + tx as i64 + dx;
                if src < 0 || src >= prev_hashes.len() as i64 {
                    continue;
                }
                let dst = ty * grid.cols + tx;
                if dst as usize >= curr_hashes.len() {
                    continue;
                }
                total += 1;
                if prev_hashes[src as usize] == curr_hashes[dst as usize] {
                    matched += 1;
                }
            }
        }
        if total < SCROLL_MIN_TILES || (matched as f64) < total as f64 * SCROLL_MATCH_THRESHOLD {
            return None;
        }

        // Proof, not a hint.
        if !verify_displacement_of(reference, current, width, height, shift_x, shift_y, region) {
            return None;
        }
        Some(Displacement {
            region,
            shift: (shift_x, shift_y),
        })
    }
}

/// The single colour of a rectangle, if it has one.
fn uniform_color(change: &PixelChange) -> Option<[u8; 3]> {
    let first = change.data.get(..BYTES_PER_PIXEL)?;
    let color = [first[0], first[1], first[2]];
    change
        .data
        .as_chunks::<BYTES_PER_PIXEL>()
        .0
        .iter()
        .all(|px| *px == color)
        .then_some(color)
}

/// `changes` minus a rectangle, clipped into at most four pieces: the
/// band above, the band below, and the left and right pieces of the band
/// the hole actually occupies.
fn subtract_region(changes: &[PixelChange], hole: (u32, u32, u32, u32)) -> Vec<PixelChange> {
    let (hx, hy, hw, hh) = hole;
    let hx1 = hx + hw;
    let hy1 = hy + hh;
    let mut out = Vec::with_capacity(changes.len());
    for c in changes {
        let (x, y, w, h) = (c.x, c.y, c.width, c.height);
        let (x1, y1) = (x + w, y + h);

        // Entirely inside the hole: the copy already sends these pixels.
        if x >= hx && x1 <= hx1 && y >= hy && y1 <= hy1 {
            continue;
        }
        // No overlap at all: nothing to clip.
        if x >= hx1 || x1 <= hx || y >= hy1 || y1 <= hy {
            out.push(c.clone());
            continue;
        }

        // Band above the hole.
        let top_end = y1.min(hy);
        if top_end > y {
            out.push(crop(c, 0, 0, w, top_end - y));
        }
        // Band below the hole.
        let bottom_start = y.max(hy1);
        if y1 > bottom_start {
            out.push(crop(c, 0, bottom_start - y, w, y1 - bottom_start));
        }
        // The middle band, left and right of the hole.
        let mid_y = y.max(hy);
        let mid_y1 = y1.min(hy1);
        if mid_y1 > mid_y {
            let left_end = x1.min(hx);
            if left_end > x {
                out.push(crop(c, 0, mid_y - y, left_end - x, mid_y1 - mid_y));
            }
            let right_start = x.max(hx1);
            if x1 > right_start {
                out.push(crop(
                    c,
                    right_start - x,
                    mid_y - y,
                    x1 - right_start,
                    mid_y1 - mid_y,
                ));
            }
        }
    }
    out
}

/// A copy of `change` restricted to a sub-rectangle, carrying the same
/// pixels.
fn crop(change: &PixelChange, dx: u32, dy: u32, w: u32, h: u32) -> PixelChange {
    debug_assert!(w > 0 && h > 0);
    let row_bytes = w as usize * BYTES_PER_PIXEL;
    let mut data = Vec::with_capacity(row_bytes * h as usize);
    for row in 0..h as usize {
        let start = (dy + row as u32) as usize * change.width as usize * BYTES_PER_PIXEL
            + dx as usize * BYTES_PER_PIXEL;
        data.extend_from_slice(&change.data[start..start + row_bytes]);
    }
    PixelChange {
        x: change.x + dx,
        y: change.y + dy,
        width: w,
        height: h,
        data,
    }
}

fn splice_rect(dst: &mut [u8], src: &[u8], width: u32, x: u32, y: u32, w: u32, h: u32) {
    let row_bytes = w as usize * BYTES_PER_PIXEL;
    for row in 0..h as usize {
        let offset = ((y as usize + row) * width as usize + x as usize) * BYTES_PER_PIXEL;
        let from = ((y as usize + row) * width as usize + x as usize) * BYTES_PER_PIXEL;
        dst[offset..offset + row_bytes].copy_from_slice(&src[from..from + row_bytes]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::OP_HEADER_BYTES;
    use crate::pcc::compositor::Compositor;
    use std::time::SystemTime;

    fn frame(w: u32, h: u32, data: Vec<u8>) -> Frame {
        Frame {
            id: 0,
            pts_us: 0,
            timestamp: SystemTime::now(),
            width: w,
            height: h,
            data,
        }
    }

    fn blank(w: u32, h: u32) -> Frame {
        frame(w, h, vec![0; (w * h) as usize * 3])
    }

    fn px(f: &mut Frame, x: u32, y: u32, rgb: [u8; 3]) {
        let i = ((y * f.width + x) * 3) as usize;
        f.data[i..i + 3].copy_from_slice(&rgb);
    }

    fn limits() -> PlanLimits {
        PlanLimits {
            snapshot_bytes: 8 * 1024 * 1024,
            max_update_bytes: crate::network::MAX_MESSAGE_SIZE as usize,
        }
    }

    /// Give a viewer the starting surface, apply a plan the way the
    /// compositor would, and return what it would be showing.
    fn replay(base: &Frame, epoch: u32, ops: &[WireOp]) -> Vec<u8> {
        let mut c = Compositor::new();
        let (_, data) =
            crate::encoder::encode_snapshot(base.width, base.height, &base.data).unwrap();
        c.begin_snapshot(
            epoch,
            base.width,
            base.height,
            crate::encoder::SnapshotFormat::Png,
            data.len() as u32,
            1,
        )
        .unwrap();
        c.push_snapshot_chunk(0, &data).unwrap();
        c.commit_snapshot(0, epoch).unwrap();
        c.apply_ops(1, epoch, ops).expect("plan must apply cleanly");
        c.buffer().to_vec()
    }

    #[test]
    fn idle_frame_produces_nothing() {
        let (w, h) = (64u32, 64u32);
        let mut reference = blank(w, h);
        let current = blank(w, h);
        let plan = Planner::default()
            .plan(&mut reference.data, w, h, &current, limits())
            .unwrap();
        assert!(plan.ops.is_empty());
        assert_eq!(plan.wire_len, 0);
    }

    #[test]
    fn uniform_region_becomes_a_fill() {
        let (w, h) = (64u32, 64u32);
        let mut reference = blank(w, h);
        let mut current = blank(w, h);
        for y in 40..56 {
            for x in 8..24 {
                px(&mut current, x, y, [0, 0, 200]);
            }
        }
        let plan = Planner::default()
            .plan(&mut reference.data, w, h, &current, limits())
            .unwrap();
        assert_eq!(plan.ops.len(), 1);
        assert_eq!(
            plan.ops[0],
            WireOp::Fill {
                x: 8,
                y: 40,
                width: 16,
                height: 16,
                color: [0, 0, 200]
            }
        );
        assert_eq!(plan.wire_len, OP_HEADER_BYTES + 3);
    }

    /// Text on a textured background: many small non-uniform rectangles.
    /// Every one must survive the round trip, and the viewer's pixels must
    /// match the captured frame exactly.
    #[test]
    fn scattered_text_like_changes_reconstruct_exactly() {
        let (w, h) = (200u32, 120u32);
        let mut reference = blank(w, h);
        for y in 0..h {
            for x in 0..w {
                px(&mut reference, x, y, [(x / 9) as u8, (y / 9) as u8, 40]);
            }
        }
        let base = reference.clone();
        let mut current = reference.clone();
        for i in 0..300u32 {
            let x = (i * 37) % w;
            let y = (i * 53) % h;
            px(&mut current, x, y, [255, 255, 255]);
        }
        let plan = Planner::default()
            .plan(&mut reference.data, w, h, &current, limits())
            .unwrap();
        assert!(!plan.ops.is_empty());
        let shown = replay(&base, 0, &plan.ops);
        assert_eq!(
            shown, current.data,
            "viewer pixels must equal captured pixels"
        );
        assert_eq!(
            reference.data, current.data,
            "the reference must track exactly what was sent"
        );
    }

    /// The problem the scroll work exists for: without a Copy, a 16px scroll
    /// rewrites almost the whole screen.
    #[test]
    fn a_scroll_becomes_a_copy_and_still_reconstructs_exactly() {
        let bs = 16u32;
        let (w, h) = (128u32, 96u32);
        let mut reference = blank(w, h);
        for y in 0..h {
            for x in 0..w {
                px(&mut reference, x, y, [(x / 8) as u8, (y / 8) as u8, 200]);
            }
        }
        let base = reference.clone();
        let shift = bs;
        let mut current = blank(w, h);
        for y in shift..h {
            for x in 0..w {
                let si = (((y - shift) * w + x) * 3) as usize;
                let di = ((y * w + x) * 3) as usize;
                current.data[di..di + 3].copy_from_slice(&reference.data[si..si + 3]);
            }
        }

        let planner = Planner::new(0, bs).unwrap();
        let plan = planner
            .plan(&mut reference.data, w, h, &current, limits())
            .expect("plan");
        assert!(
            plan.ops.iter().any(WireOp::is_copy),
            "a real scroll must produce a Copy, got {:?}",
            plan.ops
        );
        let shown = replay(&base, 0, &plan.ops);
        assert_eq!(shown, current.data, "a scroll must still be pixel-exact");
        assert_eq!(reference.data, current.data);
    }

    #[test]
    fn scroll_search_can_be_switched_off() {
        let bs = 16u32;
        let (w, h) = (128u32, 96u32);
        let mut reference = blank(w, h);
        for y in 0..h {
            for x in 0..w {
                px(&mut reference, x, y, [(x / 8) as u8, (y / 8) as u8, 200]);
            }
        }
        let mut current = blank(w, h);
        for y in bs..h {
            for x in 0..w {
                let si = (((y - bs) * w + x) * 3) as usize;
                let di = ((y * w + x) * 3) as usize;
                current.data[di..di + 3].copy_from_slice(&reference.data[si..si + 3]);
            }
        }
        let mut planner = Planner::new(0, bs).unwrap();
        planner.set_scroll_search(false);
        let plan = planner
            .plan(&mut reference.data, w, h, &current, limits())
            .unwrap();
        assert!(
            !plan.ops.iter().any(WireOp::is_copy),
            "the search must be genuinely disabled"
        );
    }

    #[test]
    fn oversized_patch_set_is_recommended_as_a_snapshot() {
        let (w, h) = (64u32, 64u32);
        let mut reference = blank(w, h);
        let mut current = blank(w, h);
        for y in 0..64 {
            for x in 0..64 {
                px(
                    &mut current,
                    x,
                    y,
                    [(x * 7) as u8, (y * 13) as u8, (x ^ y) as u8],
                );
            }
        }
        let before = reference.data.clone();
        let plan = Planner::default()
            .plan(
                &mut reference.data,
                w,
                h,
                &current,
                PlanLimits {
                    snapshot_bytes: 16,
                    max_update_bytes: crate::network::MAX_MESSAGE_SIZE as usize,
                },
            )
            .unwrap();
        assert!(plan.ops.is_empty(), "must fall back to a snapshot");
        assert!(
            plan.wire_len > 16,
            "the measured cost must still be reported"
        );
        assert_eq!(
            reference.data, before,
            "a plan that ships nothing must not advance the reference"
        );
    }

    #[test]
    fn subtract_region_clips_a_straddling_rect() {
        let c = PixelChange {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
            data: vec![7; 300],
        };
        let pieces = subtract_region(std::slice::from_ref(&c), (4, 0, 2, 10));
        let total: u32 = pieces.iter().map(|p| p.width * p.height).sum();
        assert_eq!(total, 80, "10x10 minus a 2x10 column is 8x10");
        for p in &pieces {
            assert!(
                p.x >= 4 || p.x + p.width <= 4,
                "piece overlaps the hole it should exclude"
            );
        }
    }
}
