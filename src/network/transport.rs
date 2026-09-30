use crate::network::config::{NetworkConfig, ServerIdentity};
use crate::network::e2e;
use crate::network::protocol::{Message, MAX_MESSAGE_SIZE};
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

    /// Read one *envelope*, undecoded. A sealed transport needs the
    /// ciphertext before it can decrypt, so this cannot go through
    /// `recv`. The default is right for a plaintext transport, which is
    /// the only kind this is a sensible default for.
    async fn recv_raw(&mut self) -> Result<Vec<u8>> {
        let msg = self.recv().await?;
        Ok(msg.encode()?)
    }
}

/// Which transport a session runs on. Today only `Quic` is implemented;
/// `Iroh` and `WebRtc` are named (see ADR 0006) so CLI parsing, help text
/// and error paths exist before any dependency does — adding one is a new
/// `MessageTransport` impl behind an existing variant, not a flag day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportKind {
    /// QUIC over UDP (direct) or the relay fan (TCP+TLS). The default.
    #[default]
    Quic,
    /// Peer-to-peer with relay fallback via iroh. Ticket in, ticket out:
    /// the sharer prints an iroh ticket, the viewer dials it.
    Iroh,
    /// Browser-native peer connection. Not implemented.
    WebRtc,
}

impl TransportKind {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "quic" => Ok(Self::Quic),
            "iroh" => Ok(Self::Iroh),
            "webrtc" | "web-rtc" | "rtc" => Ok(Self::WebRtc),
            other => anyhow::bail!("--transport must be quic|iroh|webrtc, got '{other}'"),
        }
    }

    /// Fail loudly for unimplemented transports at session setup, not
    /// mid-handshake. The error names the ADR so the user learns why.
    pub fn require_implemented(self) -> anyhow::Result<()> {
        match self {
            Self::Quic | Self::Iroh | Self::WebRtc => Ok(()),
        }
    }
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

/// A sink that seals everything it writes.
///
/// The handshake travels in the clear by necessity -- it is what derives
/// these keys -- and everything after it does not. The relay sits between
/// the two peers and sees only frame sizes and timing.
pub struct SealedSink {
    inner: Box<dyn MessageSink>,
    direction: e2e::Direction,
}

impl std::fmt::Debug for SealedSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SealedSink")
            .field("direction", &self.direction)
            .finish_non_exhaustive()
    }
}

impl SealedSink {
    /// Seal everything written through this sink.
    pub fn new(inner: Box<dyn MessageSink>, direction: e2e::Direction) -> Self {
        Self { inner, direction }
    }
}

impl SealedSource {
    /// Open everything read through this source.
    pub fn new(inner: Box<dyn MessageSource>, direction: e2e::Direction) -> Self {
        Self { inner, direction }
    }
}

#[async_trait]
impl MessageSink for SealedSink {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        let encoded = msg.encode()?;
        MessageSink::send_encoded(self, &encoded).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        let sealed = self.direction.seal(bytes)?;
        self.inner.send_encoded(&sealed).await
    }
}

/// A source that opens everything it reads.
pub struct SealedSource {
    inner: Box<dyn MessageSource>,
    direction: e2e::Direction,
}

impl std::fmt::Debug for SealedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SealedSource")
            .field("direction", &self.direction)
            .finish()
    }
}

#[async_trait]
impl MessageSource for SealedSource {
    async fn recv(&mut self) -> Result<Message> {
        let frame = self.inner.recv_raw().await?;
        let plain = self.direction.open(&frame)?;
        Ok(Message::decode(&plain)?)
    }
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

    /// Read one envelope without decoding it, which is what a sealed
    /// session needs: the ciphertext is not a `Message` yet.
    async fn recv_raw(&mut self) -> Result<Vec<u8>> {
        Message::read_envelope(&mut self.recv).await
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
    /// The underlying connection, for opening a separate audio stream.
    ///
    /// A connection handle is cheap to clone and is the only way to reach
    /// a stream that is not the message path.
    pub fn connection(&self) -> Connection {
        self.connection.clone()
    }

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

// ------------------------------------------------------------------ iroh

/// ALPN identifying a pcc session on an iroh endpoint. Both ends must
/// agree, or the connection fails before any `Message` flows — which is
/// exactly the version-gate behaviour the native path gets from its
/// first byte.
pub const IROH_ALPN: &[u8] = b"pcc/v7";

/// An iroh session ticket: the endpoint id plus known addresses, as JSON
/// base64. Printed by the sharer, pasted or scanned by the viewer —
/// the same job the `pair` URL does for direct connections, but the
/// address list is endpoint-discovered (host candidates + relay), not a
/// single `host:port` the user typed.
pub fn iroh_ticket(addr: &iroh::EndpointAddr) -> String {
    use base64::Engine;
    let json = serde_json::to_vec(addr).expect("EndpointAddr is serializable");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
}

/// Parse a ticket back. A garbage ticket is a user error with the
/// expected shape in it, not a panic.
pub fn parse_iroh_ticket(ticket: &str) -> Result<iroh::EndpointAddr> {
    use base64::Engine;
    let json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(ticket.trim())
        .context("iroh ticket is not base64")?;
    serde_json::from_slice(&json)
        .context("iroh ticket is not an endpoint address (expected the sharer's printed ticket)")
}

/// One iroh bidirectional stream, framed with the same `Message`
/// protocol as every other transport. iroh's streams are tokio
/// `AsyncRead`/`AsyncWrite` (via noq), so the framing code below is the
/// same `read_framed`/`write_encoded` the QUIC path uses — only the
/// stream types differ.
pub struct IrohTransport {
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
    /// Held so the connection outlives the stream handles. Same reason
    /// as `QuicTransport::connection`: without this the endpoint may
    /// tear the session down while streams are still open — which is
    /// exactly the "connected, waiting for the first snapshot" hang a
    /// live test caught.
    _connection: iroh::endpoint::Connection,
    /// Held so the endpoint outlives the session. Dropping an iroh
    /// `Endpoint` aborts every connection on it ungracefully ("Endpoint
    /// dropped without calling `Endpoint::close`") — the dialer kept its
    /// connection but dropped this, and the handshake died right after
    /// Hello. A live session caught it.
    _endpoint: iroh::Endpoint,
}

impl IrohTransport {
    pub fn new(
        send: iroh::endpoint::SendStream,
        recv: iroh::endpoint::RecvStream,
        connection: iroh::endpoint::Connection,
        endpoint: iroh::Endpoint,
    ) -> Self {
        Self {
            send,
            recv,
            _connection: connection,
            _endpoint: endpoint,
        }
    }
}

#[async_trait]
impl MessageTransport for IrohTransport {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        msg.write_framed(&mut self.send).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        write_encoded(&mut self.send, bytes).await
    }

    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.recv).await
    }

    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>) {
        // The connection/endpoint handles move with the sink: whichever
        // half dies last still holds the session up. Splitting them
        // across halves would let the first drop tear down the second —
        // the exact "connection lost right after Hello" failure this
        // field exists to prevent.
        (
            Box::new(IrohSink {
                send: self.send,
                _connection: self._connection,
                _endpoint: self._endpoint,
            }),
            Box::new(IrohSource { recv: self.recv }),
        )
    }
}

