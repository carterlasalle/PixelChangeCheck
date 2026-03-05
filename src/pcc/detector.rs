use super::types::{Frame, PixelChange, PixelChangeDetector, QualityConfig};
use anyhow::Result;

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
}

impl PixelChangeDetector for PCCDetector {
    fn detect_changes(&self, previous: &Frame, current: &Frame) -> Result<Vec<PixelChange>> {
        if previous.width != current.width || previous.height != current.height {
            anyhow::bail!("Frame dimensions do not match");
        }

        let mut changes = Vec::new();
        let width = previous.width;
        let height = previous.height;
        let row_stride = width as usize * 3; // 3 bytes per pixel (RGB)
        let threshold = self.threshold as i16;

        // Process frame in blocks
        for by in (0..height).step_by(self.block_size as usize) {
            for bx in (0..width).step_by(self.block_size as usize) {
                let block_w = std::cmp::min(self.block_size, width - bx);
                let block_h = std::cmp::min(self.block_size, height - by);

                // Fast path: skip block if bytes are identical (memcmp is SIMD-optimized)
                let block_identical = (0..block_h).all(|dy| {
                    let offset = (by + dy) as usize * row_stride + bx as usize * 3;
                    let len = block_w as usize * 3;
                    previous.data[offset..offset + len] == current.data[offset..offset + len]
                });

                if block_identical {
                    continue;
                }

                // Single pass: detect changes and find bounds simultaneously
                let mut min_x = block_w;
                let mut min_y = block_h;
                let mut max_x = 0u32;
                let mut max_y = 0u32;
                let mut has_change = false;

                for dy in 0..block_h {
                    let row_offset = (by + dy) as usize * row_stride + bx as usize * 3;
                    let row_len = block_w as usize * 3;
                    let prev_row = &previous.data[row_offset..row_offset + row_len];
                    let curr_row = &current.data[row_offset..row_offset + row_len];

                    for px in 0..block_w {
                        let i = px as usize * 3;
                        // Check if any RGB channel differs beyond threshold
                        if (prev_row[i] as i16 - curr_row[i] as i16).abs() > threshold
                            || (prev_row[i + 1] as i16 - curr_row[i + 1] as i16).abs() > threshold
                            || (prev_row[i + 2] as i16 - curr_row[i + 2] as i16).abs() > threshold
                        {
                            min_x = min_x.min(px);
                            min_y = min_y.min(dy);
                            max_x = max_x.max(px);
                            max_y = max_y.max(dy);
                            has_change = true;
                        }
                    }
                }

                if has_change {
                    let change_w = max_x - min_x + 1;
                    let change_h = max_y - min_y + 1;

                    // Extract changed region's pixel data (RGB)
                    let mut change_data = Vec::with_capacity((change_w * change_h * 3) as usize);
                    for dy in min_y..=max_y {
                        let src = (by + dy) as usize * row_stride + (bx + min_x) as usize * 3;
                        let len = change_w as usize * 3;
                        change_data.extend_from_slice(&current.data[src..src + len]);
                    }

                    changes.push(PixelChange {
                        x: bx + min_x,
                        y: by + min_y,
                        width: change_w,
                        height: change_h,
                        data: change_data,
                    });
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