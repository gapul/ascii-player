use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the video file to play
    #[arg(required = true)]
    pub file_path: PathBuf,

    /// Loop the video playback
    #[arg(short, long)]
    pub loop_playback: bool,

    /// Set playback speed factor
    #[arg(short, long, default_value_t = 1.0)]
    pub speed: f64,

    /// Enable transparent background by not drawing background colors
    #[arg(short, long)]
    pub transparent: bool,

    /// Enable alpha channel support with a specific threshold (0-255)
    #[arg(short, long, value_name = "THRESHOLD")]
    pub alpha_threshold: Option<u8>,

    /// Set terminal width (override automatic detection)
    #[arg(short, long)]
    pub width: Option<u16>,

    /// Set terminal height (override automatic detection)
    #[arg(long)]
    pub height: Option<u16>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Set color palette (ascii, grayscale, color)
    #[arg(short, long, default_value = "color")]
    pub palette: ColorPalette,

    /// SketchyBar integration - update item with playback status
    #[arg(long, value_name = "ITEM_NAME")]
    pub sketchybar_item: Option<String>,

    /// Frame rate limit (FPS)
    #[arg(short, long)]
    pub fps: Option<f64>,

    /// Start playback from specific time (in seconds)
    #[arg(long)]
    pub start_time: Option<f64>,

    /// Stop playback at specific time (in seconds)
    #[arg(long)]
    pub end_time: Option<f64>,

    /// Show video information only (don't play)
    #[arg(long)]
    pub info_only: bool,

    /// Render a single frame for testing (debug mode)
    #[arg(long)]
    pub single_frame: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ColorPalette {
    /// ASCII characters only (no color)
    Ascii,
    /// Grayscale ASCII
    Grayscale,
    /// Full color ASCII
    Color,
}

impl Cli {
    /// Validate command line arguments
    pub fn validate(&self) -> Result<(), String> {
        // Check if file exists
        if !self.file_path.exists() {
            return Err(format!("Video file does not exist: {}", self.file_path.display()));
        }

        // Validate speed factor
        if self.speed <= 0.0 {
            return Err("Speed factor must be greater than 0".to_string());
        }

        // Validate alpha threshold (u8 is always <= 255, so this check is redundant but kept for clarity)
        if let Some(_threshold) = self.alpha_threshold {
            // u8 is automatically bounded to 0-255, so no additional validation needed
        }

        // Validate terminal dimensions
        if let Some(width) = self.width {
            if width == 0 {
                return Err("Terminal width must be greater than 0".to_string());
            }
        }

        if let Some(height) = self.height {
            if height == 0 {
                return Err("Terminal height must be greater than 0".to_string());
            }
        }

        // Validate FPS
        if let Some(fps) = self.fps {
            if fps <= 0.0 {
                return Err("FPS must be greater than 0".to_string());
            }
        }

        // Validate time range
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            if start >= end {
                return Err("Start time must be less than end time".to_string());
            }
            if start < 0.0 || end < 0.0 {
                return Err("Time values must be non-negative".to_string());
            }
        }

        Ok(())
    }

    /// Get effective terminal dimensions
    pub fn get_terminal_size(&self) -> Result<(u16, u16), std::io::Error> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Ok((w, h)),
            (Some(w), None) => {
                let (_, h) = crossterm::terminal::size()?;
                Ok((w, h))
            }
            (None, Some(h)) => {
                let (w, _) = crossterm::terminal::size()?;
                Ok((w, h))
            }
            (None, None) => crossterm::terminal::size(),
        }
    }

    /// Get the ASCII character set based on palette
    pub fn get_ascii_chars(&self) -> &'static [char] {
        match self.palette {
            ColorPalette::Ascii => &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'],
            ColorPalette::Grayscale => &[' ', '░', '▒', '▓', '█'],
            ColorPalette::Color => &[' ', '░', '▒', '▓', '█'],
        }
    }

    /// Check if color output is enabled
    pub fn use_color(&self) -> bool {
        matches!(self.palette, ColorPalette::Color | ColorPalette::Grayscale)
    }

    /// Get SketchyBar item name if configured
    pub fn sketchybar_item_name(&self) -> Option<&str> {
        self.sketchybar_item.as_deref()
    }
}