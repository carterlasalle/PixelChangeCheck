use crate::network::NetworkConfig;
use crate::pcc::types::Frame;
use anyhow::{Context, Result};
use quinn::{Endpoint, Connection};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

pub struct QUICTransport {
    endpoint: Endpoint,
    config: NetworkConfig,
    connection: Option<Connection>,
    frame_tx: mpsc::Sender<Frame>,
    frame_rx: mpsc::Receiver<Frame>,
}

impl QUICTransport {
    pub fn new(endpoint: Endpoint, config: NetworkConfig) -> Self {
        let (frame_tx, frame_rx) = mpsc::channel(32); // Buffer size for frame queue
        Self {
            endpoint,
            config,
            connection: None,
            frame_tx,
            frame_rx,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.config.port.unwrap_or(5800)).parse()?;
        let connection = self.endpoint
            .connect(addr, "localhost")?
            .await
            .context("Failed to establish connection")?;
            
        self.connection = Some(connection);
        Ok(())
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> Result<()> {
        if let Some(conn) = &mut self.connection {
            let encoded: Vec<u8> = frame.encode()?;
            let (mut send, _) = conn.open_bi().await?;
            send.write_all(&encoded).await?;
            send.finish().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected"))
        }
    }

    pub async fn receive_frame(&mut self) -> Result<Frame> {
        if let Some(conn) = &mut self.connection {
            let (_, mut recv) = conn.accept_bi().await?;
            let mut buf = vec![0u8; self.config.max_packet_size];
            
            let n = recv.read(&mut buf)
                .await
                .context("Failed to receive frame")?;
            
            let n = match n {
                Some(size) => size,
                None => return Err(anyhow::anyhow!("Connection closed")),
            };
            
            buf.truncate(n);
            Frame::decode(&buf).context("Failed to decode frame")
        } else {
            Err(anyhow::anyhow!("Not connected"))
        }
    }
} 