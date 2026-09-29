//! Adaptive audio jitter control: how long to wait for a late frame
//! before the viewer gives up on it and conceals the gap.
//!
//! A fixed wait is wrong in both directions. On a quiet LAN a missing
//! frame is simply lost, and waiting 100 ms for it is pure latency. On
//! unstable cellular the next frame routinely arrives 60 ms late, and
//! giving up after 20 ms turns jitter into constant glitches.
//!
//! The controller tracks inter-arrival variance with an EW mean (the same
//! shape as the clock estimator in `sync.rs`) and converts it into a
//! hold time clamped to one frame below and a few frames above. The
//! viewer polls datagrams, and when the next frame's pts is already due
//! but its bytes have not arrived, it waits up to the current hold time
//! for them before calling `conceal_missing`. Stable links converge to
//! the floor; jittery links grow enough to stop glitching.
//!
//! Nothing here touches the pixel surface. A late audio frame is a
//! presentation problem; the authoritative pixels are never adjusted to
//! make audio line up.

use std::time::Duration;

use super::FRAME_MS;

/// EW smoothing for the arrival variance. Small enough to track a route
/// change within a second, large enough that one spike does not triple
/// the hold time.
const ALPHA: f64 = 0.125;

/// Hold at least one frame: giving up faster than the frame period means
/// concealing frames that are merely next.
const MIN_HOLD: Duration = Duration::from_millis(super::FRAME_MS);
/// Hold at most a few frames: beyond this the audio is no longer
/// "current speech" and waiting just adds latency to everything after it.
const MAX_HOLD: Duration = Duration::from_millis(super::FRAME_MS * 5);

/// One frame period in microseconds, from the shared frame constant rather
/// than a second literal that can drift from it.
const FRAME_US: u64 = FRAME_MS * 1_000;

/// Nominal hold before any sample arrives: two frames. Enough for ordinary
/// jitter, small enough that a dead link is noticed quickly.
const INITIAL_HOLD: Duration = Duration::from_millis(FRAME_MS * 2);

/// Adaptive hold time for late audio frames.
#[derive(Debug, Clone)]
pub struct Jitter {
    /// EW mean of |arrival interval - frame period|, in microseconds.
    variance_us: f64,
    samples: u32,
}

impl Jitter {
    pub fn new() -> Self {
        Self {
            variance_us: 0.0,
            samples: 0,
        }
    }

    /// Record one arrival gap, in microseconds on the viewer's local clock.
    /// Only gaps between consecutive *received* frames feed the estimate;
    /// gaps the receiver already concealed are loss, not jitter.
    pub fn observe_gap_us(&mut self, gap_us: u64) {
        let deviation = gap_us.abs_diff(FRAME_US) as f64;
        self.variance_us = if self.samples == 0 {
            deviation
        } else {
            (1.0 - ALPHA) * self.variance_us + ALPHA * deviation
        };
        self.samples = self.samples.saturating_add(1);
    }

    /// How long to wait for a frame whose pts is already due before
    /// concealing it. Grows with measured variance, clamped so a stable
    /// link stays near one frame and a wild one never exceeds five.
    pub fn hold_time(&self) -> Duration {
        if self.samples == 0 {
            return INITIAL_HOLD;
        }
        // Twice the mean deviation covers the bulk of a roughly normal
        // jitter distribution without chasing every tail sample.
        let hold_us = (2.0 * self.variance_us).round() as u64;
        Duration::from_micros(hold_us).clamp(MIN_HOLD, MAX_HOLD)
    }

    /// How many 20 ms frames fit in the current hold time. The viewer uses
    /// this to bound how many consecutive concealments one gap may
    /// produce before it stops and waits for a real frame.
    pub fn max_concealed_frames(&self) -> u32 {
        (self.hold_time().as_micros() / FRAME_US as u128).max(1) as u32
    }
}

impl Default for Jitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_arrivals_hold_near_one_frame() {
        let mut j = Jitter::new();
        // A perfect 20 ms cadence has no variance.
        for _ in 0..20 {
            j.observe_gap_us(FRAME_US);
        }
        assert_eq!(j.hold_time(), MIN_HOLD);
        assert_eq!(j.max_concealed_frames(), 1);
    }

    #[test]
    fn jittery_arrivals_grow_the_hold_but_stay_bounded() {
        let mut j = Jitter::new();
        // Alternate 5 ms and 60 ms gaps: heavy but bounded jitter.
        for _ in 0..40 {
            j.observe_gap_us(5_000);
            j.observe_gap_us(60_000);
        }
        let hold = j.hold_time();
        assert!(hold > MIN_HOLD, "variance must grow the hold, got {hold:?}");
        assert!(hold <= MAX_HOLD, "the hold must stay bounded, got {hold:?}");
    }

    #[test]
    fn a_route_change_settles_rather_than_sticking() {
        let mut j = Jitter::new();
        for _ in 0..40 {
            j.observe_gap_us(60_000);
        }
        let wild = j.hold_time();
        for _ in 0..60 {
            j.observe_gap_us(FRAME_US);
        }
        assert!(
            j.hold_time() < wild,
            "steady arrivals must shrink the hold again"
        );
    }

    #[test]
    fn no_samples_means_a_sane_default() {
        let j = Jitter::new();
        assert_eq!(j.hold_time(), INITIAL_HOLD);
    }
}