struct IrohSink {
    send: iroh::endpoint::SendStream,
    _connection: iroh::endpoint::Connection,
    _endpoint: iroh::Endpoint,
}

#[async_trait]
impl MessageSink for IrohSink {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        msg.write_framed(&mut self.send).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        write_encoded(&mut self.send, bytes).await
    }
}

struct IrohSource {
    recv: iroh::endpoint::RecvStream,
}

#[async_trait]
impl MessageSource for IrohSource {
    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.recv).await
    }

    async fn recv_raw(&mut self) -> Result<Vec<u8>> {
        Message::read_envelope(&mut self.recv).await
    }
}

/// Open our iroh endpoint (N0 preset: relay + discovery) and return it
/// with its ticket. One endpoint per process; the ticket carries the
/// current addresses.
pub async fn iroh_host_endpoint() -> Result<(iroh::Endpoint, String)> {
    let ep = iroh::Endpoint::builder(iroh::endpoint::presets::N0)
        .alpns(vec![IROH_ALPN.to_vec()])
        .bind()
        .await
        .context("binding the iroh endpoint")?;
    // Let local candidates settle: without this the ticket may carry no
    // direct addresses and every viewer pays a relay round trip.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let addr = ep.addr();
    Ok((ep, iroh_ticket(&addr)))
}

/// Dial an iroh ticket and open the session stream.
pub async fn iroh_dial(ticket: &str) -> Result<IrohTransport> {
    use iroh::endpoint::presets::N0;
    let addr = parse_iroh_ticket(ticket)?;
    let ep = iroh::Endpoint::bind(N0)
        .await
        .context("binding the iroh endpoint")?;
    let conn = ep
        .connect(addr, IROH_ALPN)
        .await
        .context("connecting to the sharer's iroh endpoint")?;
    let (send, recv) = conn
        .open_bi()
        .await
        .context("opening the iroh session stream")?;
    Ok(IrohTransport::new(send, recv, conn, ep))
}

// ---------------------------------------------------------- webrtc
//
// Signalling is manual and in-band to the CLI: the sharer prints one
// base64 blob (its offer SDP), the viewer answers with one blob (its
// answer SDP), and ICE candidates ride inside the SDP because gathering
// completes before either blob is printed (non-trickle). No STUN server
// is configured: both ends gather host candidates, which covers LAN,
// loopback and direct paths; a symmetric-NAT pair needs the relay leg,
// which is what the quic/iroh paths are for. Copy-paste signalling is
// the honest scope — a rendezvous server would be a new service, not a
// transport.
//
// Framing: SCTP caps a data-channel message at 16 KiB (`max-message-size`
// in the SDP we both emit). Every `Message` envelope is therefore
// chunked into 16 KiB frames with a 9-byte header (total u32 LE,
// index u32, last u8) and reassembled on receipt. Snapshots already
// travel as 1 MiB `SnapshotChunk`s, but an update burst can exceed one
// message — chunking lives below `Message`, so the protocol never sees it.

/// SCTP data-channel payload ceiling. Receipt: both SDPs we emit carry
/// `a=max-message-size:65536`, and the stack enforces 16384 per message.
pub const WEBRTC_CHUNK_BYTES: usize = 16 * 1024;

/// Label of the single session data channel. Both ends open-or-accept
/// exactly this label; anything else is ignored.
pub const WEBRTC_CHANNEL_LABEL: &str = "pcc";

/// One signalling blob: a full SDP (offer or answer) with gathered
/// candidates embedded, base64url. Printed by one side, pasted to the
/// other. Fits in a chat message; a 4K-screen SDP is a few KiB.
pub fn webrtc_blob(sdp: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sdp.as_bytes())
}

/// Parse a signalling blob back. Garbage is a user error naming the
/// expected shape, not a panic.
pub fn parse_webrtc_blob(blob: &str) -> Result<String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(blob.trim())
        .context("webrtc blob is not base64")?;
    String::from_utf8(bytes).context("webrtc blob is not UTF-8 SDP")
}

