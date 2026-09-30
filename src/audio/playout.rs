#[cfg(feature = "audio")]
use anyhow::{anyhow, Context, Result};
#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use super::codec::SAMPLES_PER_FRAME;
#[cfg(feature = "audio")]
use super::codec::SAMPLE_RATE;

const AUDIO_QUEUE_SAMPLES: usize = SAMPLES_PER_FRAME * 8;

/// A revision scheduled for presentation on the audio clock.
#[derive(Debug)]
pub struct ScheduledRevision<R> {
    pub revision: R,
    pub presentation_time: Instant,
}

/// Schedules video revisions around the audio output device.
///
/// **Audio is the master; video is slaved to it.** The device's buffer and
/// hardware latency define presentation "now", so callers pass the instant
/// observed by the output callback to [`Self::take_due`]. A late video frame is
/// held or dropped by the caller; an audio glitch must never stall video. This
/// scheduler never edits the authoritative pixel surface: sync happens only at
/// presentation.
pub struct Playout<R> {
    capacity: usize,
    revisions: VecDeque<ScheduledRevision<R>>,
    dropped_oldest: u64,
    audio: Arc<Mutex<AudioQueue>>,
    audio_now: Arc<Mutex<Option<Instant>>>,
    #[cfg(feature = "audio")]
    output: Option<cpal::Stream>,
}

impl<R> Playout<R> {
    /// Creates a bounded scheduler. `capacity` must be nonzero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Playout capacity must be greater than zero");
        Self {
            capacity,
            revisions: VecDeque::with_capacity(capacity),
            dropped_oldest: 0,
            audio: Arc::new(Mutex::new(AudioQueue::default())),
            audio_now: Arc::new(Mutex::new(None)),
            #[cfg(feature = "audio")]
            output: None,
        }
    }

    /// Inserts a revision in presentation-time order. A full buffer drops its
    /// oldest presentation time because stale video is less valuable than fresh.
    pub fn schedule(&mut self, revision: R, presentation_time: Instant) {
        let entry = ScheduledRevision {
            revision,
            presentation_time,
        };
        let index = self
            .revisions
            .iter()
            .position(|queued| queued.presentation_time > presentation_time)
            .unwrap_or(self.revisions.len());
        self.revisions.insert(index, entry);
        if self.revisions.len() > self.capacity {
            self.revisions.pop_front();
            self.dropped_oldest = self.dropped_oldest.saturating_add(1);
        }
    }

    /// Takes every revision whose presentation time has arrived on the audio
    /// clock. The returned revisions are strictly presentation-time ordered.
    pub fn take_due(&mut self, audio_now: Instant) -> Vec<R> {
        let mut due = Vec::new();
        while self
            .revisions
            .front()
            .is_some_and(|entry| entry.presentation_time <= audio_now)
        {
            if let Some(entry) = self.revisions.pop_front() {
                due.push(entry.revision);
            }
        }
        due
    }

    /// Alias for code that describes the audio clock as `now`.
    pub fn due(&mut self, audio_now: Instant) -> Vec<R> {
        self.take_due(audio_now)
    }

    pub fn dropped_oldest(&self) -> u64 {
        self.dropped_oldest
    }

    pub fn len(&self) -> usize {
        self.revisions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }

    /// The most recent output-callback instant, if output has started.
    pub fn audio_now(&self) -> Option<Instant> {
        *self.audio_now.lock()
    }

    /// Queues decoded PCM for the output callback. This producer-side copy is
    /// bounded; if it overflows, oldest samples are discarded to keep latency
    /// finite. The callback itself only copies or fills silence.
    pub fn queue_audio(&self, pcm: &[f32]) {
        self.audio.lock().push(pcm);
    }

    /// Starts the default output at 48 kHz. Input and output use the same fixed
    /// codec rate so the callback never resamples or allocates on its hot path.
    ///
    /// Device support only: needs `audio_output` (and therefore cpal).
    /// Without it the scheduler still works — `audio_now()` simply stays
    /// `None` and the caller presents frames immediately.
    #[cfg(feature = "audio")]
    pub fn start_default_output(&mut self) -> Result<()> {
        if self.output.is_some() {
            return Err(anyhow!("audio output is already running"));
        }
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| anyhow!("no default audio output device is available"))?;
        let name = device.name().context("reading audio output device name")?;
        let config = device
            .supported_output_configs()
            .with_context(|| format!("reading output configurations for {name}"))?
            .find(|range| {
                range.min_sample_rate().0 <= SAMPLE_RATE
                    && SAMPLE_RATE <= range.max_sample_rate().0
                    && range.channels() > 0
                    && matches!(
                        range.sample_format(),
                        cpal::SampleFormat::F32 | cpal::SampleFormat::I16 | cpal::SampleFormat::U16
                    )
            })
            .ok_or_else(|| anyhow!("audio output device {name} has no supported 48 kHz F32, I16, or U16 configuration"))?;
        let format = config.sample_format();
        let config = config
            .with_sample_rate(cpal::SampleRate(SAMPLE_RATE))
            .config();
        let stream = build_output_stream(
            &device,
            &config,
            format,
            Arc::clone(&self.audio),
            Arc::clone(&self.audio_now),
        )?;
        stream
            .play()
            .with_context(|| format!("starting audio output {name}"))?;
        self.output = Some(stream);
        Ok(())
    }

    #[cfg(feature = "audio")]
    pub fn stop_output(&mut self) {
        self.output.take();
    }
}

