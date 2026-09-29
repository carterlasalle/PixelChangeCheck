//! Audio on its own QUIC datagrams, and the clock the viewer syncs to.
//!
//! # Why this is not a `Message` variant
//!
//! Audio and pixels have opposite requirements. Pixels are exact, atomic
//! and must never be partially applied; audio is lossy, latency-critical
//! and must never queue behind a multi-megabyte snapshot. Putting audio on
//! the message path would give it the wrong semantics in both directions:
//! a dropped audio packet would stall a frame, and a snapshot would delay
//! a 20 ms frame. So audio gets unreliable datagrams beside the reliable
//! message stream, and the two are joined only at presentation time.
//!
//! # The invariant this protects
//!
//! Audio is never derived from the video surface, and the video surface is
//! never modified to make audio line up. A late audio frame is a
//! presentation problem; corrupting the authoritative lossless pixels to
//! fix it would undo every guarantee in `DESIGN.md`.

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;

use super::codec::OpusEncoder;
use super::codec::{CHANNELS, SAMPLES_PER_FRAME};

/// The sharer's side: capture and encode one Opus frame.
///
/// Audio travels in QUIC **datagrams**, not a stream. A stream is
/// reliable, so a lost audio packet is retransmitted and playback waits
/// for it -- the stall this type is meant to avoid. A datagram is not
/// retransmitted: frame 42 is simply gone, concealed, and 43 arrives on
/// time. The picture stream is unaffected either way.
pub struct AudioSender {
    encoder: OpusEncoder,
}

impl std::fmt::Debug for AudioSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSender").finish_non_exhaustive()
    }
}

impl AudioSender {
    pub fn new() -> Result<Self> {
        Ok(Self {
            encoder: OpusEncoder::new()?,
        })
    }

    /// Encode a captured frame. Done once in the capture loop so a
    /// broadcast to N viewers costs one Opus encode rather than N.
    pub fn encode_frame(&mut self, pcm: &[f32]) -> Result<Vec<u8>> {
        self.encoder.encode(pcm)
    }

    /// Encode one already-captured frame as a datagram payload.
    ///
    /// The payload also refuses an Opus frame that no single path-MTU-sized
    /// datagram could legally contain.
    pub fn datagram_payload(frame: &EncodedFrame) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(16 + frame.opus.len());
        out.extend_from_slice(&frame.pts_us.to_le_bytes());
        out.extend_from_slice(&frame.pcm_len.to_le_bytes());
        out.extend_from_slice(&(frame.opus.len() as u32).to_le_bytes());
        out.extend_from_slice(&frame.opus);
        anyhow::ensure!(
            out.len() <= crate::network::MAX_AUDIO_DATAGRAM,
            "audio frame is {} bytes, larger than one datagram",
            out.len()
        );
        Ok(out)
    }
}

/// Split a datagram payload back into its fields.
///
/// A datagram is a single UDP packet's worth of bytes, so it is either
/// wholly there or wholly gone -- there is no partial frame to resume
/// from, and no length prefix, because the datagram itself is the
/// boundary. Anything malformed is rejected rather than guessed at.
pub fn parse_datagram(payload: &[u8]) -> Result<DatagramFrame<'_>> {
    if payload.len() < 16 {
        anyhow::bail!(
            "audio datagram is {} bytes, too short for a header",
            payload.len()
        );
    }
    let pts_us = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    let pcm_len = u32::from_le_bytes(payload[8..12].try_into().unwrap());
    let opus_len = u32::from_le_bytes(payload[12..16].try_into().unwrap()) as usize;
    let body = payload.get(16..16 + opus_len).ok_or_else(|| {
        anyhow::anyhow!(
            "audio datagram declares {opus_len} bytes of opus but carries {}",
            payload.len().saturating_sub(16)
        )
    })?;
    Ok(DatagramFrame {
        pts_us,
        pcm_len,
        opus: body,
    })
}

/// One audio frame as it arrives in a datagram.
#[derive(Debug, Clone, Copy)]
pub struct DatagramFrame<'a> {
    pub pts_us: u64,
    pub pcm_len: u32,
    pub opus: &'a [u8],
}

/// The viewer's side: read, decode, and hand frames to the playout clock.
pub struct AudioReceiver {
    decoder: super::codec::OpusDecoder,
    seen: u64,
    gaps: u64,
    last_pts_us: u64,
}

impl std::fmt::Debug for AudioReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioReceiver")
            .field("seen", &self.seen)
            .field("gaps", &self.gaps)
            .finish()
    }
}

impl AudioReceiver {
    pub fn new() -> Result<Self> {
        Ok(Self {
            decoder: super::codec::OpusDecoder::new()?,
            seen: 0,
            gaps: 0,
            last_pts_us: 0,
        })
    }

    /// Decode one datagram. There is no end-of-datagram `None`: a missing
    /// datagram is simply never delivered, so this succeeds or is a
    /// malformed frame.
    pub fn decode_datagram(&mut self, payload: &[u8]) -> Result<DecodedAudio> {
        let frame = parse_datagram(payload)?;
        self.decode_frame_fields(frame.pts_us, frame.pcm_len as usize, frame.opus)
    }

