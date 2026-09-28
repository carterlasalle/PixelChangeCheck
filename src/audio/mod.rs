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
pub mod playout;
pub mod sync;

pub use capture::{AudioFrame, MicrophoneSource, NullSource, SystemAudio};
pub use codec::{
    OpusDecoder, OpusEncoder, CHANNELS, SAMPLES_PER_CHANNEL, SAMPLES_PER_FRAME, SAMPLE_RATE,
};
pub use playout::{Playout, ScheduledRevision};
pub use sync::{Estimator, OffsetSample};
