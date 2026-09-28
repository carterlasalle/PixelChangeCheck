//! The sharer's current screen, shared with every consumer.
//!
//! One immutable RGB buffer behind a pointer swap, so N viewers cost one
//! copy rather than N. The JPEG preview is encoded at most once per
//! (frame, quality) pair and then shared, which is the difference between
//! "one JPEG per shared screen" and "one JPEG per open browser tab".

mod buffer;
pub mod web;

pub use buffer::SharedSurface;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcc::types::Frame;
    use std::sync::Arc;

    fn frame(w: u32, h: u32, seed: u8) -> Frame {
        let mut data = vec![0u8; (w * h * 3) as usize];
        for (i, b) in data.iter_mut().enumerate() {
            *b = seed.wrapping_add(i as u8);
        }
        Frame::new(0, w, h, data).unwrap()
    }

    #[test]
    fn publishing_shares_one_buffer() {
        let s = SharedSurface::new(4, 4, 0.8);
        let f = frame(4, 4, 1);
        s.publish(&f, 0.8);
        let a = s.snapshot().unwrap();
        s.publish(&f, 0.8);
        let b = s.snapshot().unwrap();
        assert!(
            Arc::ptr_eq(&a, &b),
            "republishing identical dimensions must not copy the frame"
        );
    }

    #[test]
    fn jpeg_preview_is_encoded_once_per_frame() {
        let s = SharedSurface::new(8, 8, 0.8);
        s.publish(&frame(8, 8, 2), 0.8);
        let a = s.jpeg_preview().unwrap().unwrap();
        let b = s.jpeg_preview().unwrap().unwrap();
        assert!(
            Arc::ptr_eq(&a, &b),
            "the preview must not be re-encoded per viewer"
        );

        // A different frame invalidates the cache.
        s.publish(&frame(8, 8, 3), 0.8);
        let c = s.jpeg_preview().unwrap().unwrap();
        assert!(!Arc::ptr_eq(&a, &c));
    }

    #[test]
    fn changing_quality_re_encodes_the_preview() {
        let s = SharedSurface::new(8, 8, 0.8);
        s.publish(&frame(8, 8, 4), 0.8);
        let a = s.jpeg_preview().unwrap().unwrap();
        s.publish(&frame(8, 8, 4), 0.4);
        let b = s.jpeg_preview().unwrap().unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn an_empty_surface_has_no_preview() {
        let s = SharedSurface::new(0, 0, 0.8);
        assert!(s.snapshot().is_none());
        assert!(s.jpeg_preview().unwrap().is_none());
    }
}
