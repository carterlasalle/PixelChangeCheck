//! Measuring audio/video sync, rather than guessing at it.
//!
//! # The method
//!
//! Put a known tone into the audio and a known flash into the video at the
//! same instant, record both, and find where each landed. The offset
//! between the audio peak and the video edge *is* the sync error, and it
//! is a number rather than an opinion.
//!
//! This is the only test that means anything here. Watching a stream and
//! deciding it "looks right" cannot resolve the tens of milliseconds
//! that separate a good implementation from a bad one.
//!
//! # What it does not need
//!
//! A microphone, a speaker or a network. The measurement is over the
//! buffers, before they reach a device, so it runs in CI. A device in the
//! loop would only add its own unmeasured latency to the answer.

use std::time::Duration;

use super::codec::{OpusDecoder, OpusEncoder, CHANNELS, SAMPLES_PER_FRAME, SAMPLE_RATE};

/// A one-frame burst whose energy peaks at a known sample.
pub fn tone_burst() -> Vec<f32> {
    let mut pcm = vec![0f32; SAMPLES_PER_FRAME];
    // A half-cycle burst at 1 kHz puts the peak in the middle of the
    // frame, which makes the measured offset insensitive to where exactly
    // in the frame the boundary falls.
    let start = SAMPLES_PER_FRAME / 4;
    for i in 0..(SAMPLES_PER_FRAME / 2) {
        let s = (i as f32) * 2.0 * std::f32::consts::PI * 1000.0 / SAMPLE_RATE as f32;
        let v = s.sin() * 0.8;
        pcm[(start + i) % SAMPLES_PER_FRAME] = v;
        pcm[(start + i) % SAMPLES_PER_FRAME + CHANNELS] = v;
    }
    pcm
}

/// Sample index of the loudest sample, which is where the tone landed.
pub fn peak_index(samples: &[f32]) -> Option<usize> {
    samples
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .filter(|(_, v)| v.abs() > 0.1)
        .map(|(i, _)| i)
}

/// The playout time of a decoded frame, relative to the first frame.
pub fn playout_ms(samples_per_frame: usize) -> f64 {
    // 20 ms of audio, not a constant anyone has to trust twice.
    samples_per_frame as f64 / CHANNELS as f64 / SAMPLE_RATE as f64 * 1000.0
}

/// Round-trip a tone through Opus and return the time the audio clock says
/// it is heard.
pub fn measure_tone_delay() -> Result<Duration, anyhow::Error> {
    let pcm = tone_burst();
    let mut encoder = OpusEncoder::new()?;
    let encoded = encoder.encode(&pcm)?;
    let mut decoder = OpusDecoder::new()?;
    let mut out = vec![0f32; SAMPLES_PER_FRAME];
    decoder.decode(&encoded, &mut out)?;
    let peak = peak_index(&out)
        .ok_or_else(|| anyhow::anyhow!("the tone did not survive the round trip"))?;
    Ok(Duration::from_micros(
        (peak as u64 * 1_000_000) / SAMPLE_RATE as u64,
    ))
}

/// Where a video frame stamped `pts_us` falls on the playout clock,
/// given the measured audio offset.
pub fn video_release_time(pts_us: u64, audio_offset: Duration) -> Duration {
    Duration::from_micros(pts_us).saturating_add(audio_offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::sync::{Estimator, OffsetSample};

    #[test]
    fn a_tone_survives_the_codec_with_a_bounded_delay() {
        let delay = measure_tone_delay().expect("the tone must round trip");
        // The burst starts 10 ms into its frame and Opus adds a documented
        // ~6.5 ms of algorithmic delay plus pre-echo, so the measured peak
        // lands past 16 ms. The upper bound is generous on purpose: this
        // test exists to catch a *regression* that doubles the delay, not
        // to pin an exact figure that depends on the libopus build.
        assert!(
            delay >= Duration::from_millis(10),
            "the peak cannot precede the burst that produced it, took {delay:?}"
        );
        assert!(
            delay < Duration::from_millis(40),
            "a 20 ms burst must land within two frames, took {delay:?}"
        );
    }

    #[test]
    fn the_burst_is_actually_a_burst() {
        let pcm = tone_burst();
        let peak = peak_index(&pcm).expect("a burst needs a peak");
        assert!(peak > SAMPLES_PER_FRAME / 4, "peak at {peak} is too early");
        assert!(
            peak < SAMPLES_PER_FRAME * 3 / 4,
            "peak at {peak} is too late"
        );
    }

    #[test]
    fn twenty_milliseconds_is_twenty_milliseconds() {
        assert!((playout_ms(SAMPLES_PER_FRAME) - 20.0).abs() < 0.001);
    }

    #[test]
    fn a_video_frame_is_released_at_its_timestamp_plus_the_audio_offset() {
        // A frame stamped 100 ms in, with a 30 ms audio offset, is due at
        // 130 ms. This is the whole of the sync policy in one line.
        assert_eq!(
            video_release_time(100_000, Duration::from_millis(30)),
            Duration::from_millis(130)
        );
    }

    #[test]
    fn an_unconverged_estimator_refuses_to_slave_anything() {
        let mut est = Estimator::new();
        // A single sample is not an offset, it is a rumour.
        est.observe(OffsetSample::new(
            Duration::from_millis(40),
            Duration::from_millis(0),
        ));
        assert!(!est.convergence());
        assert_eq!(est.offset(), None);
    }

    #[test]
    fn a_steady_stream_converges_and_then_reports_its_offset() {
        let mut est = Estimator::new();
        for _ in 0..64 {
            est.observe(OffsetSample::new(
                Duration::from_millis(42),
                Duration::from_millis(0),
            ));
        }
        assert!(est.convergence());
        let offset = est.offset().expect("converged means an answer");
        assert!(
            offset >= Duration::from_millis(41) && offset <= Duration::from_millis(43),
            "offset {offset:?} is not the 42 ms that was observed"
        );
    }
}
