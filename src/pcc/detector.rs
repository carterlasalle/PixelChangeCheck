use super::types::{Frame, PixelChange, PixelChangeDetector, QualityConfig};
use anyhow::Result;

/// Frame data is stored as tightly packed RGB triples.
const BYTES_PER_PIXEL: u32 = 3;

pub struct PCCDetector {
    config: QualityConfig,
    threshold: u8,
    block_size: u32,
}

impl Default for PCCDetector {
    fn default() -> Self {
        Self {
            config: QualityConfig::default(),
            threshold: 5,  // Default difference threshold
            block_size: 32, // Size of blocks to compare
        }
    }
}

impl PCCDetector {
    /// Create a new PCCDetector with custom configuration
    pub fn new(config: QualityConfig, threshold: u8, block_size: u32) -> Self {
        Self {
            config,
            threshold,
            block_size,
        }
    }

    /// Compare two blocks of pixels using direct comparison
    #[inline]
    fn compare_blocks(&self, prev: &[u8], curr: &[u8]) -> bool {
        debug_assert_eq!(prev.len(), curr.len(), "Block sizes must match");
        
        // Compare bytes directly
        for (p, c) in prev.iter().zip(curr.iter()) {
            if (*p as i16 - *c as i16).abs() > self.threshold as i16 {
                return true;
            }
        }
        
        false
    }

    /// Find the bounds of changed region in a block.
    /// `width`/`height` are in pixels; `prev`/`curr` are RGB byte buffers.
    fn find_change_bounds(&self, prev: &[u8], curr: &[u8], width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found_change = false;

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * BYTES_PER_PIXEL) as usize;
                let pixel_changed = (0..BYTES_PER_PIXEL as usize).any(|c| {
                    (prev[idx + c] as i16 - curr[idx + c] as i16).abs() > self.threshold as i16
                });
                if pixel_changed {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                    found_change = true;
                }
            }
        }

        if found_change {
            Some((min_x, min_y, max_x + 1, max_y + 1))
        } else {
            None
        }
    }
}

impl PixelChangeDetector for PCCDetector {
    fn detect_changes(&self, previous: &Frame, current: &Frame) -> Result<Vec<PixelChange>> {
        if previous.width != current.width || previous.height != current.height {
            anyhow::bail!("Frame dimensions do not match");
        }

        let mut changes = Vec::new();
        let width = previous.width;
        let height = previous.height;
        
        // Byte stride of a full frame row (RGB)
        let row_stride = (width * BYTES_PER_PIXEL) as usize;

        // Process frame in blocks
        for y in (0..height).step_by(self.block_size as usize) {
            for x in (0..width).step_by(self.block_size as usize) {
                let block_width = std::cmp::min(self.block_size, width - x);
                let block_height = std::cmp::min(self.block_size, height - y);
                let block_row_bytes = (block_width * BYTES_PER_PIXEL) as usize;

                // Extract blocks from both frames (byte-accurate, RGB stride)
                let prev_block: Vec<u8> = (0..block_height)
                    .flat_map(|dy| {
                        let start = ((y + dy) as usize) * row_stride + (x as usize * BYTES_PER_PIXEL as usize);
                        let end = start + block_row_bytes;
                        previous.data[start..end].iter().copied()
                    })
                    .collect();

                let curr_block: Vec<u8> = (0..block_height)
                    .flat_map(|dy| {
                        let start = ((y + dy) as usize) * row_stride + (x as usize * BYTES_PER_PIXEL as usize);
                        let end = start + block_row_bytes;
                        current.data[start..end].iter().copied()
                    })
                    .collect();

                // Compare blocks
                if self.compare_blocks(&prev_block, &curr_block) {
                    // Find exact bounds of the change within the block (pixel coordinates)
                    if let Some((min_x, min_y, max_x, max_y)) =
                        self.find_change_bounds(&prev_block, &curr_block, block_width, block_height) {

                        let change_width = max_x - min_x;
                        let change_height = max_y - min_y;
                        let change_row_bytes = (change_width * BYTES_PER_PIXEL) as usize;

                        // Extract changed region (byte-accurate, RGB stride)
                        let mut change_data = Vec::with_capacity((change_width * change_height * BYTES_PER_PIXEL) as usize);
                        for dy in min_y..max_y {
                            let start = (dy as usize) * block_row_bytes + (min_x as usize * BYTES_PER_PIXEL as usize);
                            let end = start + change_row_bytes;
                            change_data.extend_from_slice(&curr_block[start..end]);
                        }

                        changes.push(PixelChange {
                            x: x + min_x,
                            y: y + min_y,
                            width: change_width,
                            height: change_height,
                            data: change_data,
                        });
                    }
                }
            }
        }

        Ok(changes)
    }

    fn configure(&mut self, config: QualityConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
} 