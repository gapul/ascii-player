mod cli;
mod decoder;
mod converter;
mod renderer;

use cli::Cli;
use decoder::load_video;
use converter::{FrameConverter, ConversionConfig};
use renderer::{Renderer, calculate_frame_delay};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use anyhow::Result;
use log::{info, debug, error, warn};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// SketchyBar integration helper
struct SketchyBarIntegration {
    item_name: String,
}

impl SketchyBarIntegration {
    fn new(item_name: String) -> Self {
        Self { item_name }
    }
    
    fn update_status(&self, status: &str) -> Result<()> {
        let output = Command::new("sketchybar")
            .args(&["--set", &self.item_name, "label", status])
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    debug!("SketchyBar updated: {} -> {}", self.item_name, status);
                } else {
                    warn!("SketchyBar command failed: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                warn!("Failed to execute sketchybar command: {}", e);
            }
        }
        Ok(())
    }
    
    fn set_playing(&self, filename: &str) -> Result<()> {
        let status = format!("▶ {}", filename);
        self.update_status(&status)
    }
    
    fn set_paused(&self, filename: &str) -> Result<()> {
        let status = format!("⏸ {}", filename);
        self.update_status(&status)
    }
    
    fn clear(&self) -> Result<()> {
        self.update_status("")
    }
}

impl Drop for SketchyBarIntegration {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

/// Application state for playback control
#[derive(Debug, Clone)]
struct PlaybackState {
    paused: bool,
    speed: f64,
    loop_enabled: bool,
    quit_requested: bool,
    show_help: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1.0,
            loop_enabled: false,
            quit_requested: false,
            show_help: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Validate CLI arguments
    if let Err(e) = cli.validate() {
        error!("Invalid arguments: {}", e);
        std::process::exit(1);
    }
    
    // Set up logging level
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }
    
    info!("Starting ASCII Player v{}", env!("CARGO_PKG_VERSION"));
    info!("Playing: {}", cli.file_path.display());
    
    // Initialize SketchyBar integration if configured
    let sketchybar = if let Some(item_name) = cli.sketchybar_item_name() {
        Some(SketchyBarIntegration::new(item_name.to_string()))
    } else {
        None
    };
    
    // Set up playback state
    let mut state = PlaybackState {
        speed: cli.speed,
        loop_enabled: cli.loop_playback,
        ..Default::default()
    };
    
    // If info-only mode, skip terminal initialization and just get video info
    if cli.info_only {
        info!("Info-only mode: loading video information");
        let frame_iter = load_video(&cli.file_path, cli.start_time, cli.end_time)?;
        
        let video_fps = frame_iter.decoder().fps();
        let video_duration = frame_iter.decoder().duration();
        let (video_width, video_height) = frame_iter.decoder().dimensions();
        
        println!("Video Information:");
        println!("  File: {}", cli.file_path.display());
        println!("  Dimensions: {}x{}", video_width, video_height);
        println!("  Frame Rate: {:.2} FPS", video_fps);
        println!("  Duration: {:.2} seconds", video_duration);
        println!("  Aspect Ratio: {:.2}", video_width as f64 / video_height as f64);
        return Ok(());
    }

    // Create renderer
    let mut renderer = Renderer::new(cli.transparent, cli.use_color())?;
    renderer.init()?;
    
    // Show loading screen
    renderer.display_loading("Loading video...")?;
    
    // Load video
    let mut frame_iter = match load_video(&cli.file_path, cli.start_time, cli.end_time) {
        Ok(iter) => iter,
        Err(e) => {
            renderer.display_error(&format!("Failed to load video: {}", e))?;
            tokio::time::sleep(Duration::from_secs(3)).await;
            return Err(e);
        }
    };
    
    // Get video information
    let video_fps = frame_iter.decoder().fps();
    let video_duration = frame_iter.decoder().duration();
    let (video_width, video_height) = frame_iter.decoder().dimensions();
    
    info!("Video info: {}x{}, {:.2} FPS, {:.2}s duration", 
          video_width, video_height, video_fps, video_duration);
    
    // Set up frame converter
    let conversion_config = ConversionConfig {
        palette: cli.palette.clone(),
        transparent: cli.transparent,
        alpha_threshold: cli.alpha_threshold,
        ascii_chars: cli.get_ascii_chars().to_vec(),
        ..Default::default()
    };
    let converter = FrameConverter::new(conversion_config);
    
    // Get filename for status display
    let filename = cli.file_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    
    // Update SketchyBar
    if let Some(ref sb) = sketchybar {
        sb.set_playing(filename)?;
    }
    
    // Main playback loop
    let mut frame_count = 0u64;
    let playback_start = Instant::now();
    let effective_fps = cli.fps.unwrap_or(video_fps);
    
