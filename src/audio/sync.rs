use std::time::Duration;

/// A timing observation made on the shared monotonic clock.
#[derive(Debug, Clone, Copy)]
pub struct OffsetSample {
    pub capture_to_playback_observed: Duration,
    pub rtt: Duration,
}

impl OffsetSample {
    pub fn new(capture_to_playback_observed: Duration, rtt: Duration) -> Self {
        Self {
            capture_to_playback_observed,
            rtt,
        }
    }
}

const ALPHA: f64 = 0.125;
const MIN_SAMPLES: u32 = 8;
const MAX_OFFSET: Duration = Duration::from_millis(500);
const MAX_CONVERGED_SPREAD: Duration = Duration::from_millis(5);

/// A deliberately small, conservative offset estimator.
///
/// Each observation estimates one-way offset as observed capture-to-playback
/// time minus half the round-trip time. Real paths are asymmetric, so that is
/// only an estimate; the EW mean damps jitter and the convergence gate refuses
/// to present a confident value until its recent absolute spread is small.
/// Values are clamped to 500 ms because a larger value is not useful for this
/// interactive playout path and would let a bad clock sample poison it.
#[derive(Debug, Default)]
pub struct Estimator {
    mean_seconds: Option<f64>,
    spread_seconds: f64,
    samples: u32,
}

impl Estimator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, sample: OffsetSample) {
        let one_way = sample
            .capture_to_playback_observed
            .saturating_sub(sample.rtt / 2)
            .min(MAX_OFFSET);
        let seconds = one_way.as_secs_f64();

        match self.mean_seconds {
            Some(mean) => {
                let deviation = (seconds - mean).abs();
                self.mean_seconds = Some(mean + ALPHA * (seconds - mean));
                self.spread_seconds += ALPHA * (deviation - self.spread_seconds);
            }
            None => self.mean_seconds = Some(seconds),
        }
        self.samples = self.samples.saturating_add(1);
    }

    /// Returns no estimate until enough stable observations exist. Returning
    /// `None` is safer than scheduling video against a plausible-but-wrong
    /// offset while a route is still settling.
    pub fn offset(&self) -> Option<Duration> {
        if self.convergence() {
            self.mean_seconds.map(Duration::from_secs_f64)
        } else {
            None
        }
    }

    pub fn convergence(&self) -> bool {
        self.samples >= MIN_SAMPLES
            && self.mean_seconds.is_some()
            && self.spread_seconds <= MAX_CONVERGED_SPREAD.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(offset_ms: u64) -> OffsetSample {
        OffsetSample::new(
            Duration::from_millis(offset_ms + 20),
            Duration::from_millis(40),
        )
    }

    #[test]
    fn converges_only_after_stable_samples() {
        let mut estimator = Estimator::new();
        for _ in 0..7 {
            estimator.observe(sample(25));
        }
        assert_eq!(estimator.offset(), None);
        estimator.observe(sample(25));
        assert!(estimator.convergence());
        assert_eq!(estimator.offset(), Some(Duration::from_millis(25)));
    }

    #[test]
    fn a_wild_sample_removes_confidence() {
        let mut estimator = Estimator::new();
        for _ in 0..8 {
            estimator.observe(sample(25));
        }
        estimator.observe(sample(450));
        assert!(!estimator.convergence());
        assert_eq!(estimator.offset(), None);
    }
}
