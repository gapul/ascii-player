use crate::decoder::VideoFrame;
use crate::cli::ColorPalette;
use anyhow::Result;
use log::debug;

/// Represents an ASCII frame with characters and colors
#[derive(Debug, Clone)]
pub struct AsciiFrame {
    /// ASCII characters for each position
    pub characters: Vec<char>,
    /// Foreground colors for each position (RGB)
    pub fg_colors: Vec<(u8, u8, u8)>,
    /// Background colors for each position (RGB) - Optional
    pub bg_colors: Option<Vec<(u8, u8, u8)>>,
    /// Frame width in characters
    pub width: u16,
    /// Frame height in characters
    pub height: u16,
    /// Original timestamp
    pub timestamp: f64,
    /// Frame number
    pub frame_number: u64,
}

/// ASCII conversion configuration
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Character palette to use
    pub palette: ColorPalette,
    /// Whether to use transparent background
    pub transparent: bool,
    /// Alpha threshold for transparency (0-255)
    pub alpha_threshold: Option<u8>,
    /// Custom ASCII character set
    pub ascii_chars: Vec<char>,
    /// Aspect ratio correction factor
    pub aspect_ratio: f64,
    /// Brightness adjustment (-1.0 to 1.0)
    pub brightness: f64,
    /// Contrast adjustment (0.0 to 2.0, 1.0 = normal)
    pub contrast: f64,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            palette: ColorPalette::Color,
            transparent: false,
            alpha_threshold: None,
            ascii_chars: vec![' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'],
            aspect_ratio: 0.5, // Terminal characters are typically twice as tall as wide
            brightness: 0.0,
            contrast: 1.0,
        }
    }
}

/// Video frame to ASCII converter
pub struct FrameConverter {
    config: ConversionConfig,
}

