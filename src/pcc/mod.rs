pub mod types;
pub use types::{rgb_len, Frame, PixelChange, PixelChangeDetector, QualityConfig, MAX_FRAME_BYTES};

mod compositor;
pub use compositor::{rect_is_inside, ApplyError, Compositor, Rejected};

mod detector;
pub use detector::{
    apply_changes, changed_fraction, splice, tile_hashes, verify_displacement, Grid, PCCDetector,
    MERGE_EXPANSION_LIMIT,
};

mod planner;
pub use planner::*;
