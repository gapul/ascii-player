// Re-export commonly used types for convenience
pub use crate::cli::{Cli, ColorPalette};
pub use crate::converter::{AsciiFrame, ConversionConfig, FrameConverter};
pub use crate::decoder::{load_video, FrameIterator, VideoDecoder, VideoFrame};
pub use crate::renderer::{calculate_frame_delay, Renderer};

// Re-export external types commonly used in tests
pub use anyhow::Result;
pub use std::path::Path;
