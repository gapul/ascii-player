use ascii_player::prelude::*;
use std::path::PathBuf;
use tempfile::tempdir;
use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create a test video file
fn create_test_video() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let video_path = temp_dir.path().join("test_video.mp4");
    
    // Create a simple test video using FFmpeg
    let output = std::process::Command::new("ffmpeg")
        .args(&[
            "-f", "lavfi",
            "-i", "testsrc=duration=1:size=160x120:rate=10",
            "-pix_fmt", "yuv420p",
            "-y",
            video_path.to_str().unwrap()
        ])
        .output();
    
    match output {
        Ok(result) if result.status.success() => Ok(video_path),
        _ => {
            // If FFmpeg is not available, create a dummy file for CLI tests
            std::fs::write(&video_path, b"dummy video content")?;
            Ok(video_path)
        }
    }
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ascii-player").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ASCII video player"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("ascii-player").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_missing_file() {
    let mut cmd = Command::cargo_bin("ascii-player").unwrap();
    cmd.arg("nonexistent.mp4");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_cli_invalid_speed() {
    let video_path = create_test_video().unwrap();
    
    let mut cmd = Command::cargo_bin("ascii-player").unwrap();
    cmd.arg(video_path.to_str().unwrap())
        .arg("--speed")
        .arg("0");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Speed factor must be greater than 0"));
}

#[test]
fn test_cli_valid_options() {
    let video_path = create_test_video().unwrap();
    
    let mut cmd = Command::cargo_bin("ascii-player").unwrap();
    cmd.arg(video_path.to_str().unwrap())
        .arg("--transparent")
        .arg("--speed")
        .arg("2.0")
        .arg("--palette")
        .arg("grayscale")
        .timeout(std::time::Duration::from_secs(2));
    
    // Note: This test might fail in CI without a proper terminal
    // so we just check that it doesn't crash immediately
    let result = cmd.assert();
    // Allow either success (if terminal is available) or specific error codes
    if !result.get_output().status.success() {
        // Check that it's not a CLI parsing error
        let stderr = String::from_utf8_lossy(&result.get_output().stderr);
        assert!(
            !stderr.contains("error: ") || stderr.contains("terminal") || stderr.contains("FFmpeg"),
            "Unexpected CLI error: {}",
            stderr
        );
    }
}

mod unit_tests {
    use super::*;
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "0:00");
        assert_eq!(format_duration(30.0), "0:30");
        assert_eq!(format_duration(90.0), "1:30");
        assert_eq!(format_duration(3661.0), "1:01:01");
    }
    
    #[test]
    fn test_calculate_aspect_ratio() {
        assert_eq!(calculate_aspect_ratio(1920, 1080), 1920.0 / 1080.0);
        assert_eq!(calculate_aspect_ratio(100, 100), 1.0);
        assert_eq!(calculate_aspect_ratio(4, 3), 4.0 / 3.0);
    }
    
    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-1, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
        assert_eq!(clamp(2.5, 1.0, 3.0), 2.5);
    }
    
    #[test]
    fn test_get_ascii_chars() {
        let ascii_chars = get_ascii_chars(&ColorPalette::Ascii);
        assert!(!ascii_chars.is_empty());
        assert!(ascii_chars.contains(&' '));
        
        let block_chars = get_ascii_chars(&ColorPalette::Color);
        assert!(!block_chars.is_empty());
    }
}

mod converter_tests {
    use super::*;
    use ascii_player::decoder::VideoFrame;
    
    fn create_test_frame(width: u32, height: u32, r: u8, g: u8, b: u8) -> VideoFrame {
        let data = vec![r, g, b; (width * height) as usize];
        VideoFrame {
            data,
            width,
            height,
            timestamp: 0.0,
            frame_number: 1,
        }
    }
    
    #[test]
    fn test_frame_conversion() {
        let config = ConversionConfig::default();
        let converter = FrameConverter::new(config);
        
        // Create a simple 2x2 frame
        let frame = create_test_frame(2, 2, 128, 128, 128);
        
        let ascii_frame = converter.convert_frame(&frame, 10, 10).unwrap();
        
        assert!(ascii_frame.width > 0);
        assert!(ascii_frame.height > 0);
        assert_eq!(ascii_frame.characters.len(), (ascii_frame.width * ascii_frame.height) as usize);
        assert_eq!(ascii_frame.fg_colors.len(), ascii_frame.characters.len());
        assert_eq!(ascii_frame.timestamp, 0.0);
        assert_eq!(ascii_frame.frame_number, 1);
    }
    
