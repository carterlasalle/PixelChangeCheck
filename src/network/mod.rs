mod config;
pub mod e2e;
mod protocol;
mod resilience;
mod transport;
pub mod web_e2e;
mod wire;

pub use config::{
    fingerprint_hex, generate_identity, hex_to_der, relay_credential, verify_token, NetworkConfig,
    ServerIdentity, SessionToken, MAX_AUDIO_DATAGRAM,
};
pub use protocol::{
    peek_rev, read_len_prefix, Cursor, Epoch, Message, Rev, MAX_MESSAGE_SIZE, MAX_OPS_PER_UPDATE,
    MAX_SNAPSHOT_CHUNKS, MAX_TOKEN_LEN, PROTOCOL_VERSION, SNAPSHOT_CHUNK_BYTES,
};
pub use resilience::{NetworkResilience, ResilienceConfig};
pub use transport::{
    client_endpoint, connect_direct, resolve, server_endpoint, write_encoded, Connection,
    MessageSink, MessageSource, MessageTransport, QuicTransport, SealedSink, SealedSource,
};
pub use wire::{WireOp, OP_COPY, OP_FILL, OP_HEADER_BYTES, OP_RECT};

/// Default port a sharer listens on for direct viewer connections.
pub const DEFAULT_PORT: u16 = 5800;
