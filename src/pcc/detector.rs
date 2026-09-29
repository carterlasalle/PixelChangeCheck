//! Pixel Change Check: the coarse-to-fine scan that turns two frames into
//! exact replacement rectangles.
//!
//! The previous implementation copied both sides of every tile into two
//! fresh `Vec`s before comparing them -- 4,080 temporary allocations per
//! 1080p frame, ~122k/s at 30 fps. Nothing here allocates during the scan:
//! rows are compared in place out of the two frame buffers, the changed
//! bounds are accumulated in registers, and the only bytes that are ever
//! copied are the ones that actually ship.

use super::types::{Frame, PixelChange, PixelChangeDetector};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::VecDeque;

pub const BYTES_PER_PIXEL: usize = 3;

/// How much a merged rectangle may grow before it stops being worth its own
/// message. A rect costs ~24 bytes of header plus an LZ4 frame; the pixels it
/// absorbs past the dirty set are the cost of merging. 2.0x keeps the
/// absorbed area at most equal to the area we were going to send anyway,
/// which is the point where extra headers stop being the dominant term.
pub const MERGE_EXPANSION_LIMIT: f64 = 2.0;

#[derive(Debug, Default)]
struct Scratch {
    /// One entry per tile: is it dirty, and what are its changed bounds.
    dirty: Vec<bool>,
    bounds: Vec<Rect>,
    /// Flood-fill bookkeeping for the merge pass.
    seen: Vec<bool>,
    queue: VecDeque<usize>,
    component: Vec<usize>,
    out: Vec<Rect>,
}

impl Scratch {
    fn resize(&mut self, tiles: usize) {
        if self.dirty.len() != tiles {
            self.dirty.resize(tiles, false);
            self.bounds.resize(tiles, Rect::EMPTY);
            self.seen.resize(tiles, false);
        }
    }
}

#[derive(Debug)]
pub struct PCCDetector {
    threshold: u8,
    block_size: u32,
    /// Reused across calls so a scan of a static screen allocates
    /// nothing at all. `RefCell`, because a detector is owned by one
    /// capture task and never shared.
    scratch: RefCell<Scratch>,
}

impl Default for PCCDetector {
    fn default() -> Self {
        // threshold 5: safe now that the sender diffs against the framebuffer
        // the viewer actually holds (see app::share), so a sub-threshold
        // step is not discarded -- it accumulates until it crosses the
        // threshold against what the viewer is missing.
        Self {
            threshold: 5,
            block_size: 32,
            scratch: RefCell::new(Scratch::default()),
        }
    }
}

impl PCCDetector {
    pub fn new(threshold: u8, block_size: u32) -> Result<Self> {
        if block_size == 0 {
            anyhow::bail!("block_size must be > 0, got {block_size}");
        }
        Ok(Self {
            threshold,
            block_size,
            scratch: RefCell::new(Scratch::default()),
        })
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    #[inline]
    fn differs(&self, a: u8, b: u8) -> bool {
        (a as i16 - b as i16).unsigned_abs() > self.threshold as u16
    }

    /// True when any byte of this tile row differs past the threshold.
    /// The slice comparison is a memcmp: identical rows (the overwhelmingly
    /// common case on a static desktop) leave the byte loop untouched.
    #[inline]
    fn row_changed(&self, prev: &[u8], curr: &[u8]) -> bool {
        if prev == curr {
            return false;
        }
        if self.threshold == 0 {
            return true; // slice compare already proved they differ
        }
        prev.iter()
            .zip(curr.iter())
            .any(|(p, c)| self.differs(*p, *c))
    }
}

/// Grid geometry shared by the scan, the merge pass and the hash pass.
#[derive(Debug, Clone, Copy)]
pub struct Grid {
    pub block_size: u32,
    pub cols: u32,
    pub rows: u32,
}

impl Grid {
    pub fn new(width: u32, height: u32, block_size: u32) -> Self {
        let cols = width.div_ceil(block_size);
        let rows = height.div_ceil(block_size);
        Self {
            block_size,
            cols,
            rows,
        }
    }