impl FrameConverter {
    /// Create a new frame converter with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        Self { config }
    }
    
    /// Convert a video frame to ASCII representation
    pub fn convert_frame(
        &self,
        frame: &VideoFrame,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Result<AsciiFrame> {
        debug!("Converting frame {}x{} to terminal {}x{}", 
               frame.width, frame.height, terminal_width, terminal_height);
        
        // Calculate target dimensions with aspect ratio correction
        let (target_width, target_height) = self.calculate_target_dimensions(
            frame.width, frame.height, 
            terminal_width, terminal_height
        );
        
        debug!("Target dimensions: {}x{}", target_width, target_height);
        
        // Resize frame data
        let resized_data = self.resize_frame_data(
            &frame.data, 
            frame.width, frame.height,
            target_width as u32, target_height as u32
        )?;
        
        // Convert pixels to ASCII
        let mut characters = Vec::with_capacity((target_width * target_height) as usize);
        let mut fg_colors = Vec::with_capacity((target_width * target_height) as usize);
        let mut bg_colors = if self.config.transparent {
            None
        } else {
            Some(Vec::with_capacity((target_width * target_height) as usize))
        };
        
        for y in 0..target_height {
            for x in 0..target_width {
                let pixel_index = ((y * target_width + x) * 3) as usize;
                
                if pixel_index + 2 < resized_data.len() {
                    let r = resized_data[pixel_index];
                    let g = resized_data[pixel_index + 1];
                    let b = resized_data[pixel_index + 2];
                    
                    // Apply brightness and contrast adjustments
                    let (adj_r, adj_g, adj_b) = self.adjust_color(r, g, b);
                    
                    // Calculate luminance for ASCII character selection
                    let luminance = self.calculate_luminance(adj_r, adj_g, adj_b);
                    
                    // Check alpha threshold if configured
                    if let Some(threshold) = self.config.alpha_threshold {
                        let alpha = (adj_r as u16 + adj_g as u16 + adj_b as u16) / 3;
                        if alpha < threshold as u16 {
                            characters.push(' ');
                            fg_colors.push((0, 0, 0));
                            if let Some(ref mut bg) = bg_colors {
                                bg.push((0, 0, 0));
                            }
                            continue;
                        }
                    }
                    
                    // Select ASCII character based on luminance
                    let char_index = self.luminance_to_char_index(luminance);
                    let ascii_char = self.config.ascii_chars[char_index];
                    
                    characters.push(ascii_char);
                    
                    // Set colors based on palette
                    match self.config.palette {
                        ColorPalette::Ascii => {
                            fg_colors.push((255, 255, 255)); // White text
                            if let Some(ref mut bg) = bg_colors {
                                bg.push((0, 0, 0)); // Black background
                            }
                        }
                        ColorPalette::Grayscale => {
                            let gray = luminance;
                            fg_colors.push((gray, gray, gray));
                            if let Some(ref mut bg) = bg_colors {
                                bg.push((0, 0, 0)); // Black background
                            }
                        }
                        ColorPalette::Color => {
                            fg_colors.push((adj_r, adj_g, adj_b));
                            if let Some(ref mut bg) = bg_colors {
                                // Use a darker version of the color for background
                                bg.push((adj_r / 4, adj_g / 4, adj_b / 4));
                            }
                        }
                    }
                } else {
                    // Handle edge case for incomplete pixel data
                    characters.push(' ');
                    fg_colors.push((0, 0, 0));
                    if let Some(ref mut bg) = bg_colors {
                        bg.push((0, 0, 0));
                    }
                }
            }
        }
        
        Ok(AsciiFrame {
            characters,
            fg_colors,
            bg_colors,
            width: target_width,
            height: target_height,
            timestamp: frame.timestamp,
            frame_number: frame.frame_number,
        })
    }
    
    /// Calculate target dimensions maintaining aspect ratio
    fn calculate_target_dimensions(
        &self,
        src_width: u32, src_height: u32,
        term_width: u16, term_height: u16,
    ) -> (u16, u16) {
        let src_aspect = src_width as f64 / src_height as f64;
        let term_aspect = term_width as f64 / (term_height as f64 * self.config.aspect_ratio);
        
        let (target_width, target_height) = if src_aspect > term_aspect {
            // Source is wider, fit to width
            let width = term_width;
            let height = ((term_width as f64 / src_aspect) * self.config.aspect_ratio) as u16;
            (width, height.min(term_height))
        } else {
            // Source is taller, fit to height
            let height = term_height;
            let width = ((term_height as f64 * src_aspect) / self.config.aspect_ratio) as u16;
            (width.min(term_width), height)
        };
        
        (target_width.max(1), target_height.max(1))
    }
    
    /// Resize frame data using simple nearest neighbor scaling
    fn resize_frame_data(
        &self,
        data: &[u8],
        src_width: u32, src_height: u32,
        target_width: u32, target_height: u32,
    ) -> Result<Vec<u8>> {
        let mut resized = Vec::with_capacity((target_width * target_height * 3) as usize);
        
        let x_ratio = src_width as f64 / target_width as f64;
        let y_ratio = src_height as f64 / target_height as f64;
        
        for y in 0..target_height {
            for x in 0..target_width {
                let src_x = (x as f64 * x_ratio) as u32;
                let src_y = (y as f64 * y_ratio) as u32;
                
                let src_index = ((src_y * src_width + src_x) * 3) as usize;
                
                if src_index + 2 < data.len() {
                    resized.push(data[src_index]);     // R
                    resized.push(data[src_index + 1]); // G
                    resized.push(data[src_index + 2]); // B
                } else {
                    resized.push(0); // R
                    resized.push(0); // G
                    resized.push(0); // B
                }
            }
        }
        
        Ok(resized)
    }
    
    /// Calculate luminance from RGB values
    fn calculate_luminance(&self, r: u8, g: u8, b: u8) -> u8 {
        // Use ITU-R BT.709 luma coefficients
        let luminance = 0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64;
        luminance.round().clamp(0.0, 255.0) as u8
    }
    
    /// Convert luminance to ASCII character index
    fn luminance_to_char_index(&self, luminance: u8) -> usize {
        let normalized = luminance as f64 / 255.0;
        let index = (normalized * (self.config.ascii_chars.len() - 1) as f64).round() as usize;
        index.min(self.config.ascii_chars.len() - 1)
    }
    
    /// Apply brightness and contrast adjustments
    fn adjust_color(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let adjust = |value: u8| -> u8 {
            let mut adjusted = value as f64;
            
            // Apply brightness
            adjusted += self.config.brightness * 255.0;
            
            // Apply contrast
            adjusted = (adjusted - 128.0) * self.config.contrast + 128.0;
            
            adjusted.round().clamp(0.0, 255.0) as u8
        };
        
        (adjust(r), adjust(g), adjust(b))
    }
}

