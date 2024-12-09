use super::types::{Frame, PixelChange, PixelChangeDetector, QualityConfig};
use anyhow::{Context, Result};

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

    /// Find the bounds of changed region in a block
    fn find_change_bounds(&self, prev: &[u8], curr: &[u8], width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found_change = false;

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                if (prev[idx] as i16 - curr[idx] as i16).abs() > self.threshold as i16 {
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
        
        // Process frame in blocks
        for y in (0..height).step_by(self.block_size as usize) {
            for x in (0..width).step_by(self.block_size as usize) {
                let block_width = std::cmp::min(self.block_size, width - x);
                let block_height = std::cmp::min(self.block_size, height - y);
                
                // Extract blocks from both frames
                let prev_block: Vec<u8> = (0..block_height)
                    .flat_map(|dy| {
                        let start = ((y + dy) * width + x) as usize;
                        let end = start + block_width as usize;
                        previous.data[start..end].iter().copied()
                    })
                    .collect();

                let curr_block: Vec<u8> = (0..block_height)
                    .flat_map(|dy| {
                        let start = ((y + dy) * width + x) as usize;
                        let end = start + block_width as usize;
                        current.data[start..end].iter().copied()
                    })
                    .collect();

                // Compare blocks
                if self.compare_blocks(&prev_block, &curr_block) {
                    // Find exact bounds of the change within the block
                    if let Some((min_x, min_y, max_x, max_y)) = 
                        self.find_change_bounds(&prev_block, &curr_block, block_width, block_height) {
                        
                        let change_width = max_x - min_x;
                        let change_height = max_y - min_y;
                        
                        // Extract changed region
                        let mut change_data = Vec::with_capacity((change_width * change_height) as usize);
                        for dy in min_y..max_y {
                            let start = (dy * block_width + min_x) as usize;
                            let end = start + change_width as usize;
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