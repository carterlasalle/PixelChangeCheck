#![feature(test)]
extern crate test;

use anyhow::Result;
use pixel_change_check_client::{
    capture::ScreenCapture,
    encoder::{self, FrameEncoder},
    pcc::{PCCDetector, Frame, QualityConfig},
};
use test::Bencher;
use tokio::runtime::Runtime;

const BENCH_WIDTH: u32 = 1920;
const BENCH_HEIGHT: u32 = 1080;

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
    
    for i in 0..change_pixels * 3 {
        new_frame.data[i] = 255; // Change some pixels to white
    }
    
    new_frame
}

#[bench]
fn bench_pcc_detection(b: &mut Bencher) {
    let detector = PCCDetector::default();
    let frame1 = create_test_frame(1);
    let frame2 = create_modified_frame(&frame1, 0.1); // 10% change
    
    b.iter(|| {
        detector.detect_changes(&frame1, &frame2).unwrap()
    });
}

#[bench]
fn bench_frame_encoding(b: &mut Bencher) {
    let rt = Runtime::new().unwrap();
    let encoder = rt.block_on(async {
        FrameEncoder::new(BENCH_WIDTH, BENCH_HEIGHT, QualityConfig::default()).unwrap()
    });
    let frame = create_test_frame(1);
    
    b.iter(|| {
        rt.block_on(async {
            encoder.encode_frame(&frame.into()).await.unwrap()
        })
    });
}

#[bench]
fn bench_frame_compression(b: &mut Bencher) {
    let frame = create_test_frame(1);
    
    b.iter(|| {
        encoder::compression::compress_frame(&frame.data, 0.8).unwrap()
    });
}

#[bench]
fn bench_screen_capture(b: &mut Bencher) {
    let capture = ScreenCapture::new().unwrap();
    
    b.iter(|| {
        capture.capture_frame().unwrap()
    });
}

#[bench]
fn bench_full_pipeline(b: &mut Bencher) {
    let rt = Runtime::new().unwrap();
    
    // Initialize components
    let capture = ScreenCapture::new().unwrap();
    let encoder = rt.block_on(async {
        FrameEncoder::new(BENCH_WIDTH, BENCH_HEIGHT, QualityConfig::default()).unwrap()
    });
    let detector = PCCDetector::default();
    
    let mut previous_frame = None;
    
    b.iter(|| {
        rt.block_on(async {
            // Capture
            let frame = capture.capture_frame().unwrap();
            
            // Detect changes
            if let Some(prev) = &previous_frame {
                let _changes = detector.detect_changes(prev, &frame).unwrap();
            }
            
            // Encode
            let _encoded = encoder.encode_frame(&frame.into()).await.unwrap();
            
            previous_frame = Some(frame);
        })
    });
}

// Memory benchmarks
#[bench]
fn bench_memory_usage(b: &mut Bencher) {
    let frame = create_test_frame(1);
    let modified = create_modified_frame(&frame, 0.1);
    let detector = PCCDetector::default();
    
    b.iter(|| {
        // Measure memory allocation during change detection
        let changes = detector.detect_changes(&frame, &modified).unwrap();
        
        // Force memory usage calculation
        let total_change_size: usize = changes.iter()
            .map(|c| c.data.len())
            .sum();
        
        total_change_size
    });
}

// Latency benchmarks
#[bench]
fn bench_end_to_end_latency(b: &mut Bencher) {
    let rt = Runtime::new().unwrap();
    let capture = ScreenCapture::new().unwrap();
    let encoder = rt.block_on(async {
        FrameEncoder::new(BENCH_WIDTH, BENCH_HEIGHT, QualityConfig::default()).unwrap()
    });
    
    b.iter(|| {
        rt.block_on(async {
            let start = std::time::Instant::now();
            
            // Capture and encode
            let frame = capture.capture_frame().unwrap();
            let _encoded = encoder.encode_frame(&frame.into()).await.unwrap();
            
            start.elapsed()
        })
    });
} 