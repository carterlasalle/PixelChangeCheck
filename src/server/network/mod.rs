use crate::network::{NetworkConfig, ResilienceConfig};
use crate::pcc::types::Frame;
use anyhow::{Context, Result};
use quinn::Endpoint;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

pub struct ServerNetwork {
    endpoint: Endpoint,
    config: NetworkConfig,
    resilience: ResilienceConfig,
    frame_tx: mpsc::Sender<Frame>,
    frame_rx: mpsc::Receiver<Frame>,
}

impl ServerNetwork {
    pub fn new(config: NetworkConfig, resilience: ResilienceConfig) -> Result<Self> {
        let server_config = quinn::ServerConfig::with_crypto(Arc::new(config.server_crypto_config()));
        let endpoint = Endpoint::server(
            server_config,
            format!("0.0.0.0:{}", config.port.unwrap_or(5800)).parse()?,
        )?;

        let (frame_tx, frame_rx) = mpsc::channel(32); // Buffer size for frame queue

        Ok(Self {
            endpoint,
            config,
            resilience,
            frame_tx,
            frame_rx,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Server listening on port {}", self.config.port.unwrap_or(5800));
        
        while let Some(conn) = self.endpoint.accept().await {
            let connection = conn.await?;
            let remote = connection.remote_address();
            info!("Client connected from {}", remote);
            
            // Handle connection...
            let frame_tx = self.frame_tx.clone();
            tokio::spawn(async move {
                Self::handle_connection(connection, frame_tx).await
            });
        }
        
        Ok(())
    }

    async fn handle_connection(connection: quinn::Connection, frame_tx: mpsc::Sender<Frame>) -> Result<()> {
        while let Ok((mut send, mut recv)) = connection.accept_bi().await {
            let mut buf = vec![0u8; 65535];
            
            let n = recv.read(&mut buf)
                .await
                .context("Failed to receive frame data")?;
            
            match n {
                Some(size) => {
                    buf.truncate(size);
                    if let Ok(frame) = Frame::decode(&buf) {
                        frame_tx.send(frame).await?;
                    }
                }
                None => break, // Connection closed
            }
        }
        Ok(())
    }
} 