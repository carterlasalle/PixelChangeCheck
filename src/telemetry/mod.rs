//! Telemetry: how the sharer, viewer and relay report what they are doing.
//!
//! Two audiences, one set of counters:
//!
//! * a human running this locally wants a periodic table on stderr
//!   (`--stats-interval 5`);
//! * anyone running more than one instance wants Prometheus text on a
//!   loopback-only endpoint (`--metrics-listen`).
//!
//! Both read the same [`Metrics`], so the two views cannot disagree.
//!
//! # What is deliberately absent
//!
//! Nothing here logs. The capture loop runs at 10-60 Hz; a log line per
//! frame is unreadable and costs more than the work it describes. Callers
//! record counters unconditionally and log *transitions*.

pub mod logging;
pub mod metrics;

pub use metrics::{Counter, Gauge, Histogram, HistogramSnapshot};

use std::sync::Arc;
use std::time::{Duration, Instant};

pub use logging::{LogFormat, LogGuard};

/// Everything the process measures. Cheap to clone into the capture loop;
/// every field is an atomic or a `&'static Histogram`.
#[derive(Debug, Default)]
pub struct Metrics {
    /// How long a viewer took from joining to its first exact image.
    pub first_exact_image: Histogram,
    /// How long a viewer took to its first paint of any kind.
    pub first_paint: Histogram,

    /// Per-frame work, in the capture loop.
    pub detect: Histogram,
    pub plan: Histogram,
    pub encode: Histogram,
    pub serialize: Histogram,
    /// Per-frame cost as a viewer would see it, where measurable.
    pub apply: Histogram,

    /// Bytes on the wire, by what produced them.
    pub bytes_patch: Counter,
    pub bytes_fill: Counter,
    pub bytes_copy: Counter,
    pub bytes_snapshot: Counter,
    pub bytes_keepalive: Counter,
    pub bytes_preview: Counter,
    /// Largest single encoded message seen.
    pub peak_message_bytes: Gauge,

    /// How the sharer chose to represent frames.
    pub frames_total: Counter,
    pub frames_idle: Counter,
    pub frames_snapshot_fallback: Counter,
    /// Changed-area fraction of the most recent frame, 0.0-1.0. This is
    /// the number that predicts which strategy ran.
    pub changed_area_fraction: Gauge,

    /// Per-viewer failures, aggregated.
    pub viewers_joined: Counter,
    pub viewers_rejected: Counter,
    pub lag_events: Counter,
    pub repairs: Counter,
    /// Audio datagrams sent, across all direct viewers.
    pub audio_frames_sent: Counter,
    /// Audio frames dropped before or during transmission.
    pub audio_frames_dropped: Counter,
    pub viewer_queue_depth: Gauge,
    pub viewer_rtt_ms: Gauge,
    pub oldest_pending_ms: Gauge,

    /// Capture and scheduling.
    pub captured_frames: Counter,
    pub capture_errors: Counter,
    pub loop_overruns: Counter,
    pub epoch_bumps: Counter,
    pub effective_fps: Gauge,
}