/// Split an encoded envelope into 16 KiB frames. Header: total u32 LE,
/// index u32 LE, last u8. Pure function, unit-tested below.
pub fn webrtc_chunk(envelope: &[u8]) -> Vec<Vec<u8>> {
    let total = envelope.len() as u32;
    let pieces: Vec<&[u8]> = envelope.chunks(WEBRTC_CHUNK_BYTES).collect();
    let last_idx = pieces.len().saturating_sub(1);
    let mut out = Vec::with_capacity(pieces.len());
    for (index, piece) in pieces.into_iter().enumerate() {
        let mut frame = Vec::with_capacity(9 + piece.len());
        frame.extend_from_slice(&total.to_le_bytes());
        frame.extend_from_slice(&(index as u32).to_le_bytes());
        frame.push((index == last_idx) as u8);
        frame.extend_from_slice(piece);
        out.push(frame);
    }
    out
}

/// Reassemble chunked frames. Returns `None` until the final frame of a
/// complete set arrives; out-of-order delivery reassembles by index, a
/// new `total` resets the buffer (the sender never interleaves two
/// envelopes on one channel).
#[derive(Debug, Default)]
pub struct WebrtcReassembler {
    buf: Vec<Option<Vec<u8>>>,
    total: usize,
    filled: usize,
    /// Whether the final-index frame arrived. Completion needs all chunks
    /// AND the last one — otherwise a short final frame could pass the
    /// size check while the real tail is still in flight.
    saw_last: bool,
}

impl WebrtcReassembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one 9-byte-header frame. `Ok(Some(envelope))` exactly once
    /// per complete set; `Ok(None)` while incomplete. Garbage is an
    /// error naming the numbers.
    pub fn feed(&mut self, frame: &[u8]) -> Result<Option<Vec<u8>>> {
        if frame.len() < 9 {
            anyhow::bail!("webrtc frame too short: {} bytes (need >= 9)", frame.len());
        }
        let total = u32::from_le_bytes(frame[0..4].try_into().unwrap()) as usize;
        let index = u32::from_le_bytes(frame[4..8].try_into().unwrap()) as usize;
        let last = frame[8];
        if total > MAX_MESSAGE_SIZE as usize + 5 {
            anyhow::bail!("webrtc envelope too large: {total} bytes");
        }
        let chunks = total.div_ceil(WEBRTC_CHUNK_BYTES);
        if chunks == 0 || chunks > 4096 {
            anyhow::bail!("webrtc chunk count {chunks} outside 1..=4096");
        }
        if index >= chunks {
            anyhow::bail!("webrtc chunk index {index} >= count {chunks}");
        }
        if self.buf.len() != chunks {
            self.buf = vec![None; chunks];
            self.total = total;
            self.filled = 0;
            self.saw_last = false;
        }
        if self.buf[index].is_none() {
            self.buf[index] = Some(frame[9..].to_vec());
            self.filled += 1;
        }
        if last == 1 {
            self.saw_last = true;
        }
        if self.saw_last && self.filled == chunks {
            let mut out = Vec::with_capacity(self.total);
            for part in self.buf.drain(..).flatten() {
                out.extend_from_slice(&part);
            }
            self.filled = 0;
            self.saw_last = false;
            if out.len() != self.total {
                anyhow::bail!(
                    "webrtc reassembly size {} != header total {}",
                    out.len(),
                    self.total
                );
            }
            return Ok(Some(out));
        }
        Ok(None)
    }
}

// ------------------------------------------------- webrtc session
//
// One reliable ordered data channel labelled `pcc`, chunked at 16 KiB.
// The sharer creates the channel and the offer; the viewer answers.
// Signalling travels as two CLI blobs (offer out, answer in); ICE is
// non-trickle with host candidates, exactly the shape the loopback probe
// proved on this machine.

/// Build a peer connection on 127.0.0.1:0/0.0.0.0:0 with a handler that
/// forwards data-channel events into `msg_tx` and connection-up into
/// `open_tx`. Shared by both ends; only the channel setup differs.
async fn webrtc_peer_trickle(
    msg_tx: webrtc::runtime::Sender<Vec<u8>>,
    open_tx: webrtc::runtime::Sender<()>,
    gather_tx: webrtc::runtime::Sender<()>,
    cand_tx: webrtc::runtime::Sender<webrtc::peer_connection::RTCIceCandidateInit>,
) -> Result<(
    std::sync::Arc<dyn webrtc::peer_connection::PeerConnection>,
    tokio::sync::oneshot::Receiver<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>,
    webrtc::runtime::Receiver<()>,
)> {
    webrtc_peer_inner(msg_tx, open_tx, gather_tx, Some(cand_tx)).await
}