    loop {
        // Handle input events
        if event::poll(Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            info!("Quit requested by user");
                            state.quit_requested = true;
                            break;
                        }
                        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            info!("Ctrl+C pressed, exiting");
                            state.quit_requested = true;
                            break;
                        }
                        KeyCode::Char(' ') => {
                            state.paused = !state.paused;
                            if state.paused {
                                info!("Playback paused");
                                if let Some(ref sb) = sketchybar {
                                    sb.set_paused(filename)?;
                                }
                            } else {
                                info!("Playback resumed");
                                if let Some(ref sb) = sketchybar {
                                    sb.set_playing(filename)?;
                                }
                            }
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            state.speed = (state.speed * 1.25).min(4.0);
                            info!("Speed increased to {:.2}x", state.speed);
                        }
                        KeyCode::Char('-') => {
                            state.speed = (state.speed / 1.25).max(0.25);
                            info!("Speed decreased to {:.2}x", state.speed);
                        }
                        KeyCode::Char('l') => {
                            state.loop_enabled = !state.loop_enabled;
                            info!("Loop {}", if state.loop_enabled { "enabled" } else { "disabled" });
                        }
                        KeyCode::Char('h') => {
                            state.show_help = !state.show_help;
                        }
                        KeyCode::Char('r') => {
                            info!("Restarting video from beginning");
                            frame_iter = load_video(&cli.file_path, cli.start_time, cli.end_time)?;
                            frame_count = 0;
                        }
                        _ => {}
                    }
                }
                Event::Resize(width, height) => {
                    debug!("Terminal resized to {}x{}", width, height);
                    renderer.update_dimensions()?;
                }
                _ => {}
            }
        }
        
        // Show help if requested
        if state.show_help {
            let help_text = r#"ASCII Player Controls:

SPACE  - Pause/Resume
Q/ESC  - Quit
+/=    - Increase speed
-      - Decrease speed
L      - Toggle loop
R      - Restart video
H      - Toggle this help

Press H again to hide this help."#;
            
            renderer.display_message(help_text)?;
            continue;
        }
        
        // Skip frame processing if paused
        if state.paused {
            sleep(Duration::from_millis(50)).await;
            continue;
        }
        
        // Get next frame
        let frame = match frame_iter.next() {
            Some(Ok(frame)) => frame,
            Some(Err(e)) => {
                error!("Error reading frame: {}", e);
                renderer.display_error(&format!("Playback error: {}", e))?;
                sleep(Duration::from_secs(2)).await;
                break;
            }
            None => {
                // End of video
                if state.loop_enabled {
                    info!("Video ended, restarting loop");
                    frame_iter = load_video(&cli.file_path, cli.start_time, cli.end_time)?;
                    frame_count = 0;
                    continue;
                } else {
                    info!("Video playback completed");
                    break;
                }
            }
        };
        
        // Get current terminal size
        let (term_width, term_height) = renderer.dimensions();
        
        // Convert frame to ASCII
        let ascii_frame = match converter.convert_frame(&frame, term_width, term_height) {
            Ok(frame) => frame,
            Err(e) => {
                error!("Error converting frame: {}", e);
                continue;
            }
        };
        
        // Create status line
        let elapsed = playback_start.elapsed().as_secs_f64();
        let progress = if video_duration > 0.0 {
            (frame.timestamp / video_duration * 100.0).min(100.0)
        } else {
            0.0
        };
        
        let status = format!(
            "{} | Frame: {} | Time: {:.1}s/{:.1}s ({:.1}%) | Speed: {:.2}x | FPS: {:.1}",
            filename, frame_count, frame.timestamp, video_duration, progress, state.speed, effective_fps
        );
        
        // Render frame with status
        renderer.render_frame_with_status(&ascii_frame, &status)?;
        
        frame_count += 1;
        
        // Calculate frame delay
        let target_fps = effective_fps * state.speed;
        let frame_delay = calculate_frame_delay(target_fps, 1.0);
        
        // Sleep for frame timing
        sleep(frame_delay).await;
    }
    
    // Cleanup
    info!("Cleaning up and exiting");
    renderer.cleanup()?;
    
    // Clear SketchyBar
    if let Some(ref sb) = sketchybar {
        sb.clear()?;
    }
    
    info!("Playback finished. Total frames: {}", frame_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_playback_state_default() {
        let state = PlaybackState::default();
        assert!(!state.paused);
        assert_eq!(state.speed, 1.0);
        assert!(!state.loop_enabled);
        assert!(!state.quit_requested);
        assert!(!state.show_help);
    }
    
    #[tokio::test]
    async fn test_sketchybar_integration() {
        let sb = SketchyBarIntegration::new("test_item".to_string());
        
        // These tests will only work if sketchybar is installed
        // In a real environment, you might want to mock the Command execution
        let result = sb.set_playing("test.mp4");
        assert!(result.is_ok());
        
        let result = sb.clear();
        assert!(result.is_ok());
    }
}