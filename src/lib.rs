//! ASCII Player - A responsive, color-enabled ASCII video player for the terminal
//! 
//! This crate provides functionality to convert video files into ASCII art animations
//! that can be played in the terminal with support for colors, transparency, and
//! responsive resizing.

pub mod cli;
pub mod decoder;
pub mod converter;
pub mod renderer;

pub use cli::{Cli, ColorPalette};
pub use decoder::{VideoDecoder, VideoFrame, FrameIterator, load_video};
pub use converter::{AsciiFrame, ConversionConfig, FrameConverter, frame_to_ascii};
pub use renderer::{Renderer, render_frame, calculate_frame_delay};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

/// Package description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Default ASCII character ramp for luminance mapping
pub const DEFAULT_ASCII_RAMP: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Extended ASCII character ramp with more granular detail
pub const EXTENDED_ASCII_RAMP: &[char] = &[
    ' ', '`', '.', '\'', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', 
    '~', '+', '_', '-', '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 
    't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 
    'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a', 'o', 
    '*', '#', 'M', 'W', '&', '8', '%', 'B', '@'
];

/// Block character ramp for a more solid appearance
pub const BLOCK_ASCII_RAMP: &[char] = &[' ', '░', '▒', '▓', '█'];

/// Error types used throughout the application
#[derive(thiserror::Error, Debug)]
pub enum AsciiPlayerError {
    #[error("Video decoding error: {0}")]
    VideoDecoding(#[from] ffmpeg_next::Error),
    
    #[error("Terminal error: {0}")]
    Terminal(#[from] crossterm::ErrorKind),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, AsciiPlayerError>;

/// Utility functions
pub mod utils {
    use super::*;
    
    /// Get the appropriate ASCII character set for a given palette
    pub fn get_ascii_chars(palette: &ColorPalette) -> &'static [char] {
        match palette {
            ColorPalette::Ascii => DEFAULT_ASCII_RAMP,
            ColorPalette::Grayscale => BLOCK_ASCII_RAMP,
            ColorPalette::Color => BLOCK_ASCII_RAMP,
        }
    }
    
    /// Format duration in a human-readable way
    pub fn format_duration(seconds: f64) -> String {
        let total_seconds = seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;
        
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{}:{:02}", minutes, secs)
        }
    }
    
    /// Calculate aspect ratio from dimensions
    pub fn calculate_aspect_ratio(width: u32, height: u32) -> f64 {
        width as f64 / height as f64
    }
    
    /// Clamp a value between min and max
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        Cli, ColorPalette,
        VideoDecoder, VideoFrame, FrameIterator, load_video,
        AsciiFrame, ConversionConfig, FrameConverter, frame_to_ascii,
        Renderer, render_frame, calculate_frame_delay,
        AsciiPlayerError, Result,
        utils::*,
    };
}