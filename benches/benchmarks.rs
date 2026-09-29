//! Benchmarks that measure the workload the product actually does.
//!
//! The previous benchmark discarded the detected changes and then JPEG
//! encoded the whole modified frame, so it measured a pipeline the
//! product does not run. These measure the real one, end to end:
//! capture-shaped frames -> detect -> plan -> compress -> serialise ->
//! deserialise -> apply, and report the bytes that would go on the wire.
//!
//! Run it with `cargo run --release --example benchmarks`. A debug build
//! reports numbers that mean nothing for a codec, so `--release` is not
//! optional.

use pixel_change_check_client::capture::SyntheticCapture;
use pixel_change_check_client::encoder::{encode_snapshot, SnapshotFormat};
use pixel_change_check_client::network::{Message, MAX_MESSAGE_SIZE};
use pixel_change_check_client::pcc::types::FrameCapture;
use pixel_change_check_client::pcc::{Compositor, PCCDetector, PlanLimits, Planner};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Counts allocations so the "no allocation during the scan" claim has a
/// receipt rather than being an assertion in a comment.
struct Counting;

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn take_allocations() -> (u64, u64) {
    (
        ALLOCS.swap(0, Ordering::Relaxed),
        BYTES.swap(0, Ordering::Relaxed),
    )
}

const W: u32 = 1920;
const H: u32 = 1080;

fn bench(name: &str, iterations: u32, mut body: impl FnMut()) {
    // One untimed pass so first-touch page faults are not measured.
    body();
    let start = Instant::now();
    for _ in 0..iterations {
        body();
    }
    let elapsed = start.elapsed();
    println!(
        "{name:<34} {:>8.2} ms/op  ({iterations} iterations)",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );
}