pub type SharedMetrics = Arc<Metrics>;

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shared() -> SharedMetrics {
        Arc::new(Self::new())
    }

    /// Track the largest message seen. `fetch_max` on a scaled gauge is
    /// not available, so this is a compare-then-store; a lost race here
    /// costs a slightly stale peak, which is fine.
    pub fn observe_message_bytes(&self, bytes: usize) {
        let current = self.peak_message_bytes.get_raw();
        if bytes as u64 > current {
            self.peak_message_bytes.set_raw(bytes as u64);
        }
    }

    /// A human-readable summary. Deliberately a table rather than a graph:
    /// this is read on a terminal, once every few seconds, by someone
    /// deciding whether something is wrong.
    pub fn render_table(&self, uptime: Duration) -> String {
        let f = |h: &Histogram| {
            format!(
                "n={:<5} p50={:>6.1}ms p95={:>6.1}ms max={:>7.1}ms",
                h.count(),
                h.percentile_us(0.50) as f64 / 1000.0,
                h.percentile_us(0.95) as f64 / 1000.0,
                h.max_us() as f64 / 1000.0,
            )
        };
        let mib = |c: &Counter| c.get() as f64 / (1024.0 * 1024.0);
        let total_bytes = self.bytes_patch.get()
            + self.bytes_fill.get()
            + self.bytes_copy.get()
            + self.bytes_snapshot.get()
            + self.bytes_keepalive.get()
            + self.bytes_preview.get();
        format!(
            "uptime {:.0}s  frames {}  (idle {})  fps {:.1}  changed {:.1}%\n\
             \x20 work    detect   {}\n\
             \x20         plan     {}\n\
             \x20         encode   {}\n\
             \x20         apply    {}\n\
             \x20         first exact image {}\n\
             \x20 wire    total {:.2} MiB  patch {:.2}  fill {:.2}  copy {:.2}  snapshot {:.2}  preview {:.2}  peak {} B\n\
             \x20 viewers joined {}  rejected {}  lag {}  repairs {}  queue {}  rtt {:.0}ms  pending {:.0}ms\n\
             \x20 capture frames {}  errors {}  loop overruns {}  epoch bumps {}\n",
            uptime.as_secs_f64(),
            self.frames_total.get(),
            self.frames_idle.get(),
            self.effective_fps.get(),
            self.changed_area_fraction.get() * 100.0,
            f(&self.detect),
            f(&self.plan),
            f(&self.encode),
            f(&self.apply),
            f(&self.first_exact_image),
            total_bytes as f64 / (1024.0 * 1024.0),
            mib(&self.bytes_patch),
            mib(&self.bytes_fill),
            mib(&self.bytes_copy),
            mib(&self.bytes_snapshot),
            mib(&self.bytes_preview),
            self.peak_message_bytes.get_raw(),
            self.viewers_joined.get(),
            self.viewers_rejected.get(),
            self.lag_events.get(),
            self.repairs.get(),
            self.viewer_queue_depth.get(),
            self.viewer_rtt_ms.get(),
            self.oldest_pending_ms.get(),
            self.captured_frames.get(),
            self.capture_errors.get(),
            self.loop_overruns.get(),
            self.epoch_bumps.get(),
        )
    }

    /// Prometheus text exposition.
    pub fn render_prometheus(&self, uptime: Duration) -> String {
        let mut out = String::new();
        out.push_str("# HELP pcc_uptime_seconds Process uptime.\n");
        out.push_str("# TYPE pcc_uptime_seconds gauge\n");
        out.push_str(&format!("pcc_uptime_seconds {:.6}\n", uptime.as_secs_f64()));

        fn counter(out: &mut String, name: &str, help: &str, value: u64) {
            out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} counter\n"));
            out.push_str(&format!("{name} {value}\n"));
        }
        fn gauge(out: &mut String, name: &str, help: &str, value: f64) {
            out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} gauge\n"));
            out.push_str(&format!("{name} {value}\n"));
        }

        counter(
            &mut out,
            "pcc_frames_total",
            "Frames processed by the capture loop.",
            self.frames_total.get(),
        );
        counter(
            &mut out,
            "pcc_frames_idle_total",
            "Frames that required no update.",
            self.frames_idle.get(),
        );
        counter(
            &mut out,
            "pcc_bytes_total",
            "Bytes encoded for the wire.",
            self.bytes_patch.get()
                + self.bytes_fill.get()
                + self.bytes_copy.get()
                + self.bytes_snapshot.get()
                + self.bytes_keepalive.get()
                + self.bytes_preview.get(),
        );
        counter(
            &mut out,
            "pcc_bytes_preview_total",
            "Wire bytes from motion-preview hints.",
            self.bytes_preview.get(),
        );
        counter(
            &mut out,
            "pcc_bytes_patch_total",
            "Wire bytes from exact patches.",
            self.bytes_patch.get(),
        );
        counter(
            &mut out,
            "pcc_bytes_fill_total",
            "Wire bytes from solid fills.",
            self.bytes_fill.get(),
        );
        counter(
            &mut out,
            "pcc_bytes_copy_total",
            "Wire bytes from verified copies.",
            self.bytes_copy.get(),
        );
        counter(
            &mut out,
            "pcc_bytes_snapshot_total",
            "Wire bytes from lossless snapshots.",
            self.bytes_snapshot.get(),
        );
        counter(
            &mut out,
            "pcc_viewers_joined_total",
            "Viewers that completed the handshake.",
            self.viewers_joined.get(),
        );
        counter(
            &mut out,
            "pcc_viewers_rejected_total",
            "Viewers refused for a bad or missing token.",
            self.viewers_rejected.get(),
        );
        counter(
            &mut out,
            "pcc_lag_events_total",
            "Times a viewer fell behind and needed repair.",
            self.lag_events.get(),
        );
        counter(
            &mut out,
            "pcc_repairs_total",
            "Snapshots sent to bring a viewer up to date.",
            self.repairs.get(),
        );
        counter(
            &mut out,
            "pcc_audio_frames_sent_total",
            "Audio frames transmitted as QUIC datagrams.",
            self.audio_frames_sent.get(),
        );
        counter(
            &mut out,
            "pcc_audio_frames_dropped_total",
            "Audio frames dropped before transmission.",
            self.audio_frames_dropped.get(),
        );
        counter(
            &mut out,
            "pcc_capture_errors_total",
            "Capture failures.",
            self.capture_errors.get(),
        );
        counter(
            &mut out,
            "pcc_loop_overruns_total",
            "Capture iterations that exceeded the frame budget.",
            self.loop_overruns.get(),
        );
        counter(
            &mut out,
            "pcc_epoch_bumps_total",
            "Surface epoch changes, i.e. geometry changes.",
            self.epoch_bumps.get(),
        );
        gauge(
            &mut out,
            "pcc_peak_message_bytes",
            "Largest single encoded message.",
            self.peak_message_bytes.get_raw() as f64,
        );
        gauge(
            &mut out,
            "pcc_changed_area_fraction",
            "Changed-area fraction of the most recent frame.",
            self.changed_area_fraction.get(),
        );
        gauge(
            &mut out,
            "pcc_effective_fps",
            "Capture rate the scheduler settled on.",
            self.effective_fps.get(),
        );
        gauge(
            &mut out,
            "pcc_viewer_queue_depth",
            "Messages queued for the slowest viewer.",
            self.viewer_queue_depth.get(),
        );
        gauge(
            &mut out,
            "pcc_viewer_rtt_ms",
            "Round-trip time to the slowest viewer.",
            self.viewer_rtt_ms.get(),
        );
        gauge(
            &mut out,
            "pcc_oldest_pending_ms",
            "Age of the oldest message not yet applied by a viewer.",
            self.oldest_pending_ms.get(),
        );

        out.push_str(
            &self
                .detect
                .snapshot()
                .prometheus("pcc_detect_seconds", "Detection time."),
        );
        out.push_str(
            &self
                .plan
                .snapshot()
                .prometheus("pcc_plan_seconds", "Planning time."),
        );
        out.push_str(
            &self
                .encode
                .snapshot()
                .prometheus("pcc_encode_seconds", "Encode time."),
        );
        out.push_str(
            &self
                .apply
                .snapshot()
                .prometheus("pcc_apply_seconds", "Apply time."),
        );
        out.push_str(&self.first_exact_image.snapshot().prometheus(
            "pcc_first_exact_image_seconds",
            "Join to first pixel-exact frame.",
        ));
        out
    }
}