    pub fn tile(&self, width: u32, height: u32, tx: u32, ty: u32) -> (u32, u32) {
        let x = tx * self.block_size;
        let y = ty * self.block_size;
        (x.min(width), y.min(height))
    }
}

impl PCCDetector {
    /// Scan `current` against `previous` and return the exact rectangles
    /// that turn one into the other.
    ///
    /// Takes plain buffers rather than `Frame`s: a sharer holds the
    /// authoritative surface behind an `Arc` and must be able to publish
    /// it to a joining viewer without copying the whole screen every
    /// frame. An id and a timestamp are not part of that decision.
    pub fn detect(
        &self,
        previous: &[u8],
        current: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<PixelChange>> {
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }
        let expected = (width as usize * height as usize) * BYTES_PER_PIXEL;
        if previous.len() != expected || current.len() != expected {
            anyhow::bail!(
                "frame buffer length mismatch: {width}x{height} needs {expected} bytes, \
                 got previous={} current={}",
                previous.len(),
                current.len()
            );
        }

        let grid = Grid::new(width, height, self.block_size);
        let row_stride = width as usize * BYTES_PER_PIXEL;
        // Reused across calls, so a scan of a static screen allocates
        // nothing at all.
        let mut scratch = self.scratch.borrow_mut();
        scratch.resize((grid.cols * grid.rows) as usize);
        scratch.dirty.iter_mut().for_each(|d| *d = false);

        // Pass 1: which tiles changed, and the exact changed-pixel bounds
        // inside each. Reads both frames in place; allocates nothing.
        for ty in 0..grid.rows {
            for tx in 0..grid.cols {
                let (x0, y0) = grid.tile(width, height, tx, ty);
                let w = self.block_size.min(width - x0);
                let h = self.block_size.min(height - y0);

                let mut rect = Rect::EMPTY;
                'tile: for row in 0..h {
                    let start = (y0 + row) as usize * row_stride + x0 as usize * BYTES_PER_PIXEL;
                    let end = start + w as usize * BYTES_PER_PIXEL;
                    let prev_row = &previous[start..end];
                    let curr_row = &current[start..end];
                    if !self.row_changed(prev_row, curr_row) {
                        continue;
                    }
                    for (dx, (p, c)) in prev_row.iter().zip(curr_row.iter()).enumerate() {
                        if self.differs(*p, *c) {
                            let px = x0 + dx as u32 / BYTES_PER_PIXEL as u32;
                            rect.absorb(px, y0 + row);
                        }
                    }
                    // Fast out: a tile touching all four of its edges cannot
                    // grow further, and this is the common case for
                    // full-screen motion.
                    if rect.spans_tile(x0, y0, w, h) {
                        break 'tile;
                    }
                }

                if !rect.is_empty() {
                    let idx = (ty * grid.cols + tx) as usize;
                    scratch.dirty[idx] = true;
                    scratch.bounds[idx] = rect;
                }
            }
        }

        if !scratch.dirty.iter().any(|d| *d) {
            return Ok(Vec::new());
        }

        // Pass 2: merge 4-connected dirty tiles into the smallest number
        // of rectangles that still cost less to send than the pieces.
        let Scratch {
            dirty,
            bounds,
            seen,
            queue,
            component,
            out,
        } = &mut *scratch;
        out.clear();
        seen.iter_mut().for_each(|s| *s = false);
        for start in 0..dirty.len() {
            if !dirty[start] || seen[start] {
                continue;
            }
            component.clear();
            queue.clear();
            queue.push_back(start);
            seen[start] = true;
            while let Some(idx) = queue.pop_front() {
                component.push(idx);
                let tx = (idx as u32) % grid.cols;
                let ty = (idx as u32) / grid.cols;
                for (nx, ny) in [
                    (tx.wrapping_sub(1), ty),
                    (tx + 1, ty),
                    (tx, ty.wrapping_sub(1)),
                    (tx, ty + 1),
                ] {
                    if nx >= grid.cols || ny >= grid.rows {
                        continue;
                    }
                    let nidx = (ny * grid.cols + nx) as usize;
                    if dirty[nidx] && !seen[nidx] {
                        seen[nidx] = true;
                        queue.push_back(nidx);
                    }
                }
            }

            let mut union = Rect::EMPTY;
            let mut piece_area = 0u64;
            for &idx in component.iter() {
                let r = bounds[idx];
                piece_area += r.area();
                union = union.union(r);
            }

            if component.len() > 1
                && union.area() as f64 > piece_area as f64 * MERGE_EXPANSION_LIMIT
            {
                // Merging would drag in more unchanged area than it saves.
                for &idx in component.iter() {
                    out.push(bounds[idx]);
                }
            } else {
                out.push(union);
            }
        }