fn main() {
    println!("=== PixelChangeCheck benchmarks ===");
    println!("resolution: {W}x{H}  (release build only)\n");

    let capture = SyntheticCapture::new(W, H);
    let frames: Vec<_> = (0..8).map(|_| capture.capture_frame().unwrap()).collect();

    // ---- detection -----------------------------------------------------
    println!("-- detection --");
    let detector = PCCDetector::default();
    // A sparse, realistic delta: a moving cursor-sized region.
    let mut sparse = frames[0].clone();
    for y in 300..340u32 {
        for x in 600..640u32 {
            let i = ((y * W + x) * 3) as usize;
            sparse.data[i..i + 3].copy_from_slice(&[255, 255, 255]);
        }
    }
    // Two frames differing in a few percent of the screen.
    let mut busy = frames[0].clone();
    for y in 100..900u32 {
        for x in 100..1800u32 {
            let i = ((y * W + x) * 3) as usize;
            busy.data[i] = busy.data[i].wrapping_add(17);
            busy.data[i + 1] = busy.data[i + 1].wrapping_add(31);
        }
    }

    bench("detect: identical frames", 50, || {
        let _ = detector
            .detect(&frames[0].data, &frames[1].data, W, H)
            .unwrap();
    });
    bench("detect: sparse change", 50, || {
        let _ = detector
            .detect(&frames[0].data, &sparse.data, W, H)
            .unwrap();
    });
    bench("detect: ~30% changed", 20, || {
        let _ = detector.detect(&frames[0].data, &busy.data, W, H).unwrap();
    });

    // The claim worth checking: a still screen must not allocate while
    // scanning. The old detector built two temporary vectors per tile,
    // which is ~4,080 allocations for a single 1080p frame.
    let idle = frames[0].clone();
    let _ = detector.detect(&idle.data, &idle.data, W, H).unwrap(); // warm the scratch
                                                                    // Drain the warm-up's counters so only the measured scans are counted.
    let _ = take_allocations();
    for _ in 0..10 {
        let _ = detector.detect(&idle.data, &idle.data, W, H).unwrap();
    }
    let (scan_allocs, scan_bytes) = take_allocations();
    println!(
        "{:<34} {:>8} allocs   {:>10} bytes  (10 idle {W}x{H} scans)",
        "detect: idle allocations", scan_allocs, scan_bytes
    );

    // ---- planning ------------------------------------------------------
    println!("\n-- planning and encoding --");
    let planner = Planner::default();
    bench("plan: sparse change", 50, || {
        let mut reference = frames[0].clone();
        let _ = planner
            .plan(
                &mut reference.data,
                W,
                H,
                &sparse,
                PlanLimits {
                    snapshot_bytes: usize::MAX,
                    max_update_bytes: MAX_MESSAGE_SIZE as usize,
                },
            )
            .unwrap();
    });
    bench("plan: ~30% changed", 20, || {
        let mut reference = frames[0].clone();
        let _ = planner
            .plan(
                &mut reference.data,
                W,
                H,
                &busy,
                PlanLimits {
                    snapshot_bytes: usize::MAX,
                    max_update_bytes: MAX_MESSAGE_SIZE as usize,
                },
            )
            .unwrap();
    });
    bench("encode: lossless PNG snapshot", 10, || {
        let _ = encode_snapshot(W, H, &frames[0].data).unwrap();
    });

    // ---- the whole thing ----------------------------------------------
    println!("\n-- end to end: detect -> plan -> encode -> wire -> decode -> apply --");
    let mut viewer = Compositor::new();
    let (_, snapshot) = encode_snapshot(W, H, &frames[0].data).unwrap();
    viewer
        .begin_snapshot(0, W, H, SnapshotFormat::Png, snapshot.len() as u32, 1)
        .unwrap();
    viewer.push_snapshot_chunk(0, &snapshot).unwrap();
    viewer.commit_snapshot(0, 0).unwrap();

    let iterations = 30u32;
    let start = Instant::now();
    let mut wire = 0usize;
    let mut reference = frames[0].clone();
    let mut exact = true;
    // The viewer already holds revision 0 from the snapshot above.
    for (rev, step) in (0..iterations).enumerate() {
        let rev = rev as u64 + 1;
        let current = frames[1 + (step % 7) as usize].clone();
        let plan = planner
            .plan(
                &mut reference.data,
                W,
                H,
                &current,
                PlanLimits {
                    snapshot_bytes: usize::MAX,
                    max_update_bytes: MAX_MESSAGE_SIZE as usize,
                },
            )
            .unwrap();
        let bytes = if plan.ops.is_empty() && plan.wire_len == 0 {
            Message::KeepAlive { rev }.encode().unwrap()
        } else if plan.ops.is_empty() {
            // A snapshot conveys the whole current frame, so the
            // reference becomes it -- exactly as the sharer does, and
            // without it the next diff would be measured against a
            // surface no viewer holds.
            reference = current.clone();
            // It travels as begin/chunk/commit, exactly as the sharer
            // sends it; sending raw PNG bytes would measure nothing.
            let (_, data) = encode_snapshot(W, H, &current.data).unwrap();
            let begin = Message::SnapshotBegin {
                rev,
                pts_us: 0,
                epoch: 0,
                width: W,
                height: H,
                format: SnapshotFormat::Png,
                total_len: data.len() as u32,
                chunks: 1,
            }
            .encode()
            .unwrap();
            let chunk = Message::SnapshotChunk {
                rev,
                index: 0,
                data,
            }
            .encode()
            .unwrap();
            let commit = Message::SnapshotCommit {
                rev,
                pts_us: 0,
                epoch: 0,
            }
            .encode()
            .unwrap();
            wire += begin.len() + chunk.len() + commit.len();
            // The decoder consumes them in order.
            for part in [begin, chunk, commit] {
                match Message::decode(&part).unwrap() {
                    Message::SnapshotBegin {
                        epoch,
                        width,
                        height,
                        format,
                        total_len,
                        chunks,
                        ..
                    } => {
                        viewer
                            .begin_snapshot(epoch, width, height, format, total_len, chunks)
                            .unwrap();
                    }
                    Message::SnapshotChunk { index, data, .. } => {
                        viewer.push_snapshot_chunk(index, &data).unwrap();
                    }
                    Message::SnapshotCommit {
                        rev,
                        pts_us: 0,
                        epoch,
                    } => {
                        viewer.commit_snapshot(rev, epoch).unwrap();
                    }
                    other => panic!("unexpected {other:?}"),
                }
            }
            continue;
        } else {
            Message::PartialUpdate {
                rev,
                pts_us: 0,
                epoch: 0,
                ops: plan.ops,
            }
            .encode()
            .unwrap()
        };
        wire += bytes.len();
        // Round-trip through the real decoder and the real compositor, so
        // the number below is about what a viewer would actually show.
        match Message::decode(&bytes).unwrap() {
            Message::PartialUpdate {
                rev,
                pts_us: 0,
                epoch,
                ops,
            } => {
                if viewer.apply_ops(rev, epoch, &ops).is_err() {
                    exact = false;
                }
            }
            Message::SnapshotBegin {
                epoch,
                width,
                height,
                format,
                total_len,
                chunks,
                ..
            } => {
                viewer
                    .begin_snapshot(epoch, width, height, format, total_len, chunks)
                    .unwrap();
            }
            Message::SnapshotChunk { index, data, .. } => {
                viewer.push_snapshot_chunk(index, &data).unwrap();
            }
            Message::SnapshotCommit {
                rev,
                pts_us: 0,
                epoch,
            } => {
                exact &= viewer.commit_snapshot(rev, epoch).is_ok();
            }
            _ => {}
        }
    }
    let elapsed = start.elapsed();
    let full_frame = (W * H * 3) as usize;
    println!(
        "{:<34} {:>8.2} ms/frame",
        "full pipeline",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );
    println!(
        "{:<34} {:>8} bytes/frame  ({:.2}% of a raw full frame)",
        "wire cost",
        wire / iterations as usize,
        100.0 * wire as f64 / (full_frame * iterations as usize) as f64
    );
    println!(
        "{:<34} {:>8} bytes",
        "raw full frame (baseline)", full_frame
    );
    println!(
        "{:<34} {:>8}",
        "all updates applied cleanly",
        if exact { "yes" } else { "NO" }
    );
    // The sender's reference after the same frames is what every viewer
    // must be showing; a silent drift here is the failure this whole
    // design exists to prevent.
    println!(
        "{:<34} {:>8}",
        "viewer pixels == reference",
        if viewer.buffer() == reference.data {
            "yes"
        } else {
            "NO"
        }
    );
    println!("\n=== done ===");
}
