use crate::converter::AsciiFrame;
use crossterm::{
    execute, queue,
    style::{Color, Print, SetForegroundColor, SetBackgroundColor, ResetColor},
    cursor::{MoveTo, Hide, Show},
    terminal::{Clear, ClearType, enable_raw_mode, disable_raw_mode},
};
use std::io::{stdout, Write, Stdout};
use anyhow::Result;
use log::debug;

/// Terminal renderer for ASCII frames
pub struct Renderer {
    stdout: Stdout,
    transparent_mode: bool,
    use_colors: bool,
    center_output: bool,
    terminal_width: u16,
    terminal_height: u16,
}

/// Rendering statistics
#[derive(Debug, Default)]
pub struct RenderStats {
    pub frames_rendered: u64,
    pub total_render_time_ms: u64,
    pub average_render_time_ms: f64,
    pub last_render_time_ms: u64,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(transparent_mode: bool, use_colors: bool) -> Result<Self> {
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        
        Ok(Self {
            stdout: stdout(),
            transparent_mode,
            use_colors,
            center_output: true,
            terminal_width,
            terminal_height,
        })
    }
    
    /// Initialize the terminal for rendering
    pub fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(self.stdout, Hide, Clear(ClearType::All))?;
        debug!("Terminal initialized for rendering");
        Ok(())
    }
    
    /// Restore terminal to normal state
    pub fn cleanup(&mut self) -> Result<()> {
        execute!(self.stdout, Show, ResetColor, Clear(ClearType::All))?;
        disable_raw_mode()?;
        debug!("Terminal restored to normal state");
        Ok(())
    }
    
    /// Update terminal dimensions
    pub fn update_dimensions(&mut self) -> Result<(u16, u16)> {
        let (width, height) = crossterm::terminal::size()?;
        self.terminal_width = width;
        self.terminal_height = height;
        debug!("Terminal dimensions updated: {}x{}", width, height);
        Ok((width, height))
    }
    
    /// Get current terminal dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }
    
    /// Render an ASCII frame to the terminal
    pub fn render_frame(&mut self, frame: &AsciiFrame) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Calculate centering offsets
        let (offset_x, offset_y) = if self.center_output {
            let offset_x = (self.terminal_width.saturating_sub(frame.width)) / 2;
            let offset_y = (self.terminal_height.saturating_sub(frame.height)) / 2;
            (offset_x, offset_y)
        } else {
            (0, 0)
        };
        
        // Clear the screen
        queue!(self.stdout, Clear(ClearType::All))?;
        
        // Render frame content
        for y in 0..frame.height {
            for x in 0..frame.width {
                let index = (y * frame.width + x) as usize;
                
                if index < frame.characters.len() {
                    let character = frame.characters[index];
                    let (fg_r, fg_g, fg_b) = frame.fg_colors[index];
                    
                    // Position cursor
                    queue!(self.stdout, MoveTo(offset_x + x, offset_y + y))?;
                    
                    // Skip rendering spaces in transparent mode
                    if self.transparent_mode && character == ' ' {
                        continue;
                    }
                    
                    // Set colors if enabled
                    if self.use_colors {
                        queue!(self.stdout, SetForegroundColor(Color::Rgb { r: fg_r, g: fg_g, b: fg_b }))?;
                        
                        // Set background color if not in transparent mode
                        if !self.transparent_mode {
                            if let Some(ref bg_colors) = frame.bg_colors {
                                if index < bg_colors.len() {
                                    let (bg_r, bg_g, bg_b) = bg_colors[index];
                                    queue!(self.stdout, SetBackgroundColor(Color::Rgb { r: bg_r, g: bg_g, b: bg_b }))?;
                                }
                            }
                        }
                    }
                    
                    // Print the character
                    queue!(self.stdout, Print(character))?;
                }
            }
        }
        
        // Reset colors and flush output
        if self.use_colors {
            queue!(self.stdout, ResetColor)?;
        }
        self.stdout.flush()?;
        
        let render_time = start_time.elapsed().as_millis() as u64;
        debug!("Frame rendered in {}ms ({}x{} -> {}x{} at offset {},{}) ", 
               render_time, frame.width, frame.height, 
               self.terminal_width, self.terminal_height,
               offset_x, offset_y);
        
        Ok(())
    }
    
    /// Render frame with additional status information
    pub fn render_frame_with_status(&mut self, frame: &AsciiFrame, status: &str) -> Result<()> {
        self.render_frame(frame)?;
        
        // Render status line at the bottom
        if !status.is_empty() {
            let status_y = self.terminal_height.saturating_sub(1);
            queue!(self.stdout, MoveTo(0, status_y))?;
            
            if self.use_colors {
                queue!(self.stdout, SetForegroundColor(Color::White))?;
                queue!(self.stdout, SetBackgroundColor(Color::DarkGrey))?;
            }
            
            // Truncate status to fit terminal width
            let truncated_status = if status.len() > self.terminal_width as usize {
                &status[..self.terminal_width as usize]
            } else {
                status
            };
            
            queue!(self.stdout, Print(truncated_status))?;
            
            if self.use_colors {
                queue!(self.stdout, ResetColor)?;
            }
            
            self.stdout.flush()?;
        }
        
        Ok(())
    }
    
    /// Clear the screen
    pub fn clear(&mut self) -> Result<()> {
        execute!(self.stdout, Clear(ClearType::All))?;
        debug!("Screen cleared");
        Ok(())
    }
    
    /// Display a message in the center of the screen
    pub fn display_message(&mut self, message: &str) -> Result<()> {
        let lines: Vec<&str> = message.lines().collect();
        let start_y = (self.terminal_height / 2).saturating_sub(lines.len() as u16 / 2);
        
        execute!(self.stdout, Clear(ClearType::All))?;
        
        for (i, line) in lines.iter().enumerate() {
            let y = start_y + i as u16;
            let x = (self.terminal_width / 2).saturating_sub(line.len() as u16 / 2);
            
            execute!(self.stdout, MoveTo(x, y))?;
            
            if self.use_colors {
                execute!(self.stdout, SetForegroundColor(Color::Yellow))?;
            }
            
            execute!(self.stdout, Print(line))?;
        }
        
        if self.use_colors {
            execute!(self.stdout, ResetColor)?;
        }
        
        debug!("Message displayed: {}", message);
        Ok(())
    }
    
    /// Display loading screen
    pub fn display_loading(&mut self, message: &str) -> Result<()> {
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spinner_index = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() / 100) % spinner_chars.len() as u128;
        
        let spinner = spinner_chars[spinner_index as usize];
        let full_message = format!("{} {}", spinner, message);
        
        self.display_message(&full_message)
    }
    
    /// Display error message
    pub fn display_error(&mut self, error: &str) -> Result<()> {
        execute!(self.stdout, Clear(ClearType::All))?;
        
        let y = self.terminal_height / 2;
        let x = (self.terminal_width / 2).saturating_sub(error.len() as u16 / 2);
        
        execute!(self.stdout, MoveTo(x, y))?;
        
        if self.use_colors {
            execute!(self.stdout, SetForegroundColor(Color::Red))?;
        }
        
        execute!(self.stdout, Print("ERROR: "), Print(error))?;
        
        if self.use_colors {
            execute!(self.stdout, ResetColor)?;
        }
        
        debug!("Error displayed: {}", error);
        Ok(())
    }
    
    /// Enable or disable centering
    pub fn set_centering(&mut self, center: bool) {
        self.center_output = center;
    }
    
    /// Check if renderer is in transparent mode
    pub fn is_transparent(&self) -> bool {
        self.transparent_mode
    }
    
    /// Check if renderer uses colors
    pub fn uses_colors(&self) -> bool {
        self.use_colors
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Ensure terminal is restored on drop
        let _ = self.cleanup();
    }
}

