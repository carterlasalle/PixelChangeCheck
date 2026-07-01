mod buffer;
pub mod web;
pub use buffer::FrameBuffer;

use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex as AsyncMutex, time};
use std::sync::Mutex as SyncMutex;
use tracing::{debug, error, info};

pub struct Renderer {
    pub buffer: Arc<FrameBuffer>,
    width: u32,
    height: u32,
    fps: u32,
    frame_interval: Duration,
    /// The current rendered frame data (RGB24), for async consumers.
    current_output: Arc<AsyncMutex<Vec<u8>>>,
    /// A plain-mutex mirror of `current_output`, so the synchronous web
    /// server (running on its own OS thread, outside the tokio runtime)
    /// can read the latest frame without needing to block_on anything.
    shared_snapshot: Arc<SyncMutex<Vec<u8>>>,
}

impl Renderer {
    pub async fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        let buffer = Arc::new(FrameBuffer::new(width, height));
        let frame_size = (width * height * 3) as usize;

        info!(
            "Renderer initialized: {}x{} @ {}fps",
            width, height, fps
        );

        Ok(Self {
            buffer,
            width,
            height,
            fps,
            frame_interval: Duration::from_secs(1) / fps.max(1),
            current_output: Arc::new(AsyncMutex::new(vec![0u8; frame_size])),
            shared_snapshot: Arc::new(SyncMutex::new(vec![0u8; frame_size])),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// A cheap, thread-safe snapshot handle for the web server.
    pub fn snapshot_handle(&self) -> Arc<SyncMutex<Vec<u8>>> {
        self.shared_snapshot.clone()
    }

    /// Start the render loop. Continuously pulls frames from the buffer
    /// and updates the current output.
    pub async fn start(&self) -> Result<()> {
        info!("Starting renderer at {} fps", self.fps);

        let mut interval = time::interval(self.frame_interval);

        loop {
            interval.tick().await;

            if let Some(frame) = self.buffer.next_frame().await? {
                if let Err(e) = self.render_frame(&frame).await {
                    error!("Failed to render frame: {}", e);
                    continue;
                }
            }
        }
    }

    /// Render a buffered frame into the current output
    async fn render_frame(&self, frame: &buffer::BufferedFrame) -> Result<()> {
        let mut output = self.current_output.lock().await;

        // Ensure output buffer is the right size
        let expected_size = (frame.width * frame.height * 3) as usize;
        if output.len() != expected_size {
            output.resize(expected_size, 0);
        }

        // Copy frame data to output
        if frame.data.len() == expected_size {
            output.copy_from_slice(&frame.data);
        } else {
            // Partial data or size mismatch - copy what we can
            let copy_len = frame.data.len().min(output.len());
            output[..copy_len].copy_from_slice(&frame.data[..copy_len]);
        }

        self.sync_snapshot(&output);

        debug!("Rendered frame {}: {}x{}", frame.id, frame.width, frame.height);
        Ok(())
    }

    fn sync_snapshot(&self, output: &[u8]) {
        if let Ok(mut snapshot) = self.shared_snapshot.lock() {
            if snapshot.len() != output.len() {
                snapshot.resize(output.len(), 0);
            }
            snapshot.copy_from_slice(output);
        }
    }

    /// Get a copy of the current rendered frame
    pub async fn get_current_frame(&self) -> Vec<u8> {
        self.current_output.lock().await.clone()
    }

    /// Push a freshly-reconstructed frame (e.g. from the network layer,
    /// after applying partial updates) straight into the output buffers
    /// without going through the buffer/render-loop timing.
    pub async fn publish_frame(&self, data: Vec<u8>) {
        let mut output = self.current_output.lock().await;
        *output = data;
        self.sync_snapshot(&output);
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down renderer");
        self.buffer.clear().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcc;

    #[tokio::test]
    async fn test_renderer_creation() {
        let renderer = Renderer::new(1920, 1080, 30).await;
        assert!(renderer.is_ok());
    }

    #[tokio::test]
    async fn test_frame_rendering() {
        let renderer = Renderer::new(1920, 1080, 30).await.unwrap();
        let frame = pcc::Frame {
            id: 1,
            timestamp: std::time::SystemTime::now(),
            width: 1920,
            height: 1080,
            data: vec![128; 1920 * 1080 * 3], // Gray frame
        };

        renderer.buffer.push_frame(frame).await.unwrap();
        if let Some(buffered_frame) = renderer.buffer.next_frame().await.unwrap() {
            assert!(renderer.render_frame(&buffered_frame).await.is_ok());

            // Verify the rendered output
            let output = renderer.get_current_frame().await;
            assert_eq!(output.len(), 1920 * 1080 * 3);
            assert_eq!(output[0], 128); // Check first pixel
        }
    }
}