/// Start the interval reporter on `runtime`.
///
/// The handle is taken explicitly rather than assumed: a `Runtime` that
/// merely exists has not been *entered* on this thread, so a bare
/// `tokio::spawn` from `main` would panic with "no reactor running".
/// Returns immediately; the task stops when the process does.
pub fn spawn_interval_reporter(
    runtime: &tokio::runtime::Handle,
    metrics: SharedMetrics,
    every: Duration,
) {
    if every.is_zero() {
        return;
    }
    runtime.spawn(async move {
        let started = Instant::now();
        let mut ticker = tokio::time::interval(every);
        // The first tick fires immediately; skip it so the first line
        // reports a real interval of data rather than an empty second.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            tracing::info!("{}", metrics.render_table(started.elapsed()));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn an_empty_metrics_table_still_renders() {
        let m = Metrics::new();
        let table = m.render_table(Duration::from_secs(1));
        // Every group must be present even at zero, so an operator can
        // tell "no data" from "not instrumented".
        for group in ["work", "wire", "viewers", "capture"] {
            assert!(table.contains(group), "missing {group} in:\n{table}");
        }
    }

    #[test]
    fn prometheus_output_exposes_every_family() {
        let m = Metrics::new();
        m.frames_total.incr();
        m.detect.record_duration(Duration::from_millis(2));
        let text = m.render_prometheus(Duration::from_secs(3));
        for name in [
            "pcc_frames_total",
            "pcc_bytes_total",
            "pcc_viewers_joined_total",
            "pcc_changed_area_fraction",
            "pcc_detect_seconds_bucket",
        ] {
            assert!(text.contains(name), "missing {name}");
        }
        assert!(text.contains("pcc_uptime_seconds 3.000000"));
    }

    #[test]
    fn prometheus_helpers_and_counters_agree() {
        let m = Metrics::new();
        m.bytes_patch.add(100);
        m.bytes_fill.add(25);
        let text = m.render_prometheus(Duration::from_secs(1));
        assert!(text.contains("pcc_bytes_total 125"));
        assert!(text.contains("pcc_bytes_patch_total 100"));
    }

    #[test]
    fn the_peak_message_gauge_never_goes_backwards() {
        let m = Metrics::new();
        m.observe_message_bytes(5000);
        m.observe_message_bytes(1200);
        assert_eq!(m.peak_message_bytes.get_raw(), 5000);
        m.observe_message_bytes(9000);
        assert_eq!(m.peak_message_bytes.get_raw(), 9000);
    }
}