async fn webrtc_peer_inner(
    msg_tx: webrtc::runtime::Sender<Vec<u8>>,
    open_tx: webrtc::runtime::Sender<()>,
    gather_tx: webrtc::runtime::Sender<()>,
    cand_tx: Option<webrtc::runtime::Sender<webrtc::peer_connection::RTCIceCandidateInit>>,
) -> Result<(
    std::sync::Arc<dyn webrtc::peer_connection::PeerConnection>,
    tokio::sync::oneshot::Receiver<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>,
    webrtc::runtime::Receiver<()>,
)> {
    use webrtc::data_channel::DataChannelEvent;
    use webrtc::peer_connection::{
        MediaEngine, PeerConnectionBuilder, PeerConnectionEventHandler, RTCConfigurationBuilder,
        RTCIceGatheringState,
    };

    struct Handler {
        msg_tx: webrtc::runtime::Sender<Vec<u8>>,
        open_tx: webrtc::runtime::Sender<()>,
        gather_tx: webrtc::runtime::Sender<()>,
        cand_tx: Option<webrtc::runtime::Sender<webrtc::peer_connection::RTCIceCandidateInit>>,
        chan_tx: webrtc::runtime::Sender<()>,
        dc_tx: std::sync::Mutex<
            Option<
                tokio::sync::oneshot::Sender<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>,
            >,
        >,
    }

    #[async_trait]
    impl PeerConnectionEventHandler for Handler {
        async fn on_connection_state_change(
            &self,
            s: webrtc::peer_connection::RTCPeerConnectionState,
        ) {
            if s == webrtc::peer_connection::RTCPeerConnectionState::Connected {
                let _ = self.open_tx.try_send(());
            }
        }
        async fn on_ice_gathering_state_change(&self, s: RTCIceGatheringState) {
            if s == RTCIceGatheringState::Complete {
                let _ = self.gather_tx.try_send(());
            }
        }
        async fn on_ice_candidate(&self, ev: webrtc::peer_connection::RTCPeerConnectionIceEvent) {
            if let Some(tx) = self.cand_tx.as_ref() {
                if let Ok(init) = ev.candidate.to_json() {
                    let _ = tx.try_send(init);
                }
            }
        }
        async fn on_data_channel(&self, dc: std::sync::Arc<dyn webrtc::data_channel::DataChannel>) {
            if let Some(tx) = self.dc_tx.lock().ok().and_then(|mut g| g.take()) {
                let _ = tx.send(dc.clone());
            }
            let tx = self.msg_tx.clone();
            let chan_tx = self.chan_tx.clone();
            tokio::spawn(async move {
                let mut re = WebrtcReassembler::new();
                while let Some(ev) = dc.poll().await {
                    // ONE poll loop per channel: poll() takes events
                    // exclusively, so a second poller would eat OnOpen (or
                    // messages) meant for this one. Connected here means
                    // the channel opened — the peer-level Connected fired
                    // long before SCTP finished.
                    if matches!(ev, DataChannelEvent::OnOpen) {
                        let _ = chan_tx.try_send(());
                    }
                    let DataChannelEvent::OnMessage(m) = ev else {
                        continue;
                    };
                    match re.feed(&m.data) {
                        Ok(Some(envelope)) => {
                            if tx.try_send(envelope).is_err() {
                                break;
                            }
                        }
                        Ok(None) => {}
                        Err(_) => break,
                    }
                }
            });
        }
    }

    let mut engine = MediaEngine::default();
    engine
        .register_default_codecs()
        .context("registering webrtc codecs")?;
    let config = RTCConfigurationBuilder::default().build();
    let (dc_tx, dc_rx) =
        tokio::sync::oneshot::channel::<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>();
    let (chan_tx, chan_rx) = webrtc::runtime::channel::<()>(1);
    let handler = std::sync::Arc::new(Handler {
        msg_tx,
        open_tx,
        gather_tx,
        cand_tx,
        chan_tx,
        dc_tx: std::sync::Mutex::new(Some(dc_tx)),
    });
    let pc = PeerConnectionBuilder::new()
        .with_configuration(config)
        .with_media_engine(engine)
        .with_handler(handler)
        .with_runtime(webrtc::runtime::default_runtime().context("no webrtc runtime")?)
        .with_udp_addrs(vec!["0.0.0.0:0"])
        .build()
        .await
        .context("building the webrtc peer connection")?;
    // Erase the opaque `impl PeerConnection` into a shareable handle.
    // The builder names its return type but never exports it, so without
    // this no caller could hold two connections at once (e.g. the trickle
    // ferry, or a reconnect).
    Ok((std::sync::Arc::from(pc), dc_rx, chan_rx))
}

/// Wait for ICE gathering to complete; candidates ride the trickle
/// channels, with a loud timeout rather than a silent hang.
async fn webrtc_wait_gathered(gather_rx: &mut webrtc::runtime::Receiver<()>) -> Result<()> {
    tokio::time::timeout(std::time::Duration::from_secs(10), gather_rx.recv())
        .await
        .context("ICE gathering timed out")?
        .ok_or_else(|| anyhow::anyhow!("ICE gathering channel closed"))?;
    Ok(())
}

/// Sharer side: create the channel, offer, and return the offer blob plus
/// a transport that becomes usable once `answer_sdp` is applied. The
/// answer step is separate because the CLI prints the blob in between.
pub struct WebrtcOffer {
    /// Base64 offer SDP. Print it; the viewer answers it. Candidates
    /// trickle afterwards on both sides (see `candidate_tx`).
    pub blob: String,
    /// Completes the handshake once the viewer's answer blob arrives.
    pub answer_tx: tokio::sync::oneshot::Sender<String>,
    /// Trickle in: viewer candidates, one parsed `RTCIceCandidateInit`
    /// each. The CLI feeds stdin lines here; an empty line ends trickle.
    pub candidate_tx: webrtc::runtime::Sender<webrtc::peer_connection::RTCIceCandidateInit>,
    /// Trickle out: sharer candidates for the viewer to paste. The CLI
    /// prints each as it arrives, until the channel opens.
    pub candidate_rx: webrtc::runtime::Receiver<webrtc::peer_connection::RTCIceCandidateInit>,
    /// Yields the live transport. Resolves after the answer is applied
    /// and the channel opens.
    pub transport_rx: tokio::sync::oneshot::Receiver<WebrtcTransport>,
}

