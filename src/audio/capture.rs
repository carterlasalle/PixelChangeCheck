use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::codec::{SAMPLES_PER_FRAME, SAMPLE_RATE};

/// The best available capture device, falling back to silence.
///
/// Silence rather than nothing: a share with no microphone should still
/// send a stream, so the video timeline stays on a real audio clock and
/// the viewer does not have to special-case "no audio".
pub fn default_source() -> Result<Box<dyn SystemAudio>> {
    match cpal::default_host().default_input_device() {
        Some(device) => Ok(Box::new(MicrophoneSource::from_device(device)?)),
        // No device, or the host refused to say. Silence is a valid
        // answer; failing the whole share is not.
        None => Ok(Box::new(NullSource::new())),
    }
}

/// The default capture device's name, or `None` when the machine has none.
///
/// Exists so `pcc doctor` can answer "will audio work here?" without
/// starting a stream, which on some platforms is the only way to find out
/// whether a device exists at all.
pub fn default_device_name() -> Option<String> {
    let host = cpal::default_host();
    host.default_input_device()
        .and_then(|d| d.name().ok())
        .map(|n| n.to_string())
}

/// A fixed-size PCM frame captured on the local monotonic clock.
#[derive(Debug)]
pub struct AudioFrame {
    pub pcm: Vec<f32>,
    /// The most important timestamp in this module: this is stamped in the
    /// capture callback on the same monotonic clock used by video capture.
    /// Sync must compare these instants, never wall-clock time.
    pub capture_time: Instant,
}

/// A source of 48 kHz stereo PCM frames.
///
/// Deliberately **not** `Send`: every CPAL host, device and stream is
/// bound to the thread that created it, and pretending otherwise would
/// only move the failure from a compile error to a panic inside a callback.
/// A source lives on its own capture thread; what crosses threads is
/// `AudioFrame`, which is plain data.
pub trait SystemAudio {
    fn device_name(&self) -> &str;
    fn start(&mut self) -> Result<mpsc::Receiver<AudioFrame>>;
    fn stop(&mut self) -> Result<()>;
}

const CAPTURE_QUEUE_FRAMES: usize = 8;

/// The default real microphone source.
///
/// The callback writes to a bounded channel with `try_send`: it never blocks.
/// When the consumer falls behind, the newest complete frame is dropped and
/// [`Self::dropped_frames`] increases. Callers must drain the receiver
/// continuously; this is deliberately bounded rather than risking unbounded
/// memory growth in a real-time callback.
pub struct MicrophoneSource {
    device_name: String,
    device: cpal::Device,
    config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
    stream: Option<cpal::Stream>,
    dropped_frames: Arc<AtomicU64>,
}

