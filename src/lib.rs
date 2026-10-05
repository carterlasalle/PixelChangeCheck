pub mod app;
pub mod audio;
pub mod capture;
pub mod codec;
pub mod encoder;
pub mod network;
pub mod pcc;
pub mod reach;
pub mod relay;
// The relay browser-viewer surface: an internal split of `relay`, not a
// second public entry point, so it stays crate-private.
pub(crate) mod relay_web;
pub mod server;
pub mod telemetry;

// Re-export commonly used types
pub use capture::{
    CaptureSource, CursorSample, CursorSampler, NoCursorSampler, PlatformCursorSampler,
    ScreenCapture,
};
pub use network::{NetworkConfig, NetworkResilience, ResilienceConfig, SessionToken};
pub use pcc::{PCCDetector, QualityConfig};
pub use server::renderer::SharedSurface;