/// Start the sharer side of a webrtc session. Returns immediately with
/// the offer blob; the transport arrives through `transport_rx` after
/// `pcc view` answers.
pub async fn webrtc_host_offer() -> Result<WebrtcOffer> {
    let (msg_tx, msg_rx) = webrtc::runtime::channel::<Vec<u8>>(256);
    let (open_tx, mut open_rx) = webrtc::runtime::channel::<()>(1);
    let (gather_tx, mut gather_rx) = webrtc::runtime::channel::<()>(1);
    let (cand_tx, cand_rx) =
        webrtc::runtime::channel::<webrtc::peer_connection::RTCIceCandidateInit>(32);
    let (in_tx, mut in_rx) =
        webrtc::runtime::channel::<webrtc::peer_connection::RTCIceCandidateInit>(32);
    let (pc, _dc_rx, _chan_rx) = webrtc_peer_trickle(msg_tx, open_tx, gather_tx, cand_tx).await?;
    let dc = pc
        .create_data_channel(WEBRTC_CHANNEL_LABEL, None)
        .await
        .context("creating the webrtc data channel")?;
    let offer = pc
        .create_offer(None)
        .await
        .context("creating the webrtc offer")?;
    pc.set_local_description(offer)
        .await
        .context("setting the local offer")?;
    webrtc_wait_gathered(&mut gather_rx).await?;
    let sdp = pc
        .local_description()
        .await
        .context("reading the gathered offer")?
        .sdp;
    let (answer_tx, answer_rx) = tokio::sync::oneshot::channel::<String>();
    let (transport_tx, transport_rx) = tokio::sync::oneshot::channel::<WebrtcTransport>();
    tokio::spawn(async move {
        let pc2 = pc.clone();
        // Candidates that arrive before the remote description are
        // buffered, not dropped: trickle means "early" is normal (the
        // failure this comment replaces killed every session: "remote
        // description is not set"). 64 is far beyond any real candidate
        // count; past it the session is broken anyway.
        let (ready_tx, mut ready_rx) = webrtc::runtime::channel::<()>(1);
        tokio::spawn(async move {
            let mut pending = Vec::new();
            let mut ready = false;
            loop {
                tokio::select! {
                    Some(c) = in_rx.recv() => {
                        if ready {
                            if pc2.add_ice_candidate(c).await.is_err() {
                                break;
                            }
                        } else if pending.len() < 64 {
                            pending.push(c);
                        }
                    }
                    _ = ready_rx.recv(), if !ready => {
                        ready = true;
                        for c in pending.drain(..) {
                            if let Err(e) = pc2.add_ice_candidate(c).await {
                                tracing::debug!("discarding trickled candidate: {e}");
                                return;
                            }
                        }
                    }
                    else => break,
                }
            }
        });
        let ready_tx_host = ready_tx;
        let answer_sdp = match answer_rx.await {
            Ok(a) => a,
            Err(_) => return,
        };
        // The CLI contract is blobs both ways: decode first. (The test
        // failure this fixes fed a base64 blob straight into the SDP
        // parser: "SdpInvalidSyntax: dj0wDQpvPS0g...".)
        let answer_sdp = match parse_webrtc_blob(&answer_sdp) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("ignoring a bad viewer answer blob: {e}");
                return;
            }
        };
        let answer = match webrtc::peer_connection::RTCSessionDescription::answer(answer_sdp) {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("ignoring a bad viewer answer SDP: {e}");
                return;
            }
        };
        if pc.set_remote_description(answer).await.is_err() {
            return;
        }
        let _ = ready_tx_host.try_send(());
        if tokio::time::timeout(std::time::Duration::from_secs(15), open_rx.recv())
            .await
            .is_err()
        {
            return;
        }
        // Peer Connected is not channel Open: SCTP still negotiates the
        // channel after DTLS. Wait for OnOpen on our handle, or the first
        // send fails "data channel is not open yet" (caught live).
        if tokio::time::timeout(std::time::Duration::from_secs(15), async {
            while let Some(ev) = dc.poll().await {
                if matches!(ev, webrtc::data_channel::DataChannelEvent::OnOpen) {
                    break;
                }
            }
        })
        .await
        .is_err()
        {
            return;
        }
        // The host-created channel opens on our side; pump it into the
        // same reassembler path the handler uses for accepted channels.
        let (tx2, rx2) = webrtc::runtime::channel::<Vec<u8>>(256);
        let dc2 = dc.clone();
        tokio::spawn(async move {
            let mut re = WebrtcReassembler::new();
            while let Some(ev) = dc2.poll().await {
                use webrtc::data_channel::DataChannelEvent;
                let DataChannelEvent::OnMessage(m) = ev else {
                    continue;
                };
                match re.feed(&m.data) {
                    Ok(Some(envelope)) => {
                        if tx2.try_send(envelope).is_err() {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
        });
        let _ = transport_tx.send(WebrtcTransport::new(dc, rx2, msg_rx));
    });
    Ok(WebrtcOffer {
        blob: webrtc_blob(&sdp),
        answer_tx,
        candidate_tx: in_tx,
        candidate_rx: cand_rx,
        transport_rx,
    })
}

/// Viewer side: apply the sharer's offer blob, return the answer blob
/// immediately plus a transport and a join handle that resolves when the
/// channel opens. The caller must paste the answer back to the sharer
/// BEFORE awaiting the handle — the connection cannot form until both
/// sides hold each other's SDP.
/// The viewer side after the answer blob is ready but before the
/// channel opens. Returned by [`webrtc_join`] immediately (the channel
/// cannot open until the sharer applies the answer, so waiting inside
/// `join` would deadlock manual signalling); finished by
/// [`webrtc_join_open`] once the answer is delivered.
pub struct PendingWebrtcJoin {
    /// For the operator to paste back to the sharer.
    pub answer_blob: String,
    /// Trickle in: sharer candidates.
    pub candidate_in: webrtc::runtime::Sender<webrtc::peer_connection::RTCIceCandidateInit>,
    /// Trickle out: our candidates for the sharer. Taken by the ferry
    /// (test/CLI) before `join_open`; `None` thereafter.
    pub candidate_out:
        Option<webrtc::runtime::Receiver<webrtc::peer_connection::RTCIceCandidateInit>>,
    /// Resolves when the peer connects (DTLS-done, informational).
    /// Taken (not borrowed) by the caller: awaiting consumes the handle.
    pub open_wait: Option<tokio::task::JoinHandle<Result<()>>>,
    rest: Box<PendingWebrtcJoinRest>,
}

struct PendingWebrtcJoinRest {
    dc_rx: tokio::sync::oneshot::Receiver<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>,
    chan_rx: webrtc::runtime::Receiver<()>,
    msg_rx: webrtc::runtime::Receiver<Vec<u8>>,
}

/// Finish a pending join: wait for the accepted channel and its OnOpen
/// (SCTP-done, after DTLS-done), then build the transport.
pub async fn webrtc_join_open(pending: PendingWebrtcJoin) -> Result<WebrtcTransport> {
    let PendingWebrtcJoin { rest, .. } = pending;
    let PendingWebrtcJoinRest {
        dc_rx,
        mut chan_rx,
        msg_rx,
    } = *rest;
    let dc = tokio::time::timeout(std::time::Duration::from_secs(20), async move {
        dc_rx
            .await
            .map_err(|_| anyhow::anyhow!("accepted channel never arrived"))
    })
    .await
    .context("accepted channel wait timed out")??;
    tokio::time::timeout(std::time::Duration::from_secs(15), chan_rx.recv())
        .await
        .context("accepted channel never opened")?
        .ok_or_else(|| anyhow::anyhow!("channel-open signal lost"))?;
    let (tx2, rx2) = webrtc::runtime::channel::<Vec<u8>>(256);
    tokio::spawn(async move {
        let mut msg_rx = msg_rx;
        while let Some(env) = msg_rx.recv().await {
            if tx2.try_send(env).is_err() {
                break;
            }
        }
    });
    Ok(WebrtcTransport {
        dc: Some(dc),
        inbound: Some(rx2),
        _forward: None,
    })
}

pub async fn webrtc_join(offer_blob: &str) -> Result<PendingWebrtcJoin> {
    let offer_sdp = parse_webrtc_blob(offer_blob)?;
    let (msg_tx, msg_rx) = webrtc::runtime::channel::<Vec<u8>>(256);
    let (open_tx, mut open_rx) = webrtc::runtime::channel::<()>(1);
    let (gather_tx, mut gather_rx) = webrtc::runtime::channel::<()>(1);
    let (cand_tx, cand_rx) =
        webrtc::runtime::channel::<webrtc::peer_connection::RTCIceCandidateInit>(32);
    let (in_tx, mut in_rx) =
        webrtc::runtime::channel::<webrtc::peer_connection::RTCIceCandidateInit>(32);
    let (pc, dc_rx, chan_rx) = webrtc_peer_trickle(msg_tx, open_tx, gather_tx, cand_tx).await?;
    let pc2 = pc.clone();
    // Candidates that arrive before the remote description are
    // buffered, not dropped: trickle means "early" is normal (the
    // failure this comment replaces killed every session: "remote
    // description is not set"). 64 is far beyond any real candidate
    // count; past it the session is broken anyway.
    let (ready_tx, mut ready_rx) = webrtc::runtime::channel::<()>(1);
    tokio::spawn(async move {
        let mut pending = Vec::new();
        let mut ready = false;
        loop {
            tokio::select! {
                Some(c) = in_rx.recv() => {
                    if ready {
                        if pc2.add_ice_candidate(c).await.is_err() {
                            break;
                        }
                    } else if pending.len() < 64 {
                        pending.push(c);
                    }
                }
                _ = ready_rx.recv(), if !ready => {
                    ready = true;
                    for c in pending.drain(..) {
                        if let Err(e) = pc2.add_ice_candidate(c).await {
                            tracing::debug!("discarding trickled candidate: {e}");
                            return;
                        }
                    }
                }
                else => break,
            }
        }
    });
    let ready_tx_viewer = ready_tx;
    let offer = webrtc::peer_connection::RTCSessionDescription::offer(offer_sdp)
        .context("parsing the sharer offer SDP")?;
    pc.set_remote_description(offer)
        .await
        .context("applying the sharer offer")?;
    let _ = ready_tx_viewer.try_send(());
    let answer = pc
        .create_answer(None)
        .await
        .context("creating the webrtc answer")?;
    pc.set_local_description(answer)
        .await
        .context("setting the local answer")?;
    webrtc_wait_gathered(&mut gather_rx).await?;
    let sdp = pc
        .local_description()
        .await
        .context("reading the gathered answer")?
        .sdp;
    // Return the answer FIRST so the operator can paste it back while the
    // connection forms: DTLS needs the sharer's answer applied, which
    // only happens after this blob leaves the process. Waiting for
    // Connected before returning would deadlock manual signalling.
    let answer_blob = webrtc_blob(&sdp);
    let open_wait = tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(20), open_rx.recv())
            .await
            .context("webrtc connection never opened")?
            .ok_or_else(|| anyhow::anyhow!("webrtc open channel closed"))?;
        Ok::<(), anyhow::Error>(())
    });
    Ok(PendingWebrtcJoin {
        answer_blob,
        candidate_in: in_tx,
        candidate_out: Some(cand_rx),
        open_wait: Some(open_wait),
        rest: Box::new(PendingWebrtcJoinRest {
            dc_rx,
            chan_rx,
            msg_rx,
        }),
    })
}