    fn decode_frame_fields(
        &mut self,
        pts_us: u64,
        pcm_len: usize,
        encoded: &[u8],
    ) -> Result<DecodedAudio> {
        if pcm_len > SAMPLES_PER_FRAME * 2 * 4 {
            anyhow::bail!(
                "audio frame claims {pcm_len} samples, more than one {}ms frame can hold",
                super::FRAME_MS
            );
        }

        // A frame that arrives out of order means something was lost. That
        // is a click, not a reason to stop.
        if self.last_pts_us > pts_us {
            self.gaps += 1;
        }
        self.last_pts_us = pts_us;
        self.seen += 1;

        // libopus returns samples *per channel*; the buffer is already
        // exactly one interleaved frame, so truncating to that count would
        // silently halve the audio.
        let mut pcm = vec![0f32; pcm_len];
        let per_channel = self.decoder.decode(encoded, &mut pcm)?;
        let expected = pcm_len / CHANNELS;
        if per_channel != expected {
            anyhow::bail!("Opus decoded {per_channel} samples per channel, expected {expected}");
        }
        Ok(DecodedAudio { pts_us, pcm })
    }

    pub fn frames_seen(&self) -> u64 {
        self.seen
    }

    /// Frames that arrived out of order, i.e. were lost in transit.
    pub fn gaps(&self) -> u64 {
        self.gaps
    }
}

/// An encoded frame, ready to hand to any number of viewers.
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub pts_us: u64,
    pub pcm_len: u32,
    pub opus: Arc<Vec<u8>>,
}

/// One decoded audio frame, on the shared timeline.
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub pts_us: u64,
    pub pcm: Vec<f32>,
}

impl DecodedAudio {
    /// Presentation time as a duration, for comparison with the video
    /// playout clock.
    pub fn pts(&self) -> Duration {
        Duration::from_micros(self.pts_us)
    }
}

/// Encode a 20 ms Opus frame, for the sync harness and for tests.
pub fn encode_frame_for_test(pcm: &[f32]) -> Result<Vec<u8>> {
    let mut encoder = OpusEncoder::new().context("test encoder")?;
    encoder.encode(pcm)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize) -> Vec<f32> {
        (0..frames * SAMPLES_PER_FRAME)
            .map(|i| ((i as f32) * 0.01).sin() * 0.2)
            .collect()
    }

    #[test]
    fn an_audio_frame_round_trips_through_opus() {
        let pcm = tone(1);
        let encoded = encode_frame_for_test(&pcm).unwrap();
        let mut decoder = super::super::codec::OpusDecoder::new().unwrap();
        let mut out = vec![0f32; SAMPLES_PER_FRAME];
        let n = decoder.decode(&encoded, &mut out).unwrap();
        assert!(n > 0, "a round trip must produce samples");
    }

    /// Build one datagram exactly as `datagram_payload` does, so a test
    /// can control the timestamp without fighting the framing.
    fn datagram(pts_us: u64, pcm: &[f32], encoder: &mut OpusEncoder) -> Vec<u8> {
        let encoded = encoder.encode(pcm).unwrap();
        let frame = EncodedFrame {
            pts_us,
            pcm_len: pcm.len() as u32,
            opus: Arc::new(encoded),
        };
        AudioSender::datagram_payload(&frame).unwrap()
    }

    #[tokio::test]
    async fn the_receiver_reports_gaps_instead_of_stalling() {
        let mut encoder = OpusEncoder::new().unwrap();
        let pcm = tone(1);
        let mut receiver = AudioReceiver::new().unwrap();
        let mut seen = 0;
        // A frame that claims an *earlier* time than the one before it is
        // a reorder, i.e. something was lost in transit.
        for payload in [
            datagram(1_000, &pcm, &mut encoder),
            datagram(0, &pcm, &mut encoder),
            datagram(2_000, &pcm, &mut encoder),
        ] {
            let frame = receiver.decode_datagram(&payload).unwrap();
            assert!(!frame.pcm.is_empty(), "a frame must decode to samples");
            seen += 1;
        }
        assert_eq!(seen, 3, "a gap must not stop the stream");
        assert_eq!(receiver.gaps(), 1, "the reorder must be counted");
    }

    #[tokio::test]
    async fn a_lying_frame_length_is_refused_before_allocating() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u64.to_le_bytes()); // pts
        payload.extend_from_slice(&u32::MAX.to_le_bytes()); // absurd sample count
        payload.extend_from_slice(&0u32.to_le_bytes()); // empty payload
        let mut receiver = AudioReceiver::new().unwrap();
        let err = receiver.decode_datagram(&payload).unwrap_err().to_string();
        assert!(err.contains("more than one"), "unhelpful: {err}");
    }

    #[tokio::test]
    async fn an_oversize_opus_frame_never_becomes_a_datagram() {
        let frame = EncodedFrame {
            pts_us: 0,
            pcm_len: 0,
            opus: Arc::new(vec![0u8; crate::network::MAX_AUDIO_DATAGRAM]),
        };
        let err = AudioSender::datagram_payload(&frame)
            .unwrap_err()
            .to_string();
        assert!(err.contains("larger than one datagram"), "unhelpful: {err}");
    }
}
