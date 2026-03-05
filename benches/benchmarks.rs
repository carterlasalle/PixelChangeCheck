use pixel_change_check_client::{
    encoder::{self, FrameEncoder},
    pcc::{PCCDetector, Frame, QualityConfig, PixelChangeDetector},
};
use tokio::runtime::Runtime;

const BENCH_WIDTH: u32 = 640;
const BENCH_HEIGHT: u32 = 480;

// Helper function to create test frame
fn create_test_frame(id: u64) -> Frame {
    Frame {
        id,
        timestamp: std::time::SystemTime::now(),
        width: BENCH_WIDTH,
        height: BENCH_HEIGHT,
        data: vec![0; (BENCH_WIDTH * BENCH_HEIGHT * 3) as usize],
    }
}

// Helper function to create modified frame
fn create_modified_frame(original: &Frame, change_percentage: f32) -> Frame {
    let mut new_frame = original.clone();
    let change_pixels = ((BENCH_WIDTH * BENCH_HEIGHT) as f32 * change_percentage) as usize;

    for i in 0..(change_pixels * 3).min(new_frame.data.len()) {
        new_frame.data[i] = 255;
    }

    new_frame
}

/// Run PCC detection benchmark
fn bench_pcc_detection() {
    let detector = PCCDetector::default();
    let frame1 = create_test_frame(1);
    let frame2 = create_modified_frame(&frame1, 0.1);

    let start = std::time::Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let _changes = detector.detect_changes(&frame1, &frame2).unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "PCC detection: {:.2}ms avg ({} iterations in {:.2}ms)",
        elapsed.as_millis() as f64 / iterations as f64,
        iterations,
        elapsed.as_millis()
    );
}

/// Run frame encoding benchmark
fn bench_frame_encoding() {
    let rt = Runtime::new().unwrap();
    let encoder = FrameEncoder::new(BENCH_WIDTH, BENCH_HEIGHT, QualityConfig::default()).unwrap();
    let frame = create_test_frame(1);

    let start = std::time::Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        rt.block_on(async {
            encoder.encode_frame(&frame.data).await.unwrap();
        });
    }

    let elapsed = start.elapsed();
    println!(
        "Frame encoding: {:.2}ms avg ({} iterations in {:.2}ms)",
        elapsed.as_millis() as f64 / iterations as f64,
        iterations,
        elapsed.as_millis()
    );
}

/// Run frame compression benchmark
fn bench_frame_compression() {
    let frame = create_test_frame(1);

    let start = std::time::Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let compressed = encoder::compression::compress_frame(&frame.data, 0.8).unwrap();
        let _decompressed = encoder::compression::decompress_frame(&compressed).unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "Frame compress+decompress: {:.2}ms avg ({} iterations in {:.2}ms)",
        elapsed.as_millis() as f64 / iterations as f64,
        iterations,
        elapsed.as_millis()
    );
}

/// Run full pipeline benchmark
fn bench_full_pipeline() {
    let rt = Runtime::new().unwrap();
    let encoder = FrameEncoder::new(BENCH_WIDTH, BENCH_HEIGHT, QualityConfig::default()).unwrap();
    let detector = PCCDetector::default();

    let frame1 = create_test_frame(1);
    let frame2 = create_modified_frame(&frame1, 0.1);

    let start = std::time::Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        // Detect changes
        let _changes = detector.detect_changes(&frame1, &frame2).unwrap();

        // Encode
        rt.block_on(async {
            encoder.encode_frame(&frame2.data).await.unwrap();
        });
    }

    let elapsed = start.elapsed();
    println!(
        "Full pipeline (detect+encode): {:.2}ms avg ({} iterations in {:.2}ms)",
        elapsed.as_millis() as f64 / iterations as f64,
        iterations,
        elapsed.as_millis()
    );
}

fn main() {
    println!("=== PixelChangeCheck Benchmarks ===");
    println!("Resolution: {}x{}", BENCH_WIDTH, BENCH_HEIGHT);
    println!();

    bench_pcc_detection();
    bench_frame_encoding();
    bench_frame_compression();
    bench_full_pipeline();

    println!();
    println!("=== Benchmarks complete ===");
} 