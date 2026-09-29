//! The audio output device, on its own thread.
//!
//! `cpal::Stream` is not `Send`, so a playout that owns a device cannot
//! live inside an async task. Keeping it on a dedicated thread also keeps
//! the real-time contract honest: the output callback is fed from a
//! queue this thread owns, and nothing else ever touches it.

use anyhow::Result;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::Duration;

use super::Playout;

/// How many decoded frames may wait for the output device.
///
/// Receipt: 50 frames is one second at 20 ms. A healthy device drains
/// that instantly; a device that has stopped costs a second of latency
/// before anything is dropped, which is far better than dropping under
/// normal jitter.
const QUEUE_FRAMES: usize = 50;

/// Handle to the output thread. `Send`, so it can live in an async task.
pub struct AudioOutput {
    frames: SyncSender<Vec<f32>>,
    stats: std::sync::Arc<Stats>,
}

#[derive(Debug, Default)]
pub struct Stats {
    pub played: std::sync::atomic::AtomicU64,
    pub dropped: std::sync::atomic::AtomicU64,
}

impl std::fmt::Debug for AudioOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioOutput")
            .field("played", &self.stats.played)
            .field("dropped", &self.stats.dropped)
            .finish()
    }
}

impl AudioOutput {
    /// Start output on a device thread.
    pub fn start<R: Send + 'static>(capacity: usize) -> Result<(Self, PlayedReceiver<R>)> {
        let (frames_tx, frames_rx) = mpsc::sync_channel::<Vec<f32>>(QUEUE_FRAMES);
        let (played_tx, played_rx) = mpsc::channel::<PlayedFrame<R>>();
        let stats = std::sync::Arc::new(Stats::default());

        let thread_stats = stats.clone();
        let thread = std::thread::Builder::new()
            .name("pcc-audio-out".into())
            .spawn(move || {
                let mut playout = Playout::<R>::new(capacity);
                match playout.start_default_output() {
                    Ok(()) => {}
                    Err(e) => {
                        // No device is not a failure: the session still
                        // has a clock, it just has no speaker.
                        tracing::debug!("no audio output device: {e}");
                    }
                }
                while let Ok(pcm) = frames_rx.recv() {
                    playout.queue_audio(&pcm);
                    if let Some(now) = playout.audio_now() {
                        let due = playout.take_due(now);
                        if !due.is_empty() {
                            thread_stats
                                .played
                                .fetch_add(due.len() as u64, std::sync::atomic::Ordering::Relaxed);
                            // Ask the caller to render these revisions.
                            if played_tx
                                .send(PlayedFrame {
                                    revisions: due,
                                    at: now,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            })
            .map_err(|e| anyhow::anyhow!("could not start the audio output thread: {e}"))?;
        // The thread owns the device and runs to completion; detaching it
        // avoids a join that would block the viewer.
        drop(thread);

        Ok((
            Self {
                frames: frames_tx,
                stats,
            },
            played_rx,
        ))
    }

    /// Hand one decoded frame to the device thread.
    ///
    /// Never blocks: a full queue means the device has stopped, and
    /// dropping is a click rather than a stall for everything else.
    pub fn push(&self, pcm: &[f32]) {
        match self.frames.try_send(pcm.to_vec()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.stats
                    .dropped
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }

    pub fn played(&self) -> u64 {
        self.stats.played.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn dropped(&self) -> u64 {
        self.stats
            .dropped
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Revisions the audio clock says are due now.
#[derive(Debug)]
pub struct PlayedFrame<R> {
    pub revisions: Vec<R>,
    pub at: std::time::Instant,
}

impl<R> PlayedFrame<R> {
    /// How long to wait before asking again, when nothing is due.
    pub fn next_poll() -> Duration {
        // One audio frame: asking more often is wasted work, asking
        // less often is visible latency.
        Duration::from_millis(super::FRAME_MS)
    }
}

/// A `Receiver` that has been handed to another thread.
pub type PlayedReceiver<R> = Receiver<PlayedFrame<R>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushing_never_blocks_even_with_nothing_draining() {
        let (tx, rx) = mpsc::sync_channel::<Vec<f32>>(1);
        let out = AudioOutput {
            frames: tx,
            stats: std::sync::Arc::new(Stats::default()),
        };
        // Far more than the queue holds, with no consumer at all.
        for _ in 0..1_000 {
            out.push(&[0.0; 10]);
        }
        assert!(out.dropped() > 0, "an undrained queue must count its drops");
        drop(rx);
    }
}