#[derive(Default)]
struct AudioQueue {
    samples: VecDeque<f32>,
}

impl AudioQueue {
    fn push(&mut self, pcm: &[f32]) {
        let overflow = self
            .samples
            .len()
            .saturating_add(pcm.len())
            .saturating_sub(AUDIO_QUEUE_SAMPLES);
        self.samples.drain(..overflow.min(self.samples.len()));
        self.samples.extend(pcm.iter().copied());
    }

    #[cfg(feature = "audio")]
    fn next(&mut self) -> f32 {
        self.samples.pop_front().unwrap_or(0.0)
    }
}

#[cfg(feature = "audio")]
fn build_output_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    format: cpal::SampleFormat,
    audio: Arc<Mutex<AudioQueue>>,
    audio_now: Arc<Mutex<Option<Instant>>>,
) -> Result<cpal::Stream> {
    let channels = usize::from(config.channels);
    let error_callback = |error| tracing::warn!(%error, "audio output stream error");
    match format {
        cpal::SampleFormat::F32 => device
            .build_output_stream(
                config,
                move |output: &mut [f32], _| {
                    fill_output(output, channels, &audio, &audio_now, |x| x)
                },
                error_callback,
                None,
            )
            .context("building F32 audio output stream"),
        cpal::SampleFormat::I16 => device
            .build_output_stream(
                config,
                move |output: &mut [i16], _| {
                    fill_output(output, channels, &audio, &audio_now, |x| {
                        (x.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
                    })
                },
                error_callback,
                None,
            )
            .context("building I16 audio output stream"),
        cpal::SampleFormat::U16 => device
            .build_output_stream(
                config,
                move |output: &mut [u16], _| {
                    fill_output(output, channels, &audio, &audio_now, |x| {
                        ((x.clamp(-1.0, 1.0) + 1.0) * 32_767.5) as u16
                    })
                },
                error_callback,
                None,
            )
            .context("building U16 audio output stream"),
        format => Err(anyhow!("unsupported audio output sample format {format}")),
    }
}

#[cfg(feature = "audio")]
fn fill_output<T>(
    output: &mut [T],
    channels: usize,
    audio: &Mutex<AudioQueue>,
    audio_now: &Mutex<Option<Instant>>,
    convert: impl Fn(f32) -> T,
) {
    if let Some(mut now) = audio_now.try_lock() {
        *now = Some(Instant::now());
    }
    let Some(mut queue) = audio.try_lock() else {
        for sample in output {
            *sample = convert(0.0);
        }
        return;
    };
    for frame in output.chunks_exact_mut(channels) {
        let left = queue.next();
        let right = queue.next();
        for (channel, sample) in frame.iter_mut().enumerate() {
            let pcm = match channel {
                0 if channels == 1 => (left + right) * 0.5,
                0 => left,
                1 => right,
                _ => 0.0,
            };
            *sample = convert(pcm);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn yields_revisions_in_pts_order_and_drops_oldest_on_overflow() {
        let now = Instant::now();
        let mut playout = Playout::new(2);
        playout.schedule(2, now + Duration::from_millis(20));
        playout.schedule(1, now + Duration::from_millis(10));
        playout.schedule(3, now + Duration::from_millis(30));
        assert_eq!(playout.dropped_oldest(), 1);
        assert_eq!(
            playout.take_due(now + Duration::from_millis(40)),
            vec![2, 3]
        );
    }

    #[test]
    fn yields_nothing_before_a_revision_is_due() {
        let now = Instant::now();
        let mut playout = Playout::new(1);
        playout.schedule(7, now + Duration::from_millis(10));
        assert!(playout.take_due(now).is_empty());
        assert_eq!(playout.take_due(now + Duration::from_millis(10)), vec![7]);
    }
}