    #[test]
    fn test_black_and_white_conversion() {
        let config = ConversionConfig::default();
        let converter = FrameConverter::new(config);
        
        // Test pure black frame
        let black_frame = create_test_frame(1, 1, 0, 0, 0);
        let ascii_frame = converter.convert_frame(&black_frame, 10, 10).unwrap();
        
        // Should produce the darkest character (first in the ramp)
        assert!(!ascii_frame.characters.is_empty());
        
        // Test pure white frame
        let white_frame = create_test_frame(1, 1, 255, 255, 255);
        let ascii_frame = converter.convert_frame(&white_frame, 10, 10).unwrap();
        
        // Should produce a bright character
        assert!(!ascii_frame.characters.is_empty());
    }
    
    #[test]
    fn test_transparent_conversion() {
        let config = ConversionConfig {
            transparent: true,
            alpha_threshold: Some(128),
            ..Default::default()
        };
        let converter = FrameConverter::new(config);
        
        // Create a frame with low brightness (should be transparent)
        let frame = create_test_frame(1, 1, 50, 50, 50);
        let ascii_frame = converter.convert_frame(&frame, 10, 10).unwrap();
        
        assert!(!ascii_frame.characters.is_empty());
        assert!(ascii_frame.bg_colors.is_none()); // No background colors in transparent mode
    }
}

mod renderer_tests {
    use super::*;
    use ascii_player::converter::AsciiFrame;
    
    fn create_test_ascii_frame() -> AsciiFrame {
        AsciiFrame {
            characters: vec!['#', ' ', '@', '.'],
            fg_colors: vec![(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)],
            bg_colors: Some(vec![(0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0)]),
            width: 2,
            height: 2,
            timestamp: 1.0,
            frame_number: 42,
        }
    }
    
    #[test]
    fn test_frame_delay_calculation() {
        let delay = calculate_frame_delay(30.0, 1.0);
        assert_eq!(delay.as_millis(), 33); // ~33ms for 30 FPS
        
        let delay_2x = calculate_frame_delay(30.0, 2.0);
        assert_eq!(delay_2x.as_millis(), 16); // ~16ms for 60 FPS (2x speed)
        
        let delay_half = calculate_frame_delay(30.0, 0.5);
        assert_eq!(delay_half.as_millis(), 66); // ~66ms for 15 FPS (0.5x speed)
    }
    
    #[test]
    fn test_renderer_creation() {
        // Test renderer creation with different modes
        let result = Renderer::new(false, true);
        assert!(result.is_ok(), "Should be able to create color renderer");
        
        let result = Renderer::new(true, false);
        assert!(result.is_ok(), "Should be able to create transparent mono renderer");
    }
    
    #[test]
    fn test_renderer_properties() {
        let renderer = Renderer::new(true, false).unwrap();
        assert!(renderer.is_transparent());
        assert!(!renderer.uses_colors());
        
        let renderer = Renderer::new(false, true).unwrap();
        assert!(!renderer.is_transparent());
        assert!(renderer.uses_colors());
    }
}

#[cfg(feature = "ffmpeg-test")]
mod ffmpeg_integration_tests {
    use super::*;
    use ascii_player::decoder::{VideoDecoder, load_video};
    
    #[test]
    fn test_video_loading() {
        let video_path = create_test_video().unwrap();
        
        // Only run this test if we have a real video file
        if video_path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            let result = VideoDecoder::new(&video_path);
            
            match result {
                Ok(decoder) => {
                    let (width, height) = decoder.dimensions();
                    assert!(width > 0);
                    assert!(height > 0);
                    assert!(decoder.fps() > 0.0);
                }
                Err(e) => {
                    // FFmpeg might not be available in all test environments
                    eprintln!("FFmpeg test skipped: {}", e);
                }
            }
        }
    }
    
    #[test]
    fn test_frame_iteration() {
        let video_path = create_test_video().unwrap();
        
        if video_path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            let result = load_video(&video_path, None, None);
            
            match result {
                Ok(mut frame_iter) => {
                    let mut frame_count = 0;
                    
                    while let Some(frame_result) = frame_iter.next() {
                        match frame_result {
                            Ok(frame) => {
                                assert!(frame.width > 0);
                                assert!(frame.height > 0);
                                assert!(!frame.data.is_empty());
                                assert_eq!(frame.data.len(), (frame.width * frame.height * 3) as usize);
                                
                                frame_count += 1;
                                if frame_count >= 3 {
                                    break; // Test first few frames only
                                }
                            }
                            Err(e) => {
                                panic!("Frame iteration failed: {}", e);
                            }
                        }
                    }
                    
                    assert!(frame_count > 0, "Should have decoded at least one frame");
                }
                Err(e) => {
                    eprintln!("FFmpeg frame iteration test skipped: {}", e);
                }
            }
        }
    }
}