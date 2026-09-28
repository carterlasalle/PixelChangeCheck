//! One immutable screen, shared by every consumer.
//!
//! Publishing swaps a pointer. Readers take an `Arc` and keep it as long
//! as they need. The JPEG preview is derived at most once per
//! (frame, quality) pair and then shared, so opening a fifth browser tab
//! costs a socket, not a JPEG encode of the whole screen.

use crate::encoder::encode_jpeg;
use crate::pcc::types::Frame;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug)]
struct State {
    width: u32,
    height: u32,
    rgb: Arc<Vec<u8>>,
    /// Quality the cached preview was encoded at, as raw float bits.
    quality_bits: u32,
    generation: u64,
    preview: Option<(u64, Arc<Vec<u8>>)>,
}

#[derive(Debug)]
pub struct SharedSurface {
    state: Mutex<State>,
}

impl SharedSurface {
    pub fn new(width: u32, height: u32, quality: f32) -> Self {
        Self {
            state: Mutex::new(State {
                width,
                height,
                rgb: Arc::new(Vec::new()),
                quality_bits: quality.to_bits(),
                generation: 0,
                preview: None,
            }),
        }
    }

    /// Install a freshly captured frame. A frame with the same geometry and
    /// identical bytes does not advance the generation, so an idle screen
    /// does not invalidate anybody's cached preview.
    pub fn publish(&self, frame: &Frame, quality: f32) {
        let mut state = self.state.lock();
        let same = state.width == frame.width
            && state.height == frame.height
            && state.rgb.len() == frame.data.len()
            && state.rgb.as_ref() == frame.data.as_slice();
        if same {
            if state.quality_bits != quality.to_bits() {
                // Same pixels, different preview quality: the cached JPEG
                // is stale even though the frame is not.
                state.quality_bits = quality.to_bits();
                state.preview = None;
            }
            return;
        }
        state.width = frame.width;
        state.height = frame.height;
        state.rgb = Arc::new(frame.data.clone());
        state.quality_bits = quality.to_bits();
        state.generation += 1;
    }

    pub fn dimensions(&self) -> Option<(u32, u32)> {
        let state = self.state.lock();
        (!state.rgb.is_empty()).then_some((state.width, state.height))
    }

    /// The current frame, shared rather than copied.
    pub fn snapshot(&self) -> Option<Arc<Vec<u8>>> {
        let state = self.state.lock();
        (!state.rgb.is_empty()).then(|| state.rgb.clone())
    }

    /// The current frame as a lossy JPEG preview. Explicitly *not* the
    /// authoritative surface: it exists for `<img src=...>`, which cannot
    /// composite patches.
    pub fn jpeg_preview(&self) -> Result<Option<Arc<Vec<u8>>>> {
        let mut state = self.state.lock();
        if state.rgb.is_empty() {
            return Ok(None);
        }
        let generation = state.generation;
        if let Some((cached_gen, data)) = &state.preview {
            if *cached_gen == generation {
                return Ok(Some(data.clone()));
            }
        }
        let quality = f32::from_bits(state.quality_bits);
        let encoded = encode_jpeg(state.width, state.height, quality, &state.rgb)?;
        let encoded = Arc::new(encoded);
        state.preview = Some((generation, encoded.clone()));
        Ok(Some(encoded))
    }
}
