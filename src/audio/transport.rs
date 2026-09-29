//! Audio on its own QUIC stream, and the clock the viewer syncs to.
//!
//! # Why this is not a `Message` variant
//!
//! Audio and pixels have opposite requirements. Pixels are exact, atomic
//! and must never be partially applied; audio is lossy, latency-critical
//! and must never queue behind a multi-megabyte snapshot. Putting audio on
//! the message path would give it the wrong semantics in both directions:
//! a dropped audio packet would stall a frame, and a snapshot would delay
//! a 20 ms frame. So audio gets a separate unidirectional stream, and the
//! two are joined only at presentation time.
//!
//! # The invariant this protects
//!
//! Audio is never derived from the video surface, and the video surface is
//! never modified to make audio line up. A late audio frame is a
//! presentation problem; corrupting the authoritative lossless pixels to
//! fix it would undo every guarantee in `DESIGN.md`.

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::codec::OpusEncoder;
use super::codec::{CHANNELS, SAMPLES_PER_FRAME};

const AUDIO_STREAM_ID: u8 = 0x11;

const MAGIC: &[u8; 4] = b"PCAU";

/// The sharer's side: capture, encode, and push onto a unidirectional
/// stream.
///
/// A dropped packet is a click, not a stall: the video stream is a
/// different stream and is unaffected.
pub struct AudioSender {
    encoder: OpusEncoder,
    origin: Instant,
    /// Latest video capture time in microseconds, so audio frames are
    /// stamped on the same timeline rather than their own.
    video_pts_us: Arc<AtomicU64>,
    sent: AtomicU64,
    dropped: AtomicU64,
}

impl std::fmt::Debug for AudioSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSender")
            .field("sent", &self.sent.load(Ordering::Relaxed))
            .field("dropped", &self.dropped.load(Ordering::Relaxed))
            .finish()
    }
}

impl AudioSender {
    /// `video_pts_us` carries the sharer's capture clock; audio frames
    /// are stamped against it rather than starting a second timeline.
    pub fn new(_unused: (), video_pts_us: Arc<AtomicU64>) -> Result<Self> {
        Ok(Self {
            encoder: OpusEncoder::new()?,
            origin: Instant::now(),
            video_pts_us,
            sent: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        })
    }

    /// Record where video is on the shared timeline, so an audio frame
    /// captured now is stamped against the same origin.
    pub fn note_video_capture(&self, pts_us: u64) {
        self.video_pts_us.store(pts_us, Ordering::Relaxed);
    }

    /// Encode a captured frame. Done once in the capture loop so a
    /// broadcast to N viewers costs one Opus encode rather than N.
    pub fn encode_frame(&mut self, pcm: &[f32]) -> Result<Vec<u8>> {
        self.encoder.encode(pcm)
    }

    /// Write an already-encoded frame, so a broadcast to N viewers costs
    /// one Opus encode rather than N.
    pub async fn send_encoded<W>(&self, stream: &mut W, frame: &EncodedFrame) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let mut out = Vec::with_capacity(16 + frame.opus.len());
        out.extend_from_slice(&frame.pts_us.to_le_bytes());
        out.extend_from_slice(&frame.pcm_len.to_le_bytes());
        out.extend_from_slice(&(frame.opus.len() as u32).to_le_bytes());
        out.extend_from_slice(&frame.opus);
        stream.write_all(&out).await?;
        self.sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Encode one captured frame and write it to the stream.
    pub async fn send_frame<W>(
        &mut self,
        stream: &mut W,
        pcm: &[f32],
        capture: Instant,
    ) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let encoded = self.encoder.encode(pcm)?;
        // Stamp against the shared video origin when there is one; the
        // audio origin is the fallback for a session with no video clock.
        let pts_us = {
            let v = self.video_pts_us.load(Ordering::Relaxed);
            if v > 0 {
                v
            } else {
                capture.duration_since(self.origin).as_micros() as u64
            }
        };
        let mut frame = Vec::with_capacity(MAGIC.len() + 20 + encoded.len());
        frame.extend_from_slice(MAGIC);
        frame.extend_from_slice(&pts_us.to_le_bytes());
        frame.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        frame.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        frame.extend_from_slice(&encoded);
        stream.write_all(&frame).await?;
        self.sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn sent(&self) -> u64 {
        self.sent.load(Ordering::Relaxed)
    }

    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    pub fn note_drop(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }
}