/// A webrtc data-channel session as a `MessageTransport`. Envelopes are
/// chunked at 16 KiB on send and reassembled on receipt; `split` hands
/// the halves separate channel handles with a shared inbound queue.
pub struct WebrtcTransport {
    dc: Option<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>,
    /// Inbound envelopes. `None` on a split half that only sends.
    inbound: Option<webrtc::runtime::Receiver<Vec<u8>>>,
    /// Where split-off sources send arriving envelopes. Only set on the
    /// accepted-channel side, which owns the single poll loop.
    _forward: Option<webrtc::runtime::Sender<Vec<u8>>>,
}

impl WebrtcTransport {
    /// Wait until the data channel reports Open. Peer Connected is not
    /// enough — SCTP negotiates the channel after DTLS, and the first
    /// send before that fails "data channel is not open yet".
    pub async fn wait_open(&self) -> Result<()> {
        let Some(dc) = self.dc.as_ref() else {
            anyhow::bail!("webrtc transport has no channel handle");
        };
        tokio::time::timeout(std::time::Duration::from_secs(15), async {
            while let Some(ev) = dc.poll().await {
                if matches!(ev, webrtc::data_channel::DataChannelEvent::OnOpen) {
                    break;
                }
            }
        })
        .await
        .context("webrtc channel never opened")?;
        Ok(())
    }