        // Copy only the rectangles that ship.
        let mut changes = Vec::with_capacity(out.len());
        for &r in out.iter() {
            changes.push(r.extract(current, row_stride));
        }
        Ok(changes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32, // exclusive
}

impl Rect {
    const EMPTY: Rect = Rect {
        x0: u32::MAX,
        y0: u32::MAX,
        x1: 0,
        y1: 0,
    };

    #[inline]
    fn is_empty(&self) -> bool {
        self.x0 >= self.x1 || self.y0 >= self.y1
    }

    #[inline]
    fn absorb(&mut self, x: u32, y: u32) {
        self.x0 = self.x0.min(x);
        self.y0 = self.y0.min(y);
        self.x1 = self.x1.max(x + 1);
        self.y1 = self.y1.max(y + 1);
    }

    #[inline]
    fn union(self, other: Rect) -> Rect {
        if other.is_empty() {
            return self;
        }
        Rect {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    #[inline]
    fn area(&self) -> u64 {
        if self.is_empty() {
            return 0;
        }
        (self.x1 - self.x0) as u64 * (self.y1 - self.y0) as u64
    }

    #[inline]
    fn spans_tile(&self, x0: u32, y0: u32, w: u32, h: u32) -> bool {
        self.x0 == x0 && self.y0 == y0 && self.x1 == x0 + w && self.y1 == y0 + h
    }

    fn extract(&self, frame_data: &[u8], row_stride: usize) -> PixelChange {
        let width = self.x1 - self.x0;
        let height = self.y1 - self.y0;
        let row_bytes = width as usize * BYTES_PER_PIXEL;
        let mut data = Vec::with_capacity(row_bytes * height as usize);
        for row in 0..height as usize {
            let start = (self.y0 as usize + row) * row_stride + self.x0 as usize * BYTES_PER_PIXEL;
            data.extend_from_slice(&frame_data[start..start + row_bytes]);
        }
        PixelChange {
            x: self.x0,
            y: self.y0,
            width,
            height,
            data,
        }
    }
}

impl PixelChangeDetector for PCCDetector {
    fn detect_changes(&self, previous: &Frame, current: &Frame) -> Result<Vec<PixelChange>> {
        self.detect(
            &previous.data,
            &current.data,
            previous.width,
            previous.height,
        )
    }

    fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold;
    }

    fn set_block_size(&mut self, block_size: u32) -> Result<()> {
        if block_size == 0 {
            anyhow::bail!("block_size must be > 0, got {block_size}");
        }
        self.block_size = block_size;
        Ok(())
    }
}

/// Total changed area as a fraction of the frame, for the planner's
/// full-frame fallback decision.
pub fn changed_fraction(changes: &[PixelChange], width: u32, height: u32) -> f64 {
    let total = width as u64 * height as u64;
    if total == 0 {
        return 0.0;
    }
    let changed: u64 = changes.iter().map(|c| c.pixels()).sum();
    changed as f64 / total as f64
}

/// Splice `change` (already sent) into the authoritative reference so the
/// reference keeps describing exactly what an up-to-date viewer holds.
pub fn splice(reference: &mut [u8], width: u32, change: &PixelChange) {
    change.copy_from(&change.data, width, reference, width);
}

/// Hash every tile of `frame` once, so a displacement search compares 64-bit
/// integers instead of 3 KiB of pixels. Equal hashes are a *candidate*, never
/// a proof -- callers must verify with [`verify_displacement`].
pub fn tile_hashes(frame: &Frame, grid: Grid) -> Vec<u64> {
    tile_hashes_of(&frame.data, frame.width, frame.height, grid)
}

/// [`tile_hashes`] over a plain buffer.
pub fn tile_hashes_of(data: &[u8], width: u32, height: u32, grid: Grid) -> Vec<u64> {
    let row_stride = width as usize * BYTES_PER_PIXEL;
    let mut hashes = vec![0u64; (grid.cols * grid.rows) as usize];
    for ty in 0..grid.rows {
        for tx in 0..grid.cols {
            let (x0, y0) = grid.tile(width, height, tx, ty);
            let w = grid.block_size.min(width - x0);
            let h = grid.block_size.min(height - y0);
            // Eight bytes at a time. A byte-at-a-time FNV loop over two
            // 1080p frames per capture is the most expensive thing in the
            // scroll search, and this is a filter, not a proof: every
            // candidate it proposes is verified byte-for-byte afterwards.
            let mut hash = 0xcbf29ce484222325u64;
            for row in 0..h {
                let start = (y0 + row) as usize * row_stride + x0 as usize * BYTES_PER_PIXEL;
                let row_bytes = &data[start..start + w as usize * BYTES_PER_PIXEL];
                let mut chunks = row_bytes.chunks_exact(8);
                for c in &mut chunks {
                    let word = u64::from_le_bytes(c.try_into().expect("chunks_exact(8)"));
                    hash = (hash ^ word).wrapping_mul(0x9E3779B97F4A7C15);
                    hash ^= hash >> 29;
                }
                for &b in chunks.remainder() {
                    hash = (hash ^ b as u64).wrapping_mul(0x100000001b3);
                }
            }
            hashes[(ty * grid.cols + tx) as usize] = hash;
        }
    }
    hashes
}

/// Byte-for-byte check that `previous` displaced by (dx, dy) really does
/// reproduce the corresponding region of `current`. This is what separates a
/// hash match (a search shortcut) from a verified copy.
pub fn verify_displacement(
    previous: &Frame,
    current: &Frame,
    dx: i64,
    dy: i64,
    region: (u32, u32, u32, u32),
) -> bool {
    verify_displacement_of(
        &previous.data,
        &current.data,
        previous.width,
        previous.height,
        dx,
        dy,
        region,
    )
}

/// [`verify_displacement`] over plain buffers.
#[allow(clippy::too_many_arguments)]
pub fn verify_displacement_of(
    previous: &[u8],
    current: &[u8],
    width: u32,
    height: u32,
    dx: i64,
    dy: i64,
    region: (u32, u32, u32, u32),
) -> bool {
    let (rx, ry, rw, rh) = region;
    if rw == 0 || rh == 0 {
        return false;
    }
    // Every sampled pixel must land inside both frames.
    let xs = [rx as i64, rx as i64 + rw as i64 - 1];
    let ys = [ry as i64, ry as i64 + rh as i64 - 1];
    for &x in &xs {
        for &y in &ys {
            let (sx, sy) = (x + dx, y + dy);
            if sx < 0 || sy < 0 || sx >= width as i64 || sy >= height as i64 {
                return false;
            }
        }
    }

    let row_bytes = rw as usize * BYTES_PER_PIXEL;
    for row in 0..rh as i64 {
        let dst_start = ((ry as i64 + row) * width as i64 + rx as i64) as usize * BYTES_PER_PIXEL;
        let src_start =
            (((ry as i64 + row + dy) * width as i64) + rx as i64 + dx) as usize * BYTES_PER_PIXEL;
        if previous[src_start..src_start + row_bytes] != current[dst_start..dst_start + row_bytes] {
            return false;
        }
    }
    true
}

/// Apply every `change` to `dst` in one pass. The caller has already
/// validated the whole set, so a malformed member can never land halfway
/// through a frame.
pub fn apply_changes(dst: &mut [u8], width: u32, changes: &[PixelChange]) {
    for change in changes {
        change.copy_from(&change.data, width, dst, width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn set_px(f: &mut Frame, x: u32, y: u32, rgb: [u8; 3]) {
        let i = ((y * f.width + x) * 3) as usize;
        f.data[i..i + 3].copy_from_slice(&rgb);
    }

    #[test]
    fn single_pixel_change_is_exact() {
        let a = blank(8, 8);
        let mut b = a.clone();
        set_px(&mut b, 3, 2, [0, 200, 0]);
        let d = PCCDetector::default();
        let changes = d.detect(&a.data, &b.data, a.width, a.height).unwrap();
        assert_eq!(changes.len(), 1);
        let c = &changes[0];
        assert_eq!((c.x, c.y, c.width, c.height), (3, 2, 1, 1));
        assert_eq!(c.data, vec![0, 200, 0]);
    }

    #[test]
    fn sub_threshold_drift_is_reported_against_the_reference() {
        // 100 -> 103 -> 106: each step is 3, under the threshold of 5, so
        // step 2 alone changes nothing. Step 3 crosses it against the
        // frame the viewer still holds.
        let d = PCCDetector::new(5, 8).unwrap();
        let mut base = blank(8, 8);
        set_px(&mut base, 0, 0, [100, 100, 100]);
        let mut step2 = base.clone();
        set_px(&mut step2, 0, 0, [103, 103, 103]);
        assert!(d.detect(&base.data, &step2.data, 8, 8).unwrap().is_empty());

        let mut step3 = base.clone();
        set_px(&mut step3, 0, 0, [106, 106, 106]);
        let changes = d.detect(&base.data, &step3.data, 8, 8).unwrap();
        assert_eq!(
            changes.len(),
            1,
            "drift must accumulate against the reference"
        );
        assert_eq!(changes[0].data, vec![106, 106, 106]);
    }

    #[test]
    fn merging_collapses_a_contiguous_block() {
        // A 64x32 slab is two 32x32 tiles; they must ship as one rect.
        let a = blank(64, 32);
        let mut b = a.clone();
        for y in 0..32 {
            for x in 0..64 {
                set_px(&mut b, x, y, [200, 30, 30]);
            }
        }
        let d = PCCDetector::new(0, 32).unwrap();
        let changes = d.detect(&a.data, &b.data, a.width, a.height).unwrap();
        assert_eq!(
            changes.len(),
            1,
            "expected one merged rect, got {changes:?}"
        );
        let c = &changes[0];
        assert_eq!((c.x, c.y, c.width, c.height), (0, 0, 64, 32));
        assert_eq!(c.data.len(), 64 * 32 * 3);
    }

    #[test]
    fn merge_is_declined_when_it_would_absorb_mostly_unchanged_area() {
        // Two dirty tiles diagonal from each other: a single bounding box
        // would be four tiles, 3x the dirty area.
        let a = blank(64, 64);
        let mut b = a.clone();
        set_px(&mut b, 0, 0, [255, 0, 0]);
        set_px(&mut b, 63, 63, [255, 0, 0]);
        let d = PCCDetector::new(0, 32).unwrap();
        let changes = d.detect(&a.data, &b.data, a.width, a.height).unwrap();
        assert_eq!(changes.len(), 2, "diagonal tiles must not be merged");
    }

    #[test]
    fn changes_reconstruct_current_exactly() {
        let a = blank(120, 90);
        let mut b = a.clone();
        // scattered single pixels
        for i in 0..200u32 {
            set_px(&mut b, (i * 7) % 120, (i * 13) % 90, [i as u8, 9, 200]);
        }
        let d = PCCDetector::new(0, 16).unwrap();
        let changes = d.detect(&a.data, &b.data, a.width, a.height).unwrap();
        assert!(!changes.is_empty());
        let mut recon = a.data.clone();
        apply_changes(&mut recon, a.width, &changes);
        assert_eq!(
            recon, b.data,
            "rectangles must reconstruct the frame exactly"
        );
    }

    #[test]
    fn dimension_mismatch_is_an_error_not_a_panic() {
        let d = PCCDetector::default();
        let a = blank(8, 8);
        let b = blank(8, 9);
        assert!(d.detect(&a.data, &b.data, 8, 8).is_err());
    }

    #[test]
    fn short_buffer_is_an_error_not_a_panic() {
        let d = PCCDetector::default();
        let short = frame(8, 8, vec![0; 10]);
        let a = blank(8, 8);
        assert!(d.detect(&a.data, &short.data, 8, 8).is_err());
    }

    #[test]
    fn tile_hash_and_verify_confirm_a_real_scroll() {
        // 4x3 tiles of 16px. Scroll the content up by exactly one tile.
        let bs = 16u32;
        let (w, h) = (64u32, 48u32);
        let mut a = blank(w, h);
        for y in 0..h {
            for x in 0..w {
                set_px(&mut a, x, y, [x as u8, y as u8, 7]);
            }
        }
        let mut b = a.clone();
        for y in bs..h {
            for x in 0..w {
                let s = (((y - bs) * w + x) * 3) as usize;
                let d = ((y * w + x) * 3) as usize;
                b.data[d..d + 3].copy_from_slice(&a.data[s..s + 3]);
            }
        }
        let grid = Grid::new(w, h, bs);
        let ha = tile_hashes(&a, grid);
        let hb = tile_hashes(&b, grid);
        assert_ne!(ha, hb);
        // b[y] == a[y - bs], so the source is `bs` rows *above* the
        // destination: the displacement is negative.
        // Destination rows [bs, h) read from source rows [0, h-bs).
        assert!(
            verify_displacement(&a, &b, 0, -(bs as i64), (0, bs, w, h - bs)),
            "a one-tile scroll must verify"
        );
        // A displacement that is not real must not verify.
        assert!(!verify_displacement(&a, &b, 0, -5, (0, bs, w, h - bs)));
    }
}
