use super::{ControlChannel, ControlMessage, EventChannel, FrameChannel, NetworkConfig, NetworkEvent, protocol::Message};
use anyhow::{Context, Result};
use bytes::Bytes;
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    sync::{mpsc, Mutex},
    time::{self, Duration},
};
use tracing::{debug, error, info, warn};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(5);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

pub struct QUICTransport {
    endpoint: Endpoint,
    connection: Option<Connection>,
    frame_tx: Option<FrameChannel>,
    control_tx: Option<ControlChannel>,
    event_tx: Option<EventChannel>,
    config: NetworkConfig,
}

impl QUICTransport {
    pub fn new(endpoint: Endpoint, config: NetworkConfig) -> Self {
        Self {
            endpoint,
            connection: None,
            frame_tx: None,
            control_tx: None,
            event_tx: None,
            config,
        }
    }

    // Start client connection
    pub async fn connect(&mut self) -> Result<()> {
        let connection = self.endpoint
            .connect(self.config.server_addr, "localhost")?
            .await
            .context("Failed to establish QUIC connection")?;

        info!("Connected to server at {}", self.config.server_addr);
        self.connection = Some(connection);
        
        if let Some(tx) = &self.event_tx {
            tx.send(NetworkEvent::Connected(self.config.server_addr)).await?;
        }

        Ok(())
    }

    // Start server listening
    pub async fn listen(&mut self) -> Result<()> {
        info!("Listening for connections on {}", self.config.server_addr);
        
        while let Some(conn) = self.endpoint.accept().await {
            let connection = conn.await?;
            let remote_addr = connection.remote_address();
            
            info!("Accepted connection from {}", remote_addr);
            self.connection = Some(connection);
            
            if let Some(tx) = &self.event_tx {
                tx.send(NetworkEvent::Connected(remote_addr)).await?;
            }
            
            break; // Only accept one connection for now
        }

        Ok(())
    }

    // Set up communication channels
    pub fn setup_channels(&mut self, buffer_size: usize) -> (FrameChannel, ControlChannel, EventChannel) {
        let (frame_tx, frame_rx) = mpsc::channel(buffer_size);
        let (control_tx, control_rx) = mpsc::channel(buffer_size);
        let (event_tx, event_rx) = mpsc::channel(buffer_size);

        self.frame_tx = Some(frame_tx.clone());
        self.control_tx = Some(control_tx.clone());
        self.event_tx = Some(event_tx.clone());

        (frame_tx, control_tx, event_tx)
    }

    // Start the transport processing
    pub async fn start(&mut self) -> Result<()> {
        let connection = self.connection.as_ref()
            .context("No active connection")?
            .clone();

        // Set up bi-directional stream
        let (mut send, mut recv) = connection.open_bi().await?;

        // Start keep-alive task
        let control_tx = self.control_tx.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(KEEP_ALIVE_INTERVAL);
            loop {
                interval.tick().await;
                if let Some(tx) = &control_tx {
                    if tx.send(ControlMessage::KeepAlive).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Handle incoming messages
        self.handle_messages(&mut send, &mut recv).await?;

        Ok(())
    }

    // Handle message processing
    async fn handle_messages(&self, send: &mut SendStream, recv: &mut RecvStream) -> Result<()> {
        let (message_tx, mut message_rx) = mpsc::channel(32);
        let message_tx = Arc::new(message_tx);

        // Spawn receive task
        let recv_message_tx = message_tx.clone();
        tokio::spawn(async move {
            while let Ok(data) = recv.read_chunk(65535, false).await {
                if let Some(chunk) = data {
                    if let Ok(message) = Message::deserialize(chunk.bytes) {
                        if recv_message_tx.send(message).await.is_err() {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // Main message processing loop
        while let Some(message) = message_rx.recv().await {
            match message {
                Message::FrameData { frame_id, timestamp, data } => {
                    if let Some(tx) = &self.frame_tx {
                        tx.send(data).await?;
                    }
                    // Send acknowledgment
                    send.write_chunk(Message::FrameAck { frame_id }.serialize()?).await?;
                }
                Message::KeepAlive => {
                    debug!("Received keep-alive");
                }
                Message::QualityConfig(config) => {
                    if let Some(tx) = &self.event_tx {
                        tx.send(NetworkEvent::QualityUpdated(config)).await?;
                    }
                }
                Message::Error(error) => {
                    error!("Received error: {}", error);
                    if let Some(tx) = &self.event_tx {
                        tx.send(NetworkEvent::Error(anyhow::anyhow!(error))).await?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    // Clean up resources
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(connection) = self.connection.take() {
            connection.close(0u32.into(), b"shutdown");
        }
        self.endpoint.close(0u32.into(), b"shutdown");
        Ok(())
    }
} 