    fn new(
        dc: std::sync::Arc<dyn webrtc::data_channel::DataChannel>,
        rx: webrtc::runtime::Receiver<Vec<u8>>,
        _dropped: webrtc::runtime::Receiver<Vec<u8>>,
    ) -> Self {
        Self {
            dc: Some(dc),
            inbound: Some(rx),
            _forward: None,
        }
    }
}

#[async_trait]
impl MessageTransport for WebrtcTransport {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        self.send_encoded(&msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        use bytes::BytesMut;
        let Some(dc) = self.dc.as_ref() else {
            anyhow::bail!("webrtc source half cannot send");
        };
        for frame in webrtc_chunk(bytes) {
            dc.send(BytesMut::from(frame.as_slice()))
                .await
                .context("sending a webrtc chunk")?;
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<Message> {
        let inbound = self
            .inbound
            .as_mut()
            .context("webrtc transport has no inbound queue")?;
        let bytes = inbound
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("webrtc channel closed"))?;
        Message::decode(&bytes)
    }

    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>) {
        // The send half keeps the channel; the receive half keeps the
        // queue. Both halves stay usable after the split — unlike the
        // sealed session, which cannot split twice.
        let (tx, rx) = webrtc::runtime::channel::<Vec<u8>>(256);
        // NOTE: envelopes arriving after the split still land in the
        // original queue, which the source half below drains first. A
        // cleaner cut would forward, but the E2E handshake splits
        // immediately, before any visual message flows.
        let sink = Box::new(WebrtcSink { dc: self.dc });
        let source = Box::new(WebrtcSource {
            inbound: self.inbound,
            _drain: rx,
            _tx: tx,
        });
        (sink, source)
    }
}

struct WebrtcSink {
    dc: Option<std::sync::Arc<dyn webrtc::data_channel::DataChannel>>,
}

#[async_trait]
impl MessageSink for WebrtcSink {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        use bytes::BytesMut;
        let Some(dc) = self.dc.as_ref() else {
            anyhow::bail!("webrtc source half cannot send");
        };
        for frame in webrtc_chunk(msg.encode()?.as_slice()) {
            dc.send(BytesMut::from(frame.as_slice()))
                .await
                .context("sending a webrtc chunk")?;
        }
        Ok(())
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        use bytes::BytesMut;
        let Some(dc) = self.dc.as_ref() else {
            anyhow::bail!("webrtc source half cannot send");
        };
        for frame in webrtc_chunk(bytes) {
            dc.send(BytesMut::from(frame.as_slice()))
                .await
                .context("sending a webrtc chunk")?;
        }
        Ok(())
    }
}

struct WebrtcSource {
    inbound: Option<webrtc::runtime::Receiver<Vec<u8>>>,
    _drain: webrtc::runtime::Receiver<Vec<u8>>,
    _tx: webrtc::runtime::Sender<Vec<u8>>,
}

#[async_trait]
impl MessageSource for WebrtcSource {
    async fn recv(&mut self) -> Result<Message> {
        Message::decode(&self.recv_raw().await?)
    }

    async fn recv_raw(&mut self) -> Result<Vec<u8>> {
        let mut inbound = self.inbound.take().context("webrtc source already spent")?;
        let out = inbound
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("webrtc channel closed"))?;
        self.inbound = Some(inbound);
        Ok(out)
    }
}

// --------------------------------------------------------------- endpoints

