use ffmpeg_next as ffmpeg;
use std::path::Path;
use anyhow::{Result, anyhow};
use log::{debug, info};

/// Video decoder that extracts frames from video files
pub struct VideoDecoder {
    input_context: ffmpeg::format::context::Input,
    stream_index: usize,
    decoder: ffmpeg::codec::decoder::Video,
    scaler: Option<ffmpeg::software::scaling::Context>,
    frame_count: u64,
    fps: f64,
    duration: f64,
}

/// Represents a decoded video frame with metadata
#[derive(Debug)]
pub struct VideoFrame {
    /// Raw RGB data
    pub data: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height  
    pub height: u32,
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Frame number
    pub frame_number: u64,
}

impl VideoDecoder {
    /// Create a new VideoDecoder from a file path
    pub fn new(path: &Path) -> Result<Self> {
        // Initialize FFmpeg with error handling
        match ffmpeg::init() {
            Ok(_) => debug!("FFmpeg initialized successfully"),
            Err(e) => {
                debug!("FFmpeg init error: {:?}", e);
                // Continue anyway as this might not be fatal
            }
        }
        
        debug!("Attempting to open video file: {}", path.display());
        let input_context = ffmpeg::format::input(&path)
            .map_err(|e| {
                info!("FFmpeg error details: {:?}", e);
                anyhow!("Failed to open video file '{}': {}", path.display(), e)
            })?;
        debug!("Successfully opened video file");
        
        // Find the best video stream
        let stream = input_context
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| anyhow!("No video stream found in file '{}'", path.display()))?;
        
        let stream_index = stream.index();
        
        info!("Found video stream {} in file '{}'", stream_index, path.display());
        
        // Create decoder context
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
            .map_err(|e| anyhow!("Failed to create codec context: {}", e))?;
        
        let decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| anyhow!("Failed to create video decoder: {}", e))?;
        
        // Get video metadata
        let fps = stream.avg_frame_rate();
        let fps = if fps.denominator() != 0 {
            fps.numerator() as f64 / fps.denominator() as f64
        } else {
            25.0 // Default fallback FPS
        };
        
        let duration = if stream.duration() != ffmpeg::ffi::AV_NOPTS_VALUE {
            stream.duration() as f64 * stream.time_base().numerator() as f64 / stream.time_base().denominator() as f64
        } else {
            0.0
        };
        
        debug!("Video info: {}x{}, {:.2} FPS, {:.2}s duration", 
               decoder.width(), decoder.height(), fps, duration);
        
        Ok(Self {
            input_context,
            stream_index,
            decoder,
            scaler: None,
            frame_count: 0,
            fps,
            duration,
        })
    }
    
    /// Get video FPS
    pub fn fps(&self) -> f64 {
        self.fps
    }
    
    /// Get video duration in seconds
    pub fn duration(&self) -> f64 {
        self.duration
    }
    
    /// Get video dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.decoder.width(), self.decoder.height())
    }
    
    /// Seek to a specific time in seconds
    pub fn seek_to(&mut self, timestamp: f64) -> Result<()> {
        let time_base = self.input_context.stream(self.stream_index).unwrap().time_base();
        let timestamp_ts = (timestamp / (time_base.numerator() as f64 / time_base.denominator() as f64)) as i64;
        
        self.input_context.seek(timestamp_ts, ..timestamp_ts)
            .map_err(|e| anyhow!("Failed to seek to timestamp {}: {}", timestamp, e))?;
        
        // Reset decoder state
        self.decoder.flush();
        
        debug!("Seeked to timestamp: {:.2}s", timestamp);
        Ok(())
    }
    
    /// Get the next frame from the video
    pub fn next_frame(&mut self) -> Result<Option<VideoFrame>> {
        let mut decoded_frame = ffmpeg::frame::Video::empty();
        
        // Try to decode frames until we get one from our video stream
        for (stream, packet) in self.input_context.packets() {
            if stream.index() == self.stream_index {
                self.decoder.send_packet(&packet)
                    .map_err(|e| anyhow!("Failed to send packet to decoder: {}", e))?;
                
                // Try to receive decoded frame
                match self.decoder.receive_frame(&mut decoded_frame) {
                    Ok(()) => {
                        self.frame_count += 1;
                        return self.convert_frame(&decoded_frame);
                    }
                    Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::ffi::EAGAIN => {
                        // Need more input
                        continue;
                    }
                    Err(e) => {
                        return Err(anyhow!("Failed to receive frame from decoder: {}", e));
                    }
                }
            }
        }
        
        // Try to flush any remaining frames
        self.decoder.send_eof()
            .map_err(|e| anyhow!("Failed to send EOF to decoder: {}", e))?;
        
        match self.decoder.receive_frame(&mut decoded_frame) {
            Ok(()) => {
                self.frame_count += 1;
                self.convert_frame(&decoded_frame)
            }
            Err(ffmpeg::Error::Eof) => Ok(None),
            Err(e) => Err(anyhow!("Failed to receive final frame: {}", e)),
        }
    }
    
    /// Convert FFmpeg frame to RGB format
    fn convert_frame(&mut self, frame: &ffmpeg::frame::Video) -> Result<Option<VideoFrame>> {
        let width = frame.width();
        let height = frame.height();
        
        // Initialize scaler if needed
        if self.scaler.is_none() {
            self.scaler = Some(
                ffmpeg::software::scaling::Context::get(
                    frame.format(),
                    width,
                    height,
                    ffmpeg::format::Pixel::RGB24,
                    width,
                    height,
                    ffmpeg::software::scaling::Flags::BILINEAR,
                ).map_err(|e| anyhow!("Failed to create scaling context: {}", e))?
            );
        }
        
        let mut rgb_frame = ffmpeg::frame::Video::empty();
        if let Some(ref mut scaler) = self.scaler {
            scaler.run(frame, &mut rgb_frame)
                .map_err(|e| anyhow!("Failed to scale frame: {}", e))?;
        }
        
        // Calculate timestamp
        let time_base = self.input_context.stream(self.stream_index).unwrap().time_base();
        let timestamp = if let Some(ts) = frame.timestamp() {
            if ts != ffmpeg::ffi::AV_NOPTS_VALUE {
                ts as f64 * time_base.numerator() as f64 / time_base.denominator() as f64
            } else {
                self.frame_count as f64 / self.fps
            }
        } else {
            self.frame_count as f64 / self.fps
        };
        
        // Extract RGB data
        let data = rgb_frame.data(0).to_vec();
        
        debug!("Decoded frame {}: {}x{}, timestamp: {:.3}s", 
               self.frame_count, width, height, timestamp);
        
        Ok(Some(VideoFrame {
            data,
            width,
            height,
            timestamp,
            frame_number: self.frame_count,
        }))
    }
    
    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// Iterator wrapper for VideoDecoder
