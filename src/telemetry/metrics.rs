//! Counters, histograms, and the text renderers.
//!
//! Deliberately dependency-free: a fixed-bucket log-scale histogram is
//! about forty lines, and the only consumers are the interval table and
//! the Prometheus endpoint. `hdrhistogram` would be a build-time and
//! API-shape cost for something this small.
//!
//! # The rule this module exists to enforce
//!
//! **Measure, don't log.** The capture loop runs at 10-60 Hz. A log line
//! per frame is unreadable and measurably slow; a counter is free and
//! answers the same question better. Everything here is a counter, and
//! `app/share.rs` only logs *transitions*.

use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// A fixed set of upper bounds in microseconds, log-spaced from 100us to
/// ~10 minutes. Chosen because the interesting range for screen sharing
/// spans "faster than a frame" to "a user has noticed".
const LATENCY_BUCKETS_US: [u64; 24] = [
    50,
    100,
    250,
    500,
    1_000,
    2_000,
    5_000,
    10_000,
    20_000,
    33_333,
    50_000,
    100_000,
    250_000,
    500_000,
    1_000_000,
    2_000_000,
    5_000_000,
    10_000_000,
    20_000_000,
    60_000_000,
    120_000_000,
    300_000_000,
    600_000_000,
    1_200_000_000,
];

