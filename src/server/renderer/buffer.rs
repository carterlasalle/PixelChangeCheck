use anyhow::Result;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Mutex;
use tracing::{debug, warn};

const MAX_BUFFER_SIZE: usize = 3; // Maximum number of frames to keep in buffer
const FRAME_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct FrameBuffer {
    frames: Arc<Mutex<VecDeque<BufferedFrame>>>,
    current_frame: Arc<Mutex<Option<BufferedFrame>>>,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
struct BufferedFrame {
    id: u64,
    timestamp: SystemTime,
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            frames: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_BUFFER_SIZE))),
            current_frame: Arc::new(Mutex::new(None)),
            width,
            height,
        }
    }

    // Add a new frame to the buffer
    pub async fn push_frame(&self, frame: crate::pcc::Frame) -> Result<()> {
        let mut frames = self.frames.lock().await;
        
        // Remove oldest frame if buffer is full
        if frames.len() >= MAX_BUFFER_SIZE {
            frames.pop_front();
        }
        
        // Add new frame
        frames.push_back(BufferedFrame {
            id: frame.id,
            timestamp: frame.timestamp,
            data: frame.data,
            width: frame.width,
            height: frame.height,
        });
        
        Ok(())
    }

    // Apply frame updates to the current frame
    pub async fn apply_updates(&self, updates: Vec<crate::pcc::PixelChange>) -> Result<()> {
        let mut current = self.current_frame.lock().await;
        
        if let Some(frame) = current.as_mut() {
            // Apply each update to the current frame
            for update in updates {
                let start_x = update.x;
                let start_y = update.y;
                let width = update.width;
                let height = update.height;
                
                // Update pixel data
                for y in 0..height {
                    let frame_offset = ((start_y + y) * self.width + start_x) as usize * 3;
                    let update_offset = (y * width) as usize * 3;
                    let update_end = update_offset + (width as usize * 3);
                    
                    frame.data[frame_offset..frame_offset + (width as usize * 3)]
                        .copy_from_slice(&update.data[update_offset..update_end]);
                }
            }
        } else {
            warn!("No current frame to update");
        }
        
        Ok(())
    }

    // Get the next frame for rendering
    pub async fn next_frame(&self) -> Result<Option<BufferedFrame>> {
        let mut frames = self.frames.lock().await;
        
        // Remove expired frames
        while let Some(frame) = frames.front() {
            if frame.timestamp.elapsed()? > FRAME_TIMEOUT {
                frames.pop_front();
            } else {
                break;
            }
        }
        
        // Get next frame
        if let Some(frame) = frames.pop_front() {
            let mut current = self.current_frame.lock().await;
            *current = Some(frame.clone());
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    // Get the current frame without advancing
    pub async fn current_frame(&self) -> Option<BufferedFrame> {
        self.current_frame.lock().await.clone()
    }

    // Clear the buffer
    pub async fn clear(&self) {
        let mut frames = self.frames.lock().await;
        frames.clear();
        let mut current = self.current_frame.lock().await;
        *current = None;
    }
} 