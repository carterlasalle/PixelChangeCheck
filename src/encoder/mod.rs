//! Encoding and decoding of the authoritative surface, plus the JPEG path
//! that only the browser preview uses.
//!
//! The authoritative stream is **lossless**. That is the whole reason a
//! viewer can be asserted pixel-exact against the sender: a JPEG keyframe
//! silently changes the pixels it is supposed to describe, and no amount of
//! careful diffing downstream can undo that. JPEG is still here, but only as
//! the explicitly-lossy web preview, where nobody claims exactness.

use crate::pcc::types::rgb_len;
use anyhow::{Context, Result};
use image::codecs::png::{CompressionType, PngEncoder};
use image::io::Reader as ImageReader;
use image::ImageEncoder;
use std::io::Cursor;
use tracing::debug;

/// How the authoritative snapshot's pixels are carried on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SnapshotFormat {
    /// Tightly packed RGB, uncompressed. Only ever used for small frames
    /// where compression would not pay.
    Raw = 0,
    /// LZ4 with a prepended decoded size.
    Lz4 = 1,
    /// PNG. Lossless, and decoded natively by every browser, so the
    /// browser compositor needs no image decoder at all.
    Png = 2,
}

impl From<SnapshotFormat> for u8 {
    fn from(f: SnapshotFormat) -> u8 {
        f as u8
    }
}

impl SnapshotFormat {
    pub fn from_u8(v: u8) -> Result<Self> {
        Ok(match v {
            0 => SnapshotFormat::Raw,
            1 => SnapshotFormat::Lz4,
            2 => SnapshotFormat::Png,
            other => anyhow::bail!("Unknown snapshot format: {other}"),
        })
    }
}

/// Tripwire on a compressed snapshot's length, before it is handed to a
/// decompressor. A snapshot of a frame we could legally hold can never be
/// larger than the raw surface plus a PNG container's per-row overhead.
pub const MAX_SNAPSHOT_BYTES: usize = crate::pcc::types::MAX_FRAME_BYTES + 64 * 1024;

/// Encode a full RGB surface into a lossless snapshot.
///
/// PNG with fast deflate: the encode runs off the capture hot path (a
/// snapshot is emitted on join, on repair, and on a periodic deadline), and
/// at deflate level 1 a 1080p desktop screen encodes in tens of
/// milliseconds at a fraction of level 6's size for the same losslessness.
pub fn encode_snapshot(width: u32, height: u32, rgb: &[u8]) -> Result<(SnapshotFormat, Vec<u8>)> {
    let expected = rgb_len(width, height)?;
    if rgb.len() != expected {
        anyhow::bail!(
            "snapshot source is {width}x{height} ({expected} bytes) but got {}",
            rgb.len()
        );
    }
    let mut out = Vec::new();
    PngEncoder::new_with_quality(
        Cursor::new(&mut out),
        CompressionType::Fast,
        image::codecs::png::FilterType::Adaptive,
    )
    .write_image(rgb, width, height, image::ColorType::Rgb8)
    .context("Failed to encode PNG snapshot")?;
    Ok((SnapshotFormat::Png, out))
}