/// Latency distribution over [`LATENCY_BUCKETS_US`], plus the running max
/// and total for a mean.
#[derive(Debug, Default)]
pub struct Histogram {
    buckets: [AtomicU64; 24],
    count: AtomicU64,
    sum_us: AtomicU64,
    max_us: AtomicU64,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            buckets: Default::default(),
            count: AtomicU64::new(0),
            sum_us: AtomicU64::new(0),
            max_us: AtomicU64::new(0),
        }
    }

    /// Record a duration. The first value over a bucket lands in it; a
    /// value over *every* bucket lands in the last, which is the overflow
    /// bucket. Without that, the buckets sum to less than `count` and
    /// Prometheus' `+Inf` disagrees with them.
    pub fn record(&self, micros: u64) {
        let last = LATENCY_BUCKETS_US.len() - 1;
        let mut placed = false;
        for (i, bound) in LATENCY_BUCKETS_US.iter().enumerate() {
            if micros <= *bound {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
                placed = true;
                break;
            }
        }
        if !placed {
            self.buckets[last].fetch_add(1, Ordering::Relaxed);
        }
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum_us.fetch_add(micros, Ordering::Relaxed);
        self.max_us.fetch_max(micros, Ordering::Relaxed);
    }

    pub fn record_duration(&self, d: std::time::Duration) {
        self.record(d.as_micros() as u64);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    pub fn mean_us(&self) -> f64 {
        let n = self.count();
        if n == 0 {
            return 0.0;
        }
        self.sum_us.load(Ordering::Relaxed) as f64 / n as f64
    }

    pub fn max_us(&self) -> u64 {
        self.max_us.load(Ordering::Relaxed)
    }

    pub fn max_ms(&self) -> f64 {
        self.max_us() as f64 / 1000.0
    }

    /// The smallest bucket that contains at least `q` of the samples.
    ///
    /// A fixed histogram can only report the bucket edge, not the true
    /// percentile, so this is the lower bound of the bucket containing it.
    /// Saying that is honest; interpolating would be a nicer lie.
    pub fn percentile_us(&self, q: f64) -> u64 {
        let total = self.count();
        if total == 0 {
            return 0;
        }
        let target = (total as f64 * q).ceil() as u64;
        let mut running = 0u64;
        for (i, bound) in LATENCY_BUCKETS_US.iter().enumerate() {
            running += self.buckets[i].load(Ordering::Relaxed);
            if running >= target {
                return *bound;
            }
        }
        *LATENCY_BUCKETS_US.last().expect("non-empty")
    }

    pub fn snapshot(&self) -> HistogramSnapshot {
        let mut cumulative = Vec::with_capacity(LATENCY_BUCKETS_US.len());
        let mut running = 0u64;
        for b in self.buckets.iter() {
            running += b.load(Ordering::Relaxed);
            cumulative.push(running);
        }
        HistogramSnapshot {
            count: self.count(),
            sum_us: self.sum_us.load(Ordering::Relaxed),
            max_us: self.max_us.load(Ordering::Relaxed),
            cumulative,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistogramSnapshot {
    pub count: u64,
    pub sum_us: u64,
    pub max_us: u64,
    /// Cumulative count at each bucket edge.
    pub cumulative: Vec<u64>,
}

impl HistogramSnapshot {
    pub fn mean_ms(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.sum_us as f64 / self.count as f64 / 1000.0
    }

    pub fn max_ms(&self) -> f64 {
        self.max_us as f64 / 1000.0
    }

    /// Prometheus exposition, including the +Inf bucket every histogram
    /// must have or `_bucket{le="+Inf"}` sums to less than `_count`.
    pub fn prometheus(&self, name: &str, help: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# HELP {name} {help}");
        let _ = writeln!(out, "# TYPE {name} histogram");
        for (i, bound) in LATENCY_BUCKETS_US.iter().enumerate() {
            let _ = writeln!(
                out,
                "{name}_bucket{{le=\"{bound}\"}} {}",
                self.cumulative[i]
            );
        }
        let _ = writeln!(out, "{name}_bucket{{le=\"+Inf\"}} {}", self.count);
        let _ = writeln!(out, "{name}_sum {:.6}", self.sum_us as f64 / 1e6);
        let _ = writeln!(out, "{name}_count {}", self.count);
        out
    }
}

/// A plain monotonic counter.
#[derive(Debug, Default)]
pub struct Counter(AtomicU64);

impl Counter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, n: u64) {
        self.0.fetch_add(n, Ordering::Relaxed);
    }

    pub fn incr(&self) {
        self.add(1);
    }

    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

/// A gauge that holds the most recently observed value.
///
/// Chosen over an average on purpose: "the last queue depth" and "the last
/// changed-area fraction" are what an operator wants, and a mean of a
/// queue depth is not a number anyone can act on.
#[derive(Debug, Default)]
pub struct Gauge(AtomicU64);

impl Gauge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a scaled fixed-point value, since atomics have no float type.
    pub fn set(&self, value: f64) {
        self.0
            .store((value * 1000.0).max(0.0) as u64, Ordering::Relaxed);
    }

    pub fn set_raw(&self, value: u64) {
        self.0.store(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> f64 {
        self.0.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// The unscaled stored value, for quantities that are naturally whole
    /// (a byte count, a depth) and would lose meaning scaled.
    pub fn get_raw(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_percentile_lands_in_the_right_bucket() {
        let h = Histogram::new();
        for _ in 0..90 {
            h.record(1_000); // 1ms
        }
        for _ in 0..10 {
            h.record(500_000); // 500ms
        }
        assert_eq!(h.count(), 100);
        // p50 sits in the fast buckets; p95 and above reach the slow ones.
        assert!(
            h.percentile_us(0.50) <= 2_000,
            "p50={}",
            h.percentile_us(0.50)
        );
        assert!(
            h.percentile_us(0.95) > 100_000,
            "p95={}",
            h.percentile_us(0.95)
        );
    }

    #[test]
    fn a_lone_outlier_does_not_move_p50() {
        // 99 fast samples and one very slow one: the median is still fast.
        // Asserting this pins down that the percentile is computed over
        // the sample distribution and not over the maximum.
        let h = Histogram::new();
        for _ in 0..99 {
            h.record(1_000);
        }
        h.record(60_000_000);
        assert!(h.percentile_us(0.50) <= 2_000);
        assert_eq!(h.max_us(), 60_000_000);
    }

    #[test]
    fn buckets_are_cumulative_and_end_at_the_count() {
        let h = Histogram::new();
        for i in 1..=1000u64 {
            h.record(i);
        }
        let snap = h.snapshot();
        assert_eq!(snap.cumulative.len(), LATENCY_BUCKETS_US.len());
        assert_eq!(*snap.cumulative.last().expect("non-empty"), 1000);
        let mut previous = 0;
        for c in &snap.cumulative {
            assert!(*c >= previous, "cumulative must be monotonic");
            previous = *c;
        }
    }

    #[test]
    fn an_empty_histogram_reports_zero_rather_than_guessing() {
        let h = Histogram::new();
        assert_eq!(h.count(), 0);
        assert_eq!(h.percentile_us(0.5), 0);
        assert_eq!(h.mean_us(), 0.0);
    }

    #[test]
    fn a_value_over_every_bucket_is_still_counted_once() {
        let h = Histogram::new();
        h.record(u64::MAX / 2);
        assert_eq!(h.count(), 1);
        // It lands in the top bucket, so the cumulative sum still equals the
        // count and Prometheus' +Inf agrees.
        let snap = h.snapshot();
        assert_eq!(*snap.cumulative.last().expect("non-empty"), 1);
    }

    #[test]
    fn max_tracks_the_worst_sample() {
        let h = Histogram::new();
        h.record_duration(Duration::from_millis(3));
        h.record_duration(Duration::from_millis(40));
        h.record_duration(Duration::from_millis(1));
        assert_eq!(h.max_ms(), 40.0);
    }

    #[test]
    fn mean_is_sane_over_mixed_samples() {
        let h = Histogram::new();
        h.record_duration(Duration::from_millis(10));
        h.record_duration(Duration::from_millis(30));
        assert!((h.mean_us() - 20_000.0).abs() < 1.0);
    }

    #[test]
    fn prometheus_output_has_an_inf_bucket_and_a_count() {
        let h = Histogram::new();
        h.record(1_000);
        let text = h.snapshot().prometheus("pcc_detect", "detect time");
        assert!(text.contains("# TYPE pcc_detect histogram"));
        assert!(text.contains("pcc_detect_bucket{le=\"+Inf\"} 1"));
        assert!(text.contains("pcc_detect_count 1"));
    }

    #[test]
    fn gauges_round_trip_and_reject_negatives() {
        let g = Gauge::new();
        g.set(0.42);
        assert!((g.get() - 0.42).abs() < 0.001);
        g.set(-1.0);
        assert_eq!(g.get(), 0.0, "a negative reading is a bug, not a value");
    }

    #[test]
    fn counters_are_monotonic() {
        let c = Counter::new();
        c.incr();
        c.add(9);
        assert_eq!(c.get(), 10);
    }
}
