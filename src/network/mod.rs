mod config;
pub mod e2e;
mod protocol;
mod resilience;
mod transport;
pub mod web_e2e;
mod wire;

pub use config::{
    fingerprint_hex, generate_identity, hex_to_der, load_identity, relay_credential, verify_token,
    NetworkConfig, ServerIdentity, SessionToken, MAX_AUDIO_DATAGRAM,
};
pub use e2e::SEALED_OVERHEAD;
pub use protocol::{
    peek_rev, read_len_prefix, read_len_prefix_with_slack, Cursor, Epoch, Message, Rev,
    MAX_MESSAGE_SIZE, MAX_OPS_PER_UPDATE, MAX_SNAPSHOT_CHUNKS, MAX_TOKEN_LEN, MESSAGE_FRAMING,
    PROTOCOL_VERSION, SNAPSHOT_CHUNK_BYTES,
};
pub use resilience::{NetworkResilience, ResilienceConfig};
pub use transport::{
    client_endpoint, connect_direct, iroh_dial, iroh_host_endpoint, iroh_ticket, parse_iroh_ticket,
    parse_webrtc_blob, resolve, server_endpoint, webrtc_blob, webrtc_chunk, webrtc_host_offer,
    webrtc_join, webrtc_join_open, write_encoded, Connection, IrohTransport, MessageSink,
    MessageSource, MessageTransport, PendingWebrtcJoin, QuicTransport, SealedSink, SealedSource,
    TransportKind, WebrtcReassembler, WebrtcTransport, IROH_ALPN, WEBRTC_CHANNEL_LABEL,
    WEBRTC_CHUNK_BYTES,
};
pub use wire::{WireOp, OP_COPY, OP_FILL, OP_HEADER_BYTES, OP_RECT};

/// Default port a sharer listens on for direct viewer connections.
pub const DEFAULT_PORT: u16 = 5800;
