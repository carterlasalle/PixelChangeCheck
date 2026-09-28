//! Video codecs: what the protocol reserves, and what this build can
//! actually produce.
//!
//! The rule the audits keep repeating is the one this module enforces:
//! **never assume a codec is available; probe it.** "Supports AV1" does
//! not mean a usable real-time configuration exists, and a reserved
//! identifier is not a capability.
//!
//! # Why no encoder is linked
//!
//! This is a recorded finding, not an omission. The two obvious routes to
//! a real AV1 encoder were both tried:
//!
//! * **rav1e 0.6.3** (the pure-Rust encoder) builds, but its
//!   `context/cdf_context.rs` performs an out-of-bounds `slice` access
//!   that Rust 1.97's `unsafe` precondition checks abort on. It is a
//!   latent-UB defect in the dependency, reproducible at any frame size,
//!   so linking it would ship a crash on the capture path.
//! * **rav1e 0.6.6** fixes nothing reachable here: it pins
//!   `clap = "=4.0.32"`, which cannot co-exist with this project's
//!   `clap 4.5`.
//!
//! Every other encoder (x264, libaom, SVT-AV1, VideoToolbox) needs a C
//! or platform SDK dependency, which is a larger decision than this
//! change should make on its own. So the registry ships with the
//! interface, the identifiers, and honest probing, and
//! [`encoder_for`] refuses with the reason rather than returning a stub.
//!
//! When an encoder is added, nothing else has to change: the trait, the
//! identifiers, and the capability negotiation are already here.

use anyhow::Result;

/// A codec identifier. The numbers are wire values and must not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum VideoCodec {
    H264 = 1,
    Hevc = 2,
    Av1 = 3,
    Av2 = 4,
}

/// Every codec the protocol reserves an identifier for, whether or not
/// this build can produce it.
///
/// AV2's identifier is reserved deliberately: its specification is final
/// but no encoder is linked here, and a stable identifier means a future
/// build can add it without a protocol change.
pub const RESERVED: [VideoCodec; 4] = [
    VideoCodec::H264,
    VideoCodec::Hevc,
    VideoCodec::Av1,
    VideoCodec::Av2,
];

impl VideoCodec {
    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn from_id(id: u8) -> Result<Self> {
        RESERVED
            .into_iter()
            .find(|c| c.id() == id)
            .ok_or_else(|| anyhow::anyhow!("Unknown video codec id: {id}"))
    }

    pub fn name(self) -> &'static str {
        match self {
            VideoCodec::H264 => "h264",
            VideoCodec::Hevc => "hevc",
            VideoCodec::Av1 => "av1",
            VideoCodec::Av2 => "av2",
        }
    }

    /// The `VideoDecoder` configuration string a browser needs, for the
    /// codecs a browser can decode.
    pub fn web_codec_string(self) -> &'static str {
        match self {
            // 8-bit 4:2:0, the profile a browser will take without
            // hardware negotiation.
            VideoCodec::Av1 => "av01.0.04M.08",
            VideoCodec::H264 => "avc1.42E01E",
            VideoCodec::Hevc => "hev1.1.6.L93.B0",
            VideoCodec::Av2 => "av02.0.04M.08",
        }
    }
}

/// What one consumer can decode.
///
/// Empty in this build, and that is the honest value: it is what a
/// consumer without a linked decoder reports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    codecs: Vec<VideoCodec>,
}

impl Capabilities {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn of(codecs: impl IntoIterator<Item = VideoCodec>) -> Self {
        let mut list: Vec<VideoCodec> = codecs.into_iter().collect();
        list.sort();
        list.dedup();
        Self { codecs: list }
    }

    pub fn codecs(&self) -> &[VideoCodec] {
        &self.codecs
    }

    pub fn can_decode(&self, codec: VideoCodec) -> bool {
        self.codecs.contains(&codec)
    }

    pub fn is_empty(&self) -> bool {
        self.codecs.is_empty()
    }
}

/// What this build can *produce*, discovered by construction.
///
/// It is empty, because nothing is linked. A reserved identifier is not a
/// capability, and reporting otherwise is the exact failure the audits
/// warn about ("supports AV1" does not mean a usable 4:4:4 real-time
/// configuration exists).
pub fn probe_local() -> Capabilities {
    Capabilities::none()
}

/// Encodes a surface into a self-contained, decodable bitstream.
///
/// Implemented by a linked encoder. There is none in this build; the
/// trait exists so that adding one is a single new module rather than a
/// change to the pipeline.
pub trait VideoEncoder: Send {
    fn codec(&self) -> VideoCodec;

    /// Encode one frame. `keyframe` forces a frame decodable without any
    /// preceding frame, which is what a joiner or a recovery needs.
    fn encode(&mut self, rgb: &[u8], width: u32, height: u32, keyframe: bool) -> Result<Vec<u8>>;

    /// Drop the reference chain. The next frame must be a keyframe.
    fn reset(&mut self);
}

/// Build an encoder for `codec`, or explain why this build cannot.
///
/// The error is the deliverable when nothing is linked: it names the
/// codec, and the pipeline falls back to a lossless snapshot, which is
/// always correct.
pub fn encoder_for(
    codec: VideoCodec,
    _width: u32,
    _height: u32,
    _speed: u8,
) -> Result<Box<dyn VideoEncoder>> {
    anyhow::bail!(
        "No {} encoder is linked into this build. Available encoders: {}. \
         See src/codec/mod.rs for what was tried and why it was rejected.",
        codec.name(),
        if probe_local().is_empty() {
            "none".to_string()
        } else {
            probe_local()
                .codecs()
                .iter()
                .map(|c| c.name())
                .collect::<Vec<_>>()
                .join(", ")
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reserved_identifier_round_trips() {
        for codec in RESERVED {
            assert_eq!(VideoCodec::from_id(codec.id()).unwrap(), codec);
        }
    }

    #[test]
    fn an_unknown_identifier_is_refused() {
        let err = VideoCodec::from_id(99).unwrap_err().to_string();
        assert!(
            err.contains("Unknown video codec id: 99"),
            "unhelpful: {err}"
        );
    }

    #[test]
    fn probing_reports_nothing_because_nothing_is_linked() {
        // The single most important assertion in this file: a reserved
        // identifier must never be mistaken for a capability.
        let caps = probe_local();
        assert!(caps.is_empty());
        for codec in RESERVED {
            assert!(
                !caps.can_decode(codec),
                "{} must not be reported without a linked encoder",
                codec.name()
            );
        }
    }

    #[test]
    fn requesting_an_encoder_fails_with_the_reason() {
        for codec in RESERVED {
            let err = match encoder_for(codec, 64, 64, 8) {
                Ok(_) => panic!("{} must not silently resolve to an encoder", codec.name()),
                Err(e) => e.to_string(),
            };
            assert!(
                err.contains(&format!("No {} encoder is linked", codec.name())),
                "unhelpful for {}: {err}",
                codec.name()
            );
            assert!(err.contains("src/codec/mod.rs"), "unhelpful: {err}");
        }
    }

    #[test]
    fn capabilities_can_be_declared_even_though_none_are_produced() {
        // A browser advertises what it can decode; the registry must be
        // able to carry that without inventing an encoder.
        let caps = Capabilities::of([VideoCodec::Av1, VideoCodec::Av1, VideoCodec::H264]);
        assert_eq!(caps.codecs(), &[VideoCodec::H264, VideoCodec::Av1]);
        assert!(caps.can_decode(VideoCodec::Av1));
        assert!(!caps.can_decode(VideoCodec::Av2));
    }
}