/// Decode a snapshot back to exact RGB, bounded by the geometry declared
/// alongside it.
pub fn decode_snapshot(
    format: SnapshotFormat,
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<u8>> {
    let expected = rgb_len(width, height)?;
    match format {
        SnapshotFormat::Raw => {
            if data.len() != expected {
                anyhow::bail!(
                    "raw snapshot is {width}x{height} ({expected} bytes) but got {}",
                    data.len()
                );
            }
            Ok(data.to_vec())
        }
        SnapshotFormat::Lz4 => compression::decompress_exact(data, expected),
        SnapshotFormat::Png => {
            // Read the PNG's own header first: a 10 KiB file that declares
            // 20000x20000 must be rejected before it is decoded.
            let reader = ImageReader::new(Cursor::new(data))
                .with_guessed_format()
                .context("snapshot is not a recognised image")?;
            let (iw, ih) = reader
                .into_dimensions()
                .context("Failed to read snapshot dimensions")?;
            if iw != width || ih != height {
                anyhow::bail!("snapshot declares {iw}x{ih} but the update says {width}x{height}");
            }
            let rgb_len_check = rgb_len(iw, ih)?;
            if rgb_len_check > expected {
                anyhow::bail!("decoded snapshot budget exceeded: {rgb_len_check} > {expected}");
            }
            // Re-open for the decode: `into_dimensions` consumes the
            // reader, and the header check above is what bounds it.
            let img = ImageReader::new(Cursor::new(data))
                .with_guessed_format()
                .context("snapshot is not a recognised image")?
                .decode()
                .context("Failed to decode PNG snapshot")?
                .to_rgb8();
            if img.width() != width || img.height() != height {
                anyhow::bail!("decoded snapshot is {}x{}", img.width(), img.height());
            }
            Ok(img.into_raw())
        }
    }
}

/// LZ4 for small exact regions. Fast by design: latency beats ratio when
/// the alternative is a whole extra frame.
pub mod compression {
    use super::*;
    use lz4_flex::block::{compress_prepend_size, decompress_into};

    pub fn compress_frame(frame: &[u8]) -> Result<Vec<u8>> {
        let start = std::time::Instant::now();
        let compressed = compress_prepend_size(frame);
        debug!(
            "Region compressed: {} -> {} bytes in {:?}, ratio: {:.2}:1",
            frame.len(),
            compressed.len(),
            start.elapsed(),
            frame.len() as f32 / compressed.len() as f32
        );
        Ok(compressed)
    }

    /// Decompress into a buffer of exactly `expected` bytes.
    ///
    /// This is the bound that matters: the caller derived `expected` from
    /// the geometry it already validated, so the decompressor can never be
    /// talked into allocating an arbitrary amount, and a payload that
    /// decodes to a different length is rejected instead of being applied.
    pub fn decompress_exact(compressed: &[u8], expected: usize) -> Result<Vec<u8>> {
        // `compress_prepend_size` writes a 4-byte little-endian decoded
        // size ahead of the block. Read it and check it against the
        // geometry *before* touching the decompressor: that is the check
        // that stops a compression bomb, and it is also what
        // `decompress_into` cannot do for us (it takes the raw block
        // format, without the prefix).
        if compressed.len() < 4 {
            anyhow::bail!("LZ4 payload too short: {} bytes", compressed.len());
        }
        let declared = u32::from_le_bytes(compressed[..4].try_into().unwrap()) as usize;
        if declared != expected {
            anyhow::bail!(
                "LZ4 payload declares {declared} decoded bytes, expected {expected} \
                 (width*height*3 mismatch)"
            );
        }
        let mut out = vec![0u8; expected];
        let n = decompress_into(&compressed[4..], &mut out)
            .map_err(|e| anyhow::anyhow!("LZ4 payload is corrupt: {e}"))?;
        if n != expected {
            anyhow::bail!("LZ4 decoded to {n} bytes, expected {expected}");
        }
        Ok(out)
    }
}

/// Encode raw RGB24 pixel data to JPEG. Only the browser preview uses this,
/// and it is explicitly *not* the authoritative surface.
pub fn encode_jpeg(width: u32, height: u32, quality: f32, frame: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let quality = (quality.clamp(0.0, 1.0) * 100.0) as u8;
    jpeg_encoder::Encoder::new(&mut output, quality).encode(
        frame,
        width as u16,
        height as u16,
        jpeg_encoder::ColorType::Rgb,
    )?;
    Ok(output)
}

/// A snapshot of the framebuffer plus the revision it represents, published
/// once per update for every consumer (window viewer, web preview, repair).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSnapshot {
    pub width: u32,
    pub height: u32,
    /// Shared, immutable, so N viewers cost one copy, not N.
    pub rgb: std::sync::Arc<Vec<u8>>,
}

