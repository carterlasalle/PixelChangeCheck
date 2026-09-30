#[cfg(feature = "audio")]
use anyhow::Context;
use anyhow::{anyhow, Result};
#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "audio")]
use std::sync::atomic::AtomicU64;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::codec::SAMPLES_PER_FRAME;
#[cfg(feature = "audio")]
use super::codec::SAMPLE_RATE;

/// Which audio to capture. The default stays a microphone; `system`
/// and `both` are the seam for the per-platform loopback backends
/// (WASAPI loopback, PipeWire/Pulse monitor, ScreenCaptureKit app audio),
/// which no cross-platform API exposes. Until a backend lands, asking for
/// system audio falls back to the microphone with a warning rather than
/// failing the share — silence about the gap would be worse than noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioSource {
    /// The default input device (a microphone): voices, room, calls.
    #[default]
    Mic,
    /// Whatever the machine is playing. Falls back to mic until a
    /// platform loopback backend exists (see `find_loopback`).
    System,
    /// Mic plus system, mixed. Falls back to mic alone for the same
    /// reason: half the mix is better than no share.
    Both,
    /// Silence on a real clock. The video timeline stays measurable and
    /// the viewer never special-cases "no audio".
    None_,
}

impl AudioSource {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "mic" | "microphone" => Ok(Self::Mic),
            "system" | "loopback" | "output" => Ok(Self::System),
            "both" | "mic+system" | "mix" => Ok(Self::Both),
            "none" | "off" | "silence" => Ok(Self::None_),
            other => anyhow::bail!("--audio-source must be mic|system|both|none, got '{other}'"),
        }
    }
}

/// A loopback candidate: an input device whose name says it taps the
/// output mix rather than a microphone. Covers the real ecosystem, not
/// just the textbook names — verified against the virtual devices on a
/// lived-in Mac (receipt: `pcc diagnose --audio`, 2026-09-30):
/// BlackHole, VB-Cable, Background Music, ZoomAudioDevice, Hue Sync,
/// Serato Virtual Audio, Parrot, Solstice, Teams/Ecamm virtual mics.
/// Microphones never match: "MacBook Pro Microphone" contains none of
/// these, and the test pins that.
#[cfg(feature = "audio")]
fn is_loopback_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [
        "monitor",
        "loopback",
        "stereo mix",
        "stereomix",
        "what u hear",
        "wave out",
        "blackhole",
        "vb-cable",
        "vb cable",
        "vbcable",
        "background music",
        "zoom",
        "hue sync",
        "huesync",
        "serato virtual",
        "parrot",
        "solstice",
        "virtual audio",
        "virtual mic",
        "teams audio",
        "ecamm",
    ]
    .iter()
    .any(|k| n.contains(k))
}

/// Look for a system-audio tap among the host's input devices: a PulseAudio
/// "Monitor of …", a WASAPI loopback endpoint exposed as input, a virtual
/// cable. `None` is the common case — most machines have no such device —
/// and the caller falls back rather than failing.
#[cfg(feature = "audio")]
pub fn find_loopback() -> Option<cpal::Device> {
    let host = cpal::default_host();
    let devices = host.input_devices().ok()?;
    devices
        .filter_map(|d| d.name().ok().map(|n| (n, d)))
        .find(|(n, _)| is_loopback_name(n))
        .map(|(_, d)| d)
}

/// List input devices for `pcc diagnose --audio`: (name, is_loopback).
/// Never opens a stream; naming one is enough for the picker.
/// Without the `audio` feature there are no devices to list.
#[cfg(not(feature = "audio"))]
pub fn list_input_devices() -> Vec<(String, bool)> {
    Vec::new()
}

/// List input devices for `pcc diagnose --audio`: (name, is_loopback).
/// Never opens a stream; naming one is enough for the picker.
#[cfg(feature = "audio")]
pub fn list_input_devices() -> Vec<(String, bool)> {
    let host = cpal::default_host();
    let Ok(devices) = host.input_devices() else {
        return Vec::new();
    };
    devices
        .filter_map(|d| d.name().ok())
        .map(|n| {
            let loopback = is_loopback_name(&n);
            (n, loopback)
        })
        .collect()
}

