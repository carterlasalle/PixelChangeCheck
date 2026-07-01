use crate::network::config::NetworkConfig;
use crate::network::protocol::Message;
use anyhow::{Context, Result};
use async_trait::async_trait;
use quinn::{ClientConfig, Endpoint, RecvStream, SendStream, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;

/// A bidirectional, message-framed connection to a peer. Implemented for
/// direct QUIC connections and for TCP connections proxied through a relay
/// server, so the rest of the app (capture/render loops) doesn't need to
/// care which path frames are taking.
#[async_trait]
pub trait MessageTransport: Send {
    async fn send(&mut self, msg: &Message) -> Result<()>;
    async fn recv(&mut self) -> Result<Message>;
}

/// A single QUIC bidirectional stream, framed with our `Message` protocol.
pub struct QuicTransport {
    send: SendStream,
    recv: RecvStream,
}

impl QuicTransport {
    pub fn new(send: SendStream, recv: RecvStream) -> Self {
        Self { send, recv }
    }
}

#[async_trait]
impl MessageTransport for QuicTransport {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        msg.write_framed(&mut self.send).await
    }

    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.recv).await
    }
}

/// Build a QUIC client endpoint used to dial out to a sharer (or a relay).
pub fn client_endpoint(config: &NetworkConfig) -> Result<Endpoint> {
    let mut client_config = ClientConfig::new(Arc::new(config.client_crypto_config()));
    client_config.transport_config(config.transport_config());
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// Build a QUIC server endpoint that viewers can connect to directly.
pub fn server_endpoint(config: &NetworkConfig, bind_addr: SocketAddr) -> Result<Endpoint> {
    let mut server_config = ServerConfig::with_crypto(Arc::new(config.server_crypto_config()));
    server_config.transport_config(config.transport_config());
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

/// Dial a sharer directly and open the single bi-directional stream used for
/// the whole session.
pub async fn connect_direct(config: &NetworkConfig, addr: SocketAddr) -> Result<QuicTransport> {
    let endpoint = client_endpoint(config)?;
    let connection = endpoint
        .connect(addr, "localhost")?
        .await
        .context("Failed to establish QUIC connection")?;
    let (send, recv) = connection
        .open_bi()
        .await
        .context("Failed to open bidirectional stream")?;
    Ok(QuicTransport::new(send, recv))
}