impl SurfaceSnapshot {
    pub fn new(width: u32, height: u32, rgb: Vec<u8>) -> Result<Self> {
        let expected = rgb_len(width, height)?;
        if rgb.len() != expected {
            anyhow::bail!(
                "snapshot is {width}x{height} ({expected} bytes) but got {}",
                rgb.len()
            );
        }
        Ok(Self {
            width,
            height,
            rgb: std::sync::Arc::new(rgb),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 3) as usize);
        for y in 0..h {
            for x in 0..w {
                v.push((x % 256) as u8);
                v.push((y % 256) as u8);
                v.push(((x + y) % 256) as u8);
            }
        }
        v
    }

    #[test]
    fn png_snapshot_roundtrips_exactly() {
        let rgb = gradient(64, 48);
        let (fmt, data) = encode_snapshot(64, 48, &rgb).unwrap();
        assert_eq!(fmt, SnapshotFormat::Png);
        let decoded = decode_snapshot(fmt, 64, 48, &data).unwrap();
        assert_eq!(decoded, rgb, "PNG snapshot must be lossless");
    }

    #[test]
    fn raw_snapshot_roundtrips_exactly() {
        let rgb = gradient(8, 8);
        let decoded = decode_snapshot(SnapshotFormat::Raw, 8, 8, &rgb).unwrap();
        assert_eq!(decoded, rgb);
    }

    #[test]
    fn lz4_snapshot_roundtrips_exactly() {
        let rgb = gradient(64, 48);
        let c = compression::compress_frame(&rgb).unwrap();
        let decoded = decode_snapshot(SnapshotFormat::Lz4, 64, 48, &c).unwrap();
        assert_eq!(decoded, rgb);
    }

    #[test]
    fn lz4_decode_into_wrong_size_is_rejected() {
        let rgb = gradient(64, 48);
        let c = compression::compress_frame(&rgb).unwrap();
        // Ask for a surface one byte different: the decode must refuse
        // rather than hand back a buffer the caller would blit blindly.
        let err = compression::decompress_exact(&c, rgb.len() - 1)
            .unwrap_err()
            .to_string();
        assert!(err.contains("declares"), "unhelpful: {err}");
    }

    #[test]
    fn lz4_decode_rejects_a_truncated_or_corrupt_block() {
        let rgb = gradient(64, 48);
        let c = compression::compress_frame(&rgb).unwrap();
        assert!(compression::decompress_exact(&c[..2], rgb.len()).is_err());
        // A token that claims far more literals than the block holds.
        let mut corrupt = compression::compress_frame(&[1, 2, 3]).unwrap();
        corrupt[4] = 0xFF;
        assert!(compression::decompress_exact(&corrupt, 3).is_err());
    }

    #[test]
    fn snapshot_lying_about_its_dimensions_is_rejected() {
        let rgb = gradient(64, 48);
        let (fmt, data) = encode_snapshot(64, 48, &rgb).unwrap();
        let err = decode_snapshot(fmt, 128, 96, &data)
            .unwrap_err()
            .to_string();
        assert!(err.contains("declares 64x48"), "unhelpful: {err}");
    }

    #[test]
    fn corrupt_payload_is_an_error_not_a_panic() {
        assert!(decode_snapshot(SnapshotFormat::Lz4, 4, 4, &[0, 1, 2, 3]).is_err());
        assert!(decode_snapshot(SnapshotFormat::Png, 4, 4, b"not a png").is_err());
        assert!(decode_snapshot(SnapshotFormat::Raw, 4, 4, &[0; 3]).is_err());
    }

    #[test]
    fn oversized_geometry_is_rejected_with_both_numbers() {
        let err = decode_snapshot(SnapshotFormat::Raw, 30_000, 30_000, &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("max_frame_bytes"), "unhelpful: {err}");
    }
}