pub struct FrameIterator {
    decoder: VideoDecoder,
    start_time: Option<f64>,
    end_time: Option<f64>,
    has_seeked: bool,
}

impl FrameIterator {
    /// Create a new frame iterator
    pub fn new(decoder: VideoDecoder, start_time: Option<f64>, end_time: Option<f64>) -> Self {
        Self {
            decoder,
            start_time,
            end_time,
            has_seeked: false,
        }
    }
    
    /// Get the underlying decoder reference
    pub fn decoder(&self) -> &VideoDecoder {
        &self.decoder
    }
}

impl Iterator for FrameIterator {
    type Item = Result<VideoFrame>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Seek to start time if specified and not already done
        if let Some(start_time) = self.start_time {
            if !self.has_seeked {
                if let Err(e) = self.decoder.seek_to(start_time) {
                    return Some(Err(e));
                }
                self.has_seeked = true;
            }
        }
        
        match self.decoder.next_frame() {
            Ok(Some(frame)) => {
                // Check if we've reached the end time
                if let Some(end_time) = self.end_time {
                    if frame.timestamp >= end_time {
                        return None;
                    }
                }
                Some(Ok(frame))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Create a frame iterator from a video file
pub fn load_video(path: &Path, start_time: Option<f64>, end_time: Option<f64>) -> Result<FrameIterator> {
    let decoder = VideoDecoder::new(path)?;
    Ok(FrameIterator::new(decoder, start_time, end_time))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_decoder_creation() {
        // This test requires a sample video file
        let test_video = PathBuf::from("tests/assets/sample.mp4");
        if test_video.exists() {
            let result = VideoDecoder::new(&test_video);
            assert!(result.is_ok(), "Failed to create decoder: {:?}", result.err());
        }
    }
    
    #[test]
    fn test_invalid_file() {
        let invalid_path = PathBuf::from("nonexistent.mp4");
        let result = VideoDecoder::new(&invalid_path);
        assert!(result.is_err(), "Should fail for nonexistent file");
    }
}