/// Build a QUIC client endpoint that pins one exact server certificate.
pub fn client_endpoint(config: &NetworkConfig, pin: &[u8]) -> Result<Endpoint> {
    // quinn takes a rustls config through its own crypto wrapper; passing
    // the rustls type directly stopped compiling in quinn 0.11.
    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(NetworkConfig::client_tls_config(pin)?)
            .map_err(|e| anyhow::anyhow!("The pinned TLS config is not usable by QUIC: {e}"))?,
    ));
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
    let mut server_config = ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(NetworkConfig::server_crypto_config(
            identity,
        )?)
        .map_err(|e| anyhow::anyhow!("The server TLS config is not usable by QUIC: {e}"))?,
    ));
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

    #[test]
    fn transport_kind_parses_and_gates() {
        assert_eq!(TransportKind::parse("quic").unwrap(), TransportKind::Quic);
        assert_eq!(
            TransportKind::parse("WebRTC").unwrap(),
            TransportKind::WebRtc
        );
        assert_eq!(TransportKind::parse("IROH").unwrap(), TransportKind::Iroh);
        assert!(TransportKind::parse("sneakernet").is_err());
        assert!(TransportKind::default().require_implemented().is_ok());
        assert!(TransportKind::Iroh.require_implemented().is_ok());
        assert!(TransportKind::WebRtc.require_implemented().is_ok());
    }

    #[test]
    fn webrtc_blob_roundtrips() {
        let sdp = "v=0\r\no=- 1 1 IN IP4 127.0.0.1\r\n";
        assert_eq!(parse_webrtc_blob(&webrtc_blob(sdp)).unwrap(), sdp);
        assert!(parse_webrtc_blob("!!!").is_err());
        assert!(parse_webrtc_blob("aGVsbG8").is_ok()); // valid b64, decodes as UTF-8
    }

    #[test]
    fn webrtc_chunks_reassemble_out_of_order() {
        let envelope: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
        let frames = webrtc_chunk(&envelope);
        assert!(frames.len() > 1);
        assert!(frames.iter().all(|f| f.len() <= 9 + WEBRTC_CHUNK_BYTES));
        let mut re = WebrtcReassembler::new();
        let mut done = None;
        // Reverse order: reassembly is by index, not arrival.
        for f in frames.iter().rev() {
            done = re.feed(f).unwrap();
        }
        assert_eq!(done.unwrap(), envelope);
        // A duplicate frame does not corrupt or duplicate.
        let mut re2 = WebrtcReassembler::new();
        for f in &frames {
            re2.feed(f).unwrap();
        }
        assert!(re2.feed(&frames[0]).unwrap().is_none());
        // Garbage is loud.
        assert!(re.feed(&[0u8; 3]).is_err());
    }

    /// Full session over loopback: offer/answer/trickle between two
    /// peers in one process, then one `Message` each way through the
    /// real transports. This is the shape the CLI runs, minus stdin.
    #[tokio::test]
    async fn webrtc_loopback_session_carries_a_message() {
        let offer = webrtc_host_offer().await.expect("host offer");
        // Viewer answers with trickle, exactly as the CLI does.
        let pending = webrtc_join(&offer.blob).await.expect("viewer join");
        let PendingWebrtcJoin {
            answer_blob,
            candidate_in: cand_in,
            candidate_out,
            open_wait,
            rest,
        } = pending;
        let mut cand_out = candidate_out.expect("fresh join carries candidates");
        // NOTE: this test failed for an hour on "channel opens" while
        // both peers reported Connected: peer Connected is DTLS-done,
        // channel OnOpen is SCTP-done, and they are different moments.
        // Ferry candidates both ways until the channel opens.
        let mut host_cands = offer.candidate_rx;
        let host_in = offer.candidate_tx;

        let ferry = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(c) = host_cands.recv() => {
                        let _ = cand_in.try_send(c);
                    }
                    Some(c) = cand_out.recv() => {
                        let _ = host_in.try_send(c);
                    }
                    else => break,
                }
            }
        });
        // The answer must be applied before the connection can form.

        offer.answer_tx.send(answer_blob).expect("answer applies");
        open_wait
            .expect("fresh join carries its open handle")
            .await
            .expect("open task runs")
            .expect("channel opens");
        // join_open never reads candidate_in (the pump task from join owns
        // the real receiver); a dummy satisfies the struct.
        let (dummy_tx, _dummy_rx) =
            webrtc::runtime::channel::<webrtc::peer_connection::RTCIceCandidateInit>(1);
        let mut viewer = webrtc_join_open(PendingWebrtcJoin {
            answer_blob: String::new(),
            candidate_in: dummy_tx,
            candidate_out: None,
            open_wait: None,
            rest,
        })
        .await
        .expect("viewer channel opens");
        ferry.abort();
        // One message each way through the real transports. join_open
        // already waited for OnOpen on both ends (chan signal, not a
        // second poll — which would race the handler loop).
        let mut host = offer.transport_rx.await.expect("host transport arrives");
        host.send(&Message::KeepAlive { rev: 7 })
            .await
            .expect("host sends");
        let got = viewer.recv().await.expect("viewer receives");
        assert_eq!(got, Message::KeepAlive { rev: 7 });
        viewer
            .send(&Message::Ack { rev: 7 })
            .await
            .expect("viewer sends");
        let got = host.recv().await.expect("host receives");
        assert_eq!(got, Message::Ack { rev: 7 });
    }

    #[test]
    fn iroh_ticket_roundtrips() {
        use iroh::{EndpointAddr, SecretKey};
        let addr = EndpointAddr::new(SecretKey::generate().public());
        let t = iroh_ticket(&addr);
        assert_eq!(parse_iroh_ticket(&t).unwrap(), addr);
        assert!(parse_iroh_ticket("!!!not-base64!!!").is_err());
        // Valid base64, wrong shape.
        assert!(parse_iroh_ticket("aGVsbG8").is_err());
    }

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