/// Convenience function to convert a frame with default settings
pub fn frame_to_ascii(
    frame: &VideoFrame,
    terminal_width: u16,
    terminal_height: u16,
    ascii_chars: &[char],
) -> AsciiFrame {
    let config = ConversionConfig {
        ascii_chars: ascii_chars.to_vec(),
        ..Default::default()
    };
    
    let converter = FrameConverter::new(config);
    converter.convert_frame(frame, terminal_width, terminal_height)
        .expect("Frame conversion should not fail with valid inputs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::VideoFrame;
    
    fn create_test_frame(width: u32, height: u32, r: u8, g: u8, b: u8) -> VideoFrame {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.extend_from_slice(&[r, g, b]);
        }
        VideoFrame {
            data,
            width,
            height,
            timestamp: 0.0,
            frame_number: 1,
        }
    }
    
    #[test]
    fn test_luminance_calculation() {
        let converter = FrameConverter::new(ConversionConfig::default());
        
        // Test pure white
        assert_eq!(converter.calculate_luminance(255, 255, 255), 255);
        
        // Test pure black
        assert_eq!(converter.calculate_luminance(0, 0, 0), 0);
        
        // Test pure red (should be darker than white)
        let red_luma = converter.calculate_luminance(255, 0, 0);
        assert!(red_luma < 255);
        assert!(red_luma > 0);
    }
    
    #[test]
    fn test_char_index_mapping() {
        let converter = FrameConverter::new(ConversionConfig::default());
        
        // Test extremes
        assert_eq!(converter.luminance_to_char_index(0), 0);
        assert_eq!(converter.luminance_to_char_index(255), converter.config.ascii_chars.len() - 1);
        
        // Test middle value
        let mid_index = converter.luminance_to_char_index(128);
        assert!(mid_index < converter.config.ascii_chars.len());
    }
    
    #[test]
    fn test_frame_conversion() {
        let config = ConversionConfig::default();
        let converter = FrameConverter::new(config);
        
        // Create a simple 2x2 black frame
        let frame = create_test_frame(2, 2, 0, 0, 0);
        
        let ascii_frame = converter.convert_frame(&frame, 10, 10).unwrap();
        
        assert!(ascii_frame.width > 0);
        assert!(ascii_frame.height > 0);
        assert_eq!(ascii_frame.characters.len(), (ascii_frame.width * ascii_frame.height) as usize);
        assert_eq!(ascii_frame.fg_colors.len(), ascii_frame.characters.len());
    }
    
    #[test]
    fn test_aspect_ratio_calculation() {
        let converter = FrameConverter::new(ConversionConfig::default());
        
        // Test square source to wider terminal
        let (w, h) = converter.calculate_target_dimensions(100, 100, 80, 20);
        assert!(w <= 80);
        assert!(h <= 20);
        
        // Test wide source to square terminal
        let (w, h) = converter.calculate_target_dimensions(200, 100, 40, 40);
        assert!(w <= 40);
        assert!(h <= 40);
    }
}