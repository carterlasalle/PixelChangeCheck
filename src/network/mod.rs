use anyhow::{Context, Result};
use bytes::Bytes;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use crate::pcc::types::Frame;

mod config;
mod transport;
mod resilience;
mod protocol;

pub use config::NetworkConfig;
pub use transport::QUICTransport;
pub use resilience::ResilienceConfig;
pub use protocol::*;

const DEFAULT_PORT: u16 = 5800;

pub struct NetworkManager {
    endpoint: Endpoint,
    config: NetworkConfig,
}

impl NetworkManager {
    pub async fn new_client(config: NetworkConfig) -> Result<Self> {
        let client_config = ClientConfig::new(Arc::new(config.client_crypto_config()));
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        Ok(Self { endpoint, config })
    }

    pub async fn new_server(config: NetworkConfig) -> Result<Self> {
        let server_config = ServerConfig::with_crypto(Arc::new(config.server_crypto_config()));
        let endpoint = Endpoint::server(
            server_config,
            format!("0.0.0.0:{}", config.port.unwrap_or(DEFAULT_PORT)).parse()?,
        )?;

        Ok(Self { endpoint, config })
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<Connection> {
        let connection = self
            .endpoint
            .connect(addr, "localhost")?
            .await
            .context("Failed to establish connection")?;

        Connection::new(connection).await
    }

    pub async fn accept(&self) -> Result<Connection> {
        let incoming_conn = self
            .endpoint
            .accept()
            .await
            .context("No incoming connections")?;

        let connection = incoming_conn
            .await
            .context("Failed to establish connection")?;

        Connection::new(connection).await
    }
}

pub struct Connection {
    quinn_conn: quinn::Connection,
    send_stream: quinn::SendStream,
    recv_stream: quinn::RecvStream,
    frame_tx: mpsc::Sender<Frame>,
    frame_rx: mpsc::Receiver<Frame>,
}

impl Connection {
    async fn new(quinn_conn: quinn::Connection) -> Result<Self> {
        let (send_stream, recv_stream) = quinn_conn
            .open_bi()
            .await
            .context("Failed to open bidirectional stream")?;

        let (frame_tx, frame_rx) = mpsc::channel(32); // Buffer size for frame queue

        Ok(Self {
            quinn_conn,
            send_stream,
            recv_stream,
            frame_tx,
            frame_rx,
        })
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> Result<()> {
        let encoded: Vec<u8> = frame.encode()?;
        self.send_stream
            .write_all(&encoded)
            .await
            .context("Failed to send frame")?;
        Ok(())
    }

    pub async fn receive_frame(&mut self) -> Result<Frame> {
        let mut buf = vec![0u8; 8192]; // Initial buffer size
        let n = self
            .recv_stream
            .read(&mut buf)
            .await
            .context("Failed to receive frame")?;
            
        let n = match n {
            Some(size) => size,
            None => return Err(anyhow::anyhow!("Connection closed")),
        };
        
        buf.truncate(n);
        Frame::decode(&buf).context("Failed to decode frame")
    }

    pub async fn start_frame_processing(mut self) -> Result<()> {
        let (mut send_stream, mut recv_stream) = self.quinn_conn.open_bi().await?;

        // Spawn receive task
        let frame_tx = self.frame_tx.clone();
        tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 8192];
                match recv_stream.read(&mut buf).await {
                    Ok(Some(n)) if n > 0 => {
                        buf.truncate(n);
                        if let Ok(frame) = Frame::decode(&buf) {
                            if frame_tx.send(frame).await.is_err() {
                                break;
                            }
                        }
                    }
                    _ => break,
                }
            }
        });

        // Spawn send task
        let mut frame_rx = self.frame_rx;
        tokio::spawn(async move {
            while let Some(frame) = frame_rx.recv().await {
                if let Ok(encoded) = frame.encode() {
                    if send_stream.write_all(&encoded).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}