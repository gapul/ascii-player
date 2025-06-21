// Re-export commonly used types for convenience
pub use crate::cli::{Cli, ColorPalette};
pub use crate::decoder::{VideoDecoder, VideoFrame, FrameIterator, load_video};
pub use crate::converter::{FrameConverter, ConversionConfig, AsciiFrame};
pub use crate::renderer::{Renderer, calculate_frame_delay};

// Re-export external types commonly used in tests
pub use anyhow::Result;
pub use std::path::Path;