use crate::network::config::{NetworkConfig, ServerIdentity};
use crate::network::protocol::Message;
use anyhow::{Context, Result};
use async_trait::async_trait;
pub use quinn::Connection;
use quinn::{ClientConfig, Endpoint, RecvStream, SendStream, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

/// The writing half of a transport.
#[async_trait]
pub trait MessageSink: Send {
    async fn send(&mut self, msg: &Message) -> Result<()>;

    /// Write a message that has already been encoded once and is being
    /// shared, byte for byte, with every other viewer.
    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()>;
}

/// The reading half of a transport.
#[async_trait]
pub trait MessageSource: Send {
    async fn recv(&mut self) -> Result<Message>;
}

/// A bidirectional, message-framed connection to a peer. Implemented for
/// direct QUIC connections and for TCP connections proxied through a relay
/// server, so the rest of the app never needs to care which path frames are
/// taking.
#[async_trait]
pub trait MessageTransport: Send {
    async fn send(&mut self, msg: &Message) -> Result<()>;
    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()>;
    async fn recv(&mut self) -> Result<Message>;

    /// Split into independently owned halves, so a reader task can watch
    /// for viewer control messages while a writer task streams updates.
    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>);
}

/// Write an already-encoded envelope to a stream: 4-byte little-endian
/// length, then the envelope.
pub async fn write_encoded<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    bytes: &[u8],
) -> Result<()> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| anyhow::anyhow!("Encoded message does not fit a u32 length"))?;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(bytes).await?;
    writer.flush().await?;
    Ok(())
}

// ------------------------------------------------------------------- QUIC

struct QuicSink {
    send: SendStream,
    _connection: Connection,
}

#[async_trait]
impl MessageSink for QuicSink {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        self.send_encoded(&msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        write_encoded(&mut self.send, bytes).await
    }
}

struct QuicSource {
    recv: RecvStream,
    _connection: Connection,
}

#[async_trait]
impl MessageSource for QuicSource {
    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.recv).await
    }
}

/// A single QUIC bidirectional stream, framed with our `Message` protocol.
pub struct QuicTransport {
    send: SendStream,
    recv: RecvStream,
    /// Held so the connection outlives the stream handle, which makes the
    /// lifetime explicit rather than incidental.
    connection: Connection,
}

impl QuicTransport {
    pub fn new(send: SendStream, recv: RecvStream, connection: Connection) -> Self {
        Self {
            send,
            recv,
            connection,
        }
    }
}

#[async_trait]
impl MessageTransport for QuicTransport {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        write_encoded(&mut self.send, &msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        write_encoded(&mut self.send, bytes).await
    }

    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.recv).await
    }

    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>) {
        (
            Box::new(QuicSink {
                send: self.send,
                _connection: self.connection.clone(),
            }),
            Box::new(QuicSource {
                recv: self.recv,
                _connection: self.connection,
            }),
        )
    }
}

// --------------------------------------------------------------- endpoints

/// Build a QUIC client endpoint that pins one exact server certificate.
pub fn client_endpoint(config: &NetworkConfig, pin: &[u8]) -> Result<Endpoint> {
    let mut client_config = ClientConfig::new(Arc::new(NetworkConfig::client_tls_config(pin)?));
    client_config.transport_config(config.transport_config());
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// Build a QUIC server endpoint that viewers can connect to directly.
pub fn server_endpoint(
    config: &NetworkConfig,
    identity: &ServerIdentity,
    bind_addr: SocketAddr,
) -> Result<Endpoint> {
    let mut server_config =
        ServerConfig::with_crypto(Arc::new(NetworkConfig::server_crypto_config(identity)?));
    server_config.transport_config(config.transport_config());
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

/// Resolve a `host:port` string, so `--connect sharer.local:5800` works and
/// not just a literal address. IPv4 is preferred so a dual-stack lookup on
/// a LAN resolves to the address a viewer will actually reach.
pub async fn resolve(target: &str) -> Result<SocketAddr> {
    let mut addrs: Vec<SocketAddr> = tokio::net::lookup_host(target)
        .await
        .with_context(|| format!("Could not resolve '{target}'"))?
        .collect();
    addrs.sort_by_key(|a| u8::from(a.is_ipv6()));
    addrs
        .into_iter()
        .next()
        .with_context(|| format!("'{target}' resolved to no addresses"))
}

/// Dial a sharer directly and open the single bi-directional stream used
/// for the whole session. The certificate is validated by fingerprint, not
/// by `server_name`: a self-signed LAN certificate has no DNS identity
/// worth checking, and the pin is the stronger check anyway.
pub async fn connect_direct(
    config: &NetworkConfig,
    pin: &[u8],
    target: &str,
    server_name: &str,
) -> Result<QuicTransport> {
    let addr = resolve(target).await?;
    let endpoint = client_endpoint(config, pin)?;
    let connection = endpoint
        .connect(addr, server_name)
        .with_context(|| format!("Failed to reach {target} ({addr})"))?
        .await
        .with_context(|| format!("QUIC handshake with {target} failed"))?;
    let (send, recv) = connection
        .open_bi()
        .await
        .context("Failed to open bidirectional stream")?;
    Ok(QuicTransport::new(send, recv, connection))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    /// A single `await`; reuse a current-thread runtime rather than adding
    /// an executor dependency for two assertions.
    fn block_on<F: Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    fn resolve_handles_a_literal_address() {
        let addr = block_on(resolve("127.0.0.1:5800")).unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:5800");
    }

    #[test]
    fn an_unresolvable_target_says_so() {
        let err = block_on(resolve("pcc.invalid.example:5800"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("pcc.invalid.example"), "unhelpful: {err}");
    }
}