/// Open the requested source. Silence rather than nothing: a share with
/// no microphone should still send a stream, so the video timeline stays
/// on a real audio clock and the viewer does not have to special-case
/// "no audio".
pub fn open_source(which: AudioSource) -> Result<Box<dyn SystemAudio>> {
    match which {
        AudioSource::None_ => Ok(Box::new(NullSource::new())),
        #[cfg(not(feature = "audio"))]
        _ => {
            let _ = which;
            Ok(Box::new(NullSource::new()))
        }
        #[cfg(feature = "audio")]
        AudioSource::Mic => match cpal::default_host().default_input_device() {
            Some(device) => Ok(Box::new(MicrophoneSource::from_device(device)?)),
            // No device, or the host refused to say. Silence is a valid
            // answer; failing the whole share is not.
            None => Ok(Box::new(NullSource::new())),
        },
        // No platform loopback backend is linked yet. If the OS exposes
        // the mix as an input device (Pulse monitor, virtual cable), use
        // it directly; otherwise the microphone with a warning.
        #[cfg(feature = "audio")]
        AudioSource::System => match find_loopback() {
            Some(device) => Ok(Box::new(MicrophoneSource::from_device(device)?)),
            None => {
                tracing::warn!(
                    "no system-audio loopback device found; capturing the microphone instead                      (WASAPI loopback / PipeWire monitor / ScreenCaptureKit backends not yet implemented)"
                );
                open_source(AudioSource::Mic)
            }
        },
        #[cfg(feature = "audio")]
        AudioSource::Both => match find_loopback() {
            Some(device) => Ok(Box::new(MixedSource::new(device)?)),
            None => {
                tracing::warn!(
                    "no system-audio loopback device found; --audio-source both captures the microphone alone"
                );
                open_source(AudioSource::Mic)
            }
        },
    }
}

/// The best available capture device, falling back to silence.
pub fn default_source() -> Result<Box<dyn SystemAudio>> {
    open_source(AudioSource::Mic)
}

/// Mic plus system, mixed to one stereo frame. The mic is the default
/// input device and the system side is a loopback tap found by
/// `find_loopback`; the mix is a saturating add, so two loud sources clip
/// together rather than wrapping into garbage. Two capture threads, one
/// mixed receiver — the mixer drops to mic-alone if the system side
/// stalls, because half a mix beats a stalled share.
/// Device-backed mixing. Needs the `audio` feature like everything
/// else that touches a device; without it `open_source` resolves every
/// source to silence and this type has no constructor.
/// Dead without the feature, by construction rather than by accident.
#[cfg(feature = "audio")]
pub struct MixedSource {
    /// Device names, resolved on the owner thread. Names — not handles —
    /// because CPAL devices are `!Send` on some hosts and a handle here
    /// would have to cross into the spawned thread.
    mic_name: Option<String>,
    system_name: String,
    running: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
}

#[cfg(feature = "audio")]
impl MixedSource {
    pub fn new(system_device: cpal::Device) -> Result<Self> {
        let system_name = system_device
            .name()
            .context("reading loopback device name")?;
        let mic_name = cpal::default_host()
            .default_input_device()
            .and_then(|d| d.name().ok());
        Ok(Self {
            mic_name,
            system_name,
            running: None,
            worker: None,
        })
    }
}

/// Find an input device by name. Used by the mixer owner thread, which is
/// the only thread that ever touches a device handle.
#[cfg(feature = "audio")]
fn open_named(name: &str) -> Result<MicrophoneSource> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .context("enumerating audio input devices")?;
    for device in devices {
        if device.name().as_deref().unwrap_or("") == name {
            return MicrophoneSource::from_device(device);
        }
    }
    anyhow::bail!("audio device '{name}' disappeared")
}

/// Saturating add of two stereo frames. Public for tests: the mix must
/// clip, never wrap.
pub fn mix_frames(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x + y).clamp(-1.0, 1.0))
        .collect()
}

#[cfg(feature = "audio")]
impl SystemAudio for MixedSource {
    fn device_name(&self) -> &str {
        "mic+system mix"
    }

