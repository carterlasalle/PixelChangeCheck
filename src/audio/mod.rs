//! Real-time audio support for PixelChangeCheck.
//!
//! Audio is **never** derived from the video surface, and the video surface is
//! **never** modified to make audio line up. The pixel surface remains
//! authoritative; synchronization happens only when a revision is presented.
//! A late audio glitch is a presentation problem, never a reason to touch
//! authoritative pixels.
//!
//! The capture timestamp and video timestamps use [`std::time::Instant`], then
//! [`Playout`] treats audio output as the master clock and schedules video
//! around it. Capture and output queues are bounded so a stalled peer cannot
//! turn real-time work into unbounded memory growth.

pub mod capture;
pub mod codec;
pub mod jitter;
pub mod output;
pub mod playout;
pub mod sync;
pub mod syncmeasure;
pub mod transport;

/// Mixing needs devices behind the `audio` feature.
#[cfg(feature = "audio")]
pub use capture::MixedSource;
/// Device-backed capture needs cpal behind the `audio` feature; the
/// silence path and the source enum do not.
#[cfg(feature = "audio")]
pub use capture::{find_loopback, MicrophoneSource};
pub use capture::{
    list_input_devices, mix_frames, open_source, AudioFrame, AudioSource, NullSource, SystemAudio,
};
#[cfg(feature = "audio")]
pub use codec::{OpusDecoder, OpusEncoder};
pub use codec::{CHANNELS, SAMPLES_PER_CHANNEL, SAMPLES_PER_FRAME, SAMPLE_RATE};
pub use jitter::Jitter;
pub use playout::{Playout, ScheduledRevision};
pub use sync::{Estimator, OffsetSample};

/// Twenty milliseconds, the frame size the codec is configured for.
pub const FRAME_MS: u64 = 20;

pub use output::{AudioOutput, PlayedFrame};
pub use transport::DecodedAudio;
/// Encode/decode needs the Opus codec behind the `audio` feature; the
/// datagram framing and its types do not.
#[cfg(feature = "audio")]
pub use transport::{AudioReceiver, AudioSender};