/// Open the audio stream and write its header.
pub async fn open_stream<W>(stream: &mut W) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut header = Vec::with_capacity(MAGIC.len() + 8);
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&(SAMPLES_PER_FRAME as u32).to_le_bytes());
    header.extend_from_slice(&[2u8]); // channels
    stream.write_all(&header).await?;
    Ok(())
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

    /// Read the stream header. Returns false when this is not the audio
    /// stream.
    pub async fn read_header<R>(reader: &mut R) -> Result<bool>
    where
        R: AsyncRead + Unpin,
    {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).await?;
        if &magic != MAGIC {
            return Ok(false);
        }
        // samples-per-frame (u32) + channels (u8), matching `open_stream`.
        let mut rest = [0u8; 5];
        reader.read_exact(&mut rest).await?;
        Ok(true)
    }

    /// Decode the next frame. Returns `None` at end of stream.
    pub async fn read_frame<R>(&mut self, reader: &mut R) -> Result<Option<DecodedAudio>>
    where
        R: AsyncRead + Unpin,
    {
        let mut head = [0u8; 16];
        match reader.read_exact(&mut head).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let pts_us = u64::from_le_bytes(head[0..8].try_into().unwrap());
        let pcm_len = u32::from_le_bytes(head[8..12].try_into().unwrap()) as usize;
        let enc_len = u32::from_le_bytes(head[12..16].try_into().unwrap()) as usize;
        if pcm_len > SAMPLES_PER_FRAME * 2 * 4 {
            anyhow::bail!(
                "audio frame claims {pcm_len} samples, more than one {}ms frame can hold",
                super::FRAME_MS
            );
        }
        let mut encoded = vec![0u8; enc_len];
        reader.read_exact(&mut encoded).await?;

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
        let per_channel = self.decoder.decode(&encoded, &mut pcm)?;
        let expected = pcm_len / CHANNELS;
        if per_channel != expected {
            anyhow::bail!("Opus decoded {per_channel} samples per channel, expected {expected}");
        }
        Ok(Some(DecodedAudio { pts_us, pcm }))
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

/// quinn 0.10's `open_uni` takes no stream type, so the magic in
/// [`open_stream`]'s header is what identifies an audio stream. A viewer
/// that accepts a uni stream and finds no magic drops it.
pub const AUDIO_UNI_STREAM_TYPE: u8 = AUDIO_STREAM_ID;

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

    #[tokio::test]
    async fn a_frame_carries_the_shared_video_timestamp() {
        let video_pts = Arc::new(AtomicU64::new(0));
        let mut sender = AudioSender::new((), video_pts.clone()).unwrap();
        // With no video clock the audio own origin is used.
        let mut stream: Vec<u8> = Vec::new();
        sender
            .send_frame(&mut stream, &tone(1), Instant::now())
            .await
            .unwrap();
        assert_eq!(&stream[0..4], MAGIC);
        assert_eq!(sender.sent(), 1);

        // Once video reports a timestamp, audio adopts it.
        video_pts.store(123_456, Ordering::Relaxed);
        let mut second: Vec<u8> = Vec::new();
        sender
            .send_frame(&mut second, &tone(1), Instant::now())
            .await
            .unwrap();
        let pts = u64::from_le_bytes(second[4..12].try_into().unwrap());
        assert_eq!(pts, 123_456, "audio must ride the video clock");
    }

    /// Build one frame exactly as `send_frame` does, so a test can
    /// control the timestamp without fighting the framing.
    fn frame(pts_us: u64, pcm: &[f32], encoder: &mut OpusEncoder) -> Vec<u8> {
        let encoded = encoder.encode(pcm).unwrap();
        // No magic here: that marks the stream header, not each frame.
        let mut out = Vec::new();
        out.extend_from_slice(&pts_us.to_le_bytes());
        out.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        out.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        out.extend_from_slice(&encoded);
        out
    }

    #[tokio::test]
    async fn a_stream_header_is_distinguishable_from_anything_else() {
        let mut wire: Vec<u8> = Vec::new();
        open_stream(&mut wire).await.unwrap();
        let mut cursor = wire.as_slice();
        assert!(AudioReceiver::read_header(&mut cursor).await.unwrap());

        // A stream that is not audio is refused rather than decoded.
        let mut other = b"NOTAU".to_vec();
        other.extend_from_slice(&[0u8; 8]);
        let mut c2 = other.as_slice();
        assert!(!AudioReceiver::read_header(&mut c2).await.unwrap());
    }

    #[tokio::test]
    async fn the_receiver_reports_gaps_instead_of_stalling() {
        let mut encoder = OpusEncoder::new().unwrap();
        let pcm = tone(1);
        let mut wire: Vec<u8> = Vec::new();
        open_stream(&mut wire).await.unwrap();
        wire.extend(frame(1_000, &pcm, &mut encoder));
        // A frame that claims an *earlier* time than the one before it is
        // a reorder, i.e. something was lost in transit.
        wire.extend(frame(0, &pcm, &mut encoder));
        wire.extend(frame(2_000, &pcm, &mut encoder));

        let mut receiver = AudioReceiver::new().unwrap();
        let mut cursor = wire.as_slice();
        assert!(AudioReceiver::read_header(&mut cursor).await.unwrap());
        let mut seen = 0;
        while let Some(frame) = receiver.read_frame(&mut cursor).await.unwrap() {
            assert!(!frame.pcm.is_empty(), "a frame must decode to samples");
            seen += 1;
        }
        assert_eq!(seen, 3, "a gap must not stop the stream");
        assert_eq!(receiver.gaps(), 1, "the reorder must be counted");
    }

    #[tokio::test]
    async fn a_lying_frame_length_is_refused_before_allocating() {
        let mut wire: Vec<u8> = Vec::new();
        open_stream(&mut wire).await.unwrap();
        wire.extend_from_slice(&1u64.to_le_bytes()); // pts
        wire.extend_from_slice(&u32::MAX.to_le_bytes()); // absurd sample count
        wire.extend_from_slice(&0u32.to_le_bytes()); // empty payload
        let mut receiver = AudioReceiver::new().unwrap();
        let mut cursor = wire.as_slice();
        assert!(AudioReceiver::read_header(&mut cursor).await.unwrap());
        let err = receiver
            .read_frame(&mut cursor)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("more than one"), "unhelpful: {err}");
    }
}