    fn start(&mut self) -> Result<mpsc::Receiver<AudioFrame>> {
        if self.running.is_some() {
            return Err(anyhow!("mixed source is already running"));
        }
        if self.running.is_some() {
            return Err(anyhow!("mixed source is already running"));
        }
        // Handles stay behind: only names cross into the owner thread,
        // which re-opens both sides where they will live and die.
        let mic_name = self.mic_name.clone();
        let system_name = self.system_name.clone();
        // CPAL devices and streams are `!Send`: even an unstarted source
        // cannot move into a spawned thread. So one owner thread opens,
        // starts, mixes and drops both sources; nothing audio-flavoured
        // ever crosses a thread, only the mixed `AudioFrame`s going out.
        let (sender, receiver) = mpsc::sync_channel(CAPTURE_QUEUE_FRAMES);
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let worker = thread::spawn(move || {
            // Both sources are created, started and dropped on this
            // owner thread — nothing audio-flavoured ever crosses one.
            let Ok(mut system) = open_named(&system_name) else {
                return;
            };
            let mut mic = mic_name.as_deref().and_then(|n| open_named(n).ok());
            let Ok(system_frames) = system.start() else {
                return;
            };
            let mic_frames = match mic.as_mut() {
                Some(m) => m.start().ok(),
                None => None,
            };
            // Latest-only on both sides: 5 ms each keeps both fresh
            // without either starving, and audio is a clock, not a log.
            while worker_running.load(Ordering::Relaxed) {
                let mic_frame = match &mic_frames {
                    Some(rx) => rx.recv_timeout(Duration::from_millis(5)).ok(),
                    None => {
                        thread::sleep(Duration::from_millis(5));
                        None
                    }
                };
                let sys_frame = system_frames.recv_timeout(Duration::from_millis(5)).ok();
                match (mic_frame, sys_frame) {
                    (None, None) => continue,
                    (m, s) => {
                        let pcm = match (m, s) {
                            (Some(m), Some(s)) => mix_frames(&m.pcm, &s.pcm),
                            (Some(m), None) => m.pcm,
                            (None, Some(s)) => s.pcm,
                            (None, None) => unreachable!(),
                        };
                        if matches!(
                            sender.try_send(AudioFrame {
                                pcm,
                                capture_time: Instant::now(),
                            }),
                            Err(mpsc::TrySendError::Disconnected(_))
                        ) {
                            break;
                        }
                    }
                }
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
                .map_err(|_| anyhow!("mixed source worker panicked"))?;
        }
        Ok(())
    }
}

#[cfg(feature = "audio")]
impl Drop for MixedSource {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// The default capture device's name, or `None` when the machine has none.
///
/// Exists so `pcc doctor` can answer "will audio work here?" without
/// starting a stream, which on some platforms is the only way to find out
/// whether a device exists at all.
/// Without the `audio` feature there is never a device.
#[cfg(not(feature = "audio"))]
pub fn default_device_name() -> Option<String> {
    None
}

#[cfg(feature = "audio")]
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
/// Real-device capture. Needs the `audio` feature (cpal); without it
/// this type does not exist and every audio source resolves to silence.
#[cfg(feature = "audio")]
pub struct MicrophoneSource {
    device_name: String,
    device: cpal::Device,
    config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
    stream: Option<cpal::Stream>,
    dropped_frames: Arc<AtomicU64>,
}

#[cfg(feature = "audio")]
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

#[cfg(feature = "audio")]
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

/// Assembles interleaved frames from a cpal input callback. Only the
/// microphone path feeds it, so without the `audio` feature nothing does.
#[cfg(feature = "audio")]
struct FrameAssembler {
    channels: usize,
    pcm: Vec<f32>,
}

#[cfg(feature = "audio")]
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

    #[test]
    fn audio_source_parses_and_rejects_garbage() {
        use AudioSource::*;
        assert_eq!(AudioSource::parse("mic").unwrap(), Mic);
        assert_eq!(AudioSource::parse("MICROPHONE").unwrap(), Mic);
        assert_eq!(AudioSource::parse("system").unwrap(), System);
        assert_eq!(AudioSource::parse("loopback").unwrap(), System);
        assert_eq!(AudioSource::parse("both").unwrap(), Both);
        assert_eq!(AudioSource::parse("mix").unwrap(), Both);
        assert_eq!(AudioSource::parse("none").unwrap(), None_);
        assert_eq!(AudioSource::parse("off").unwrap(), None_);
        assert!(AudioSource::parse("surround").is_err());
        assert!(AudioSource::parse("").is_err());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn loopback_names_match_monitors_and_mixes() {
        for yes in [
            "Monitor of Built-in Audio",
            "Stereo Mix (Realtek)",
            "loopback PCM",
            "What U Hear",
        ] {
            assert!(is_loopback_name(yes), "{yes:?} should count as loopback");
        }
        for yes in [
            "BlackHole 2ch",
            "VB-Cable",
            "Background Music",
            "Background Music (UI Sounds)",
            "ZoomAudioDevice",
            "Hue Sync Audio",
            "Serato Virtual Audio",
            "Microsoft Teams Audio",
        ] {
            assert!(is_loopback_name(yes), "{yes:?} should count as loopback");
        }
        for no in [
            "Built-in Microphone",
            "USB Headset",
            "MacBook Pro Speakers",
            "MacBook Pro Microphone",
        ] {
            assert!(!is_loopback_name(no), "{no:?} should not count as loopback");
        }
    }

    #[test]
    fn mix_clips_rather_than_wrapping() {
        assert_eq!(mix_frames(&[0.5, -0.5], &[0.5, -0.5]), vec![1.0, -1.0]);
        assert_eq!(mix_frames(&[0.9, 0.9], &[0.9, 0.9]), vec![1.0, 1.0]);
        assert_eq!(mix_frames(&[0.1], &[0.2, 0.3]), vec![0.3]);
    }

    // A MicrophoneSource test is deliberately omitted: CI is headless and may
    // have no input device or permission to open one.
}