impl MicrophoneSource {
    /// Opens the host's default input device at 48 kHz.
    pub fn new() -> Result<Self> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or_else(|| anyhow!("no default audio input device is available"))?;
        Self::from_device(device)
    }

    /// Opens a particular input device. Devices with one channel are duplicated
    /// into stereo; devices with more channels use their first two channels.
    pub fn from_device(device: cpal::Device) -> Result<Self> {
        let device_name = device.name().context("reading audio input device name")?;
        let config = device
            .supported_input_configs()
            .with_context(|| format!("reading input configurations for {device_name}"))?
            .find(|range| {
                range.min_sample_rate().0 <= SAMPLE_RATE
                    && SAMPLE_RATE <= range.max_sample_rate().0
                    && range.channels() > 0
                    && matches!(
                        range.sample_format(),
                        cpal::SampleFormat::F32 | cpal::SampleFormat::I16 | cpal::SampleFormat::U16
                    )
            })
            .ok_or_else(|| {
                anyhow!(
                    "audio input device {device_name} has no supported 48 kHz F32, I16, or U16 configuration"
                )
            })?;
        let sample_format = config.sample_format();
        let config = config
            .with_sample_rate(cpal::SampleRate(SAMPLE_RATE))
            .config();

        Ok(Self {
            device_name,
            device,
            config,
            sample_format,
            stream: None,
            dropped_frames: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Number of complete frames discarded because the consumer was too slow.
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames.load(Ordering::Relaxed)
    }

    fn build_stream(&self, sender: mpsc::SyncSender<AudioFrame>) -> Result<cpal::Stream> {
        let channels = usize::from(self.config.channels);
        let dropped_frames = Arc::clone(&self.dropped_frames);
        let error_callback = |error| tracing::warn!(%error, "audio input stream error");

        match self.sample_format {
            cpal::SampleFormat::F32 => {
                let mut assembler = FrameAssembler::new(channels);
                self.device
                    .build_input_stream(
                        &self.config,
                        move |data: &[f32], _| {
                            assembler.push(data, |sample| sample, &sender, &dropped_frames);
                        },
                        error_callback,
                        None,
                    )
                    .context("building F32 audio input stream")
            }
            cpal::SampleFormat::I16 => {
                let mut assembler = FrameAssembler::new(channels);
                self.device
                    .build_input_stream(
                        &self.config,
                        move |data: &[i16], _| {
                            assembler.push(
                                data,
                                |sample| f32::from(sample) / 32_768.0,
                                &sender,
                                &dropped_frames,
                            );
                        },
                        error_callback,
                        None,
                    )
                    .context("building I16 audio input stream")
            }
            cpal::SampleFormat::U16 => {
                let mut assembler = FrameAssembler::new(channels);
                self.device
                    .build_input_stream(
                        &self.config,
                        move |data: &[u16], _| {
                            assembler.push(
                                data,
                                |sample| (f32::from(sample) - 32_768.0) / 32_768.0,
                                &sender,
                                &dropped_frames,
                            );
                        },
                        error_callback,
                        None,
                    )
                    .context("building U16 audio input stream")
            }
            format => Err(anyhow!("unsupported audio input sample format {format}")),
        }
    }
}

impl SystemAudio for MicrophoneSource {
    fn device_name(&self) -> &str {
        &self.device_name
    }

    fn start(&mut self) -> Result<mpsc::Receiver<AudioFrame>> {
        if self.stream.is_some() {
            return Err(anyhow!(
                "audio input {0} is already running",
                self.device_name
            ));
        }

        let (sender, receiver) = mpsc::sync_channel(CAPTURE_QUEUE_FRAMES);
        let stream = self.build_stream(sender)?;
        stream
            .play()
            .with_context(|| format!("starting audio input {}", self.device_name))?;
        self.stream = Some(stream);
        Ok(receiver)
    }

    fn stop(&mut self) -> Result<()> {
        self.stream.take();
        Ok(())
    }
}

struct FrameAssembler {
    channels: usize,
    pcm: Vec<f32>,
}

impl FrameAssembler {
    fn new(channels: usize) -> Self {
        Self {
            channels,
            pcm: Vec::with_capacity(SAMPLES_PER_FRAME),
        }
    }

    fn push<T: Copy>(
        &mut self,
        data: &[T],
        mut convert: impl FnMut(T) -> f32,
        sender: &mpsc::SyncSender<AudioFrame>,
        dropped_frames: &AtomicU64,
    ) {
        for input_frame in data.chunks_exact(self.channels) {
            let left = convert(input_frame[0]);
            let right = if self.channels == 1 {
                left
            } else {
                convert(input_frame[1])
            };
            self.pcm.extend([left, right]);

            if self.pcm.len() == SAMPLES_PER_FRAME {
                let pcm = std::mem::replace(&mut self.pcm, Vec::with_capacity(SAMPLES_PER_FRAME));
                let frame = AudioFrame {
                    pcm,
                    capture_time: Instant::now(),
                };
                if matches!(sender.try_send(frame), Err(mpsc::TrySendError::Full(_))) {
                    dropped_frames.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}

/// A deterministic silence source for headless operation and tests.
pub struct NullSource {
    running: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
}

impl NullSource {
    pub fn new() -> Self {
        Self {
            running: None,
            worker: None,
        }
    }
}

impl Default for NullSource {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemAudio for NullSource {
    fn device_name(&self) -> &str {
        "silence"
    }

    fn start(&mut self) -> Result<mpsc::Receiver<AudioFrame>> {
        if self.running.is_some() {
            return Err(anyhow!("silence source is already running"));
        }

        let (sender, receiver) = mpsc::sync_channel(CAPTURE_QUEUE_FRAMES);
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let worker = thread::spawn(move || {
            while worker_running.load(Ordering::Relaxed) {
                let frame = AudioFrame {
                    pcm: vec![0.0; SAMPLES_PER_FRAME],
                    capture_time: Instant::now(),
                };
                if matches!(
                    sender.try_send(frame),
                    Err(mpsc::TrySendError::Disconnected(_))
                ) {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        });

        self.running = Some(running);
        self.worker = Some(worker);
        Ok(receiver)
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(running) = self.running.take() {
            running.store(false, Ordering::Relaxed);
        }
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| anyhow!("silence source worker panicked"))?;
        }
        Ok(())
    }
}

impl Drop for NullSource {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn null_source_emits_stereo_frames_with_monotonic_capture_times() {
        let mut source = NullSource::new();
        let receiver = source.start().expect("starts silence source");
        let first = receiver
            .recv_timeout(Duration::from_millis(100))
            .expect("first silence frame");
        let second = receiver
            .recv_timeout(Duration::from_millis(100))
            .expect("second silence frame");
        assert_eq!(first.pcm.len(), SAMPLES_PER_FRAME);
        assert_eq!(second.pcm.len(), SAMPLES_PER_FRAME);
        assert!(second.capture_time >= first.capture_time);
        source.stop().expect("stops silence source");
    }

    // A MicrophoneSource test is deliberately omitted: CI is headless and may
    // have no input device or permission to open one.
}