/// Convenience function to render a frame with default settings
pub fn render_frame(frame: &AsciiFrame, transparent_mode: bool) -> Result<()> {
    let mut renderer = Renderer::new(transparent_mode, true)?;
    renderer.init()?;
    renderer.render_frame(frame)?;
    renderer.cleanup()?;
    Ok(())
}

/// Calculate optimal frame rate for smooth playback
pub fn calculate_frame_delay(target_fps: f64, speed_multiplier: f64) -> std::time::Duration {
    let effective_fps = target_fps * speed_multiplier;
    let frame_time_ms = 1000.0 / effective_fps;
    std::time::Duration::from_millis(frame_time_ms as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::AsciiFrame;
    
    fn create_test_frame() -> AsciiFrame {
        AsciiFrame {
            characters: vec!['#', ' ', '@', ' '],
            fg_colors: vec![(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)],
            bg_colors: Some(vec![(0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0)]),
            width: 2,
            height: 2,
            timestamp: 1.0,
            frame_number: 42,
        }
    }
    
    #[test]
    fn test_renderer_creation() {
        let result = Renderer::new(false, true);
        assert!(result.is_ok(), "Should be able to create renderer");
    }
    
    #[test]
    fn test_frame_delay_calculation() {
        let delay = calculate_frame_delay(30.0, 1.0);
        assert_eq!(delay.as_millis(), 33); // ~33ms for 30 FPS
        
        let delay_2x = calculate_frame_delay(30.0, 2.0);
        assert_eq!(delay_2x.as_millis(), 16); // ~16ms for 60 FPS (2x speed)
    }
    
    #[test]
    fn test_transparent_mode() {
        let renderer = Renderer::new(true, true).unwrap();
        assert!(renderer.is_transparent());
        
        let renderer = Renderer::new(false, true).unwrap();
        assert!(!renderer.is_transparent());
    }
    
    #[test]
    fn test_color_mode() {
        let renderer = Renderer::new(false, true).unwrap();
        assert!(renderer.uses_colors());
        
        let renderer = Renderer::new(false, false).unwrap();
        assert!(!renderer.uses_colors());
    }
}