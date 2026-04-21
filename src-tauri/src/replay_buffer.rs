//! Replay buffer: manages rolling FFmpeg segments and clip extraction.
//!
//! FFmpeg encodes to sequential 2-second MP4 segments in a temp directory.
//! This module tracks segments, deletes expired ones, and concatenates
//! the survivors into a clip when the user presses the save hotkey.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Manages the rolling segment buffer on disk.
pub struct ReplayBuffer {
    /// Directory where segments are stored
    buffer_dir: PathBuf,
    /// Maximum number of segments to keep (buffer_duration / segment_duration)
    max_segments: usize,
    /// Duration of each segment in seconds
    segment_duration: u32,
    /// Total buffer duration in seconds
    buffer_duration: u32,
    /// Output directory for saved clips
    output_dir: PathBuf,
}

impl ReplayBuffer {
    /// Create a new replay buffer manager.
    pub fn new(buffer_duration: u32, segment_duration: u32, output_dir: PathBuf) -> Self {
        let max_segments = (buffer_duration / segment_duration) as usize + 2; // +2 for safety margin

        // Create buffer directory in system temp
        let buffer_dir = std::env::temp_dir().join("clipsync_buffer");
        fs::create_dir_all(&buffer_dir).expect("Failed to create buffer directory");

        // Create output directory
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        println!(
            "[ClipSync Buffer] Initialized: {}s buffer, {}s segments, max {} segments",
            buffer_duration, segment_duration, max_segments
        );
        println!("[ClipSync Buffer] Buffer dir: {}", buffer_dir.display());
        println!("[ClipSync Buffer] Output dir: {}", output_dir.display());

        Self {
            buffer_dir,
            max_segments,
            segment_duration,
            buffer_duration,
            output_dir,
        }
    }

    /// Get the buffer directory path (for FFmpeg segment output).
    pub fn buffer_dir(&self) -> &Path {
        &self.buffer_dir
    }

    /// Get the segment filename pattern for FFmpeg.
    pub fn segment_pattern(&self) -> String {
        self.buffer_dir
            .join("seg_%05d.mp4")
            .to_string_lossy()
            .to_string()
    }

    /// Get the segment duration.
    pub fn segment_duration(&self) -> u32 {
        self.segment_duration
    }

    /// Clean old segments, keeping only the most recent `max_segments`.
    pub fn cleanup_old_segments(&self) {
        let segments = self.list_segments();
        if segments.len() > self.max_segments {
            let to_remove = segments.len() - self.max_segments;
            for seg in segments.iter().take(to_remove) {
                if let Err(e) = fs::remove_file(seg) {
                    eprintln!(
                        "[ClipSync Buffer] Failed to remove old segment {:?}: {}",
                        seg, e
                    );
                }
            }
        }
    }

    /// List all segment files in the buffer directory, sorted by name (chronological).
    fn list_segments(&self) -> Vec<PathBuf> {
        let mut segments: Vec<PathBuf> = fs::read_dir(&self.buffer_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|ext| ext == "mp4")
                    .unwrap_or(false)
                    && p.file_name()
                        .map(|n| n.to_string_lossy().starts_with("seg_"))
                        .unwrap_or(false)
            })
            .collect();

        // Sort by filename (seg_00000.mp4, seg_00001.mp4, ...) = chronological order
        segments.sort();
        segments
    }

    /// Save a clip from the current buffer.
    ///
    /// Concatenates the most recent segments covering `buffer_duration` seconds
    /// into a single MP4 file using `ffmpeg -f concat -c copy` (no re-encoding).
    ///
    /// Returns the path to the saved clip, or an error.
    pub fn save_clip(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let segments = self.list_segments();

        if segments.is_empty() || segments.len() < 2 {
            return Err("Not enough segments available to save".into());
        }

        // Skip the LAST segment — FFmpeg is still writing to it (incomplete moov atom).
        // This is critical: the most recent segment is always in-progress.
        let complete_segments = &segments[..segments.len() - 1];

        // Take the most recent complete segments covering buffer_duration
        let segments_needed =
            (self.buffer_duration / self.segment_duration) as usize;
        let segments_to_use: Vec<&PathBuf> = if complete_segments.len() > segments_needed {
            complete_segments[complete_segments.len() - segments_needed..].iter().collect()
        } else {
            complete_segments.iter().collect()
        };

        let actual_duration = segments_to_use.len() as u32 * self.segment_duration;
        println!(
            "[ClipSync Buffer] Saving clip from {} segments (~{}s)",
            segments_to_use.len(),
            actual_duration
        );

        // Create concat file for FFmpeg
        let concat_file = self.buffer_dir.join("concat_list.txt");
        let concat_content: String = segments_to_use
            .iter()
            .map(|seg| format!("file '{}'", seg.to_string_lossy().replace('\\', "/")))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&concat_file, &concat_content)?;

        // Generate output filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let output_path = self.output_dir.join(format!("clip_{}.mp4", timestamp));

        // Run FFmpeg concat (this is fast — just remuxing, no re-encoding)
        println!(
            "[ClipSync Buffer] Concatenating to: {}",
            output_path.display()
        );
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel", "warning",
                "-f", "concat",
                "-safe", "0",
                "-i", &concat_file.to_string_lossy(),
                "-c", "copy",
                "-y",
                &output_path.to_string_lossy(),
            ])
            .status()?;

        // Clean up concat file
        let _ = fs::remove_file(&concat_file);

        if status.success() {
            println!(
                "[ClipSync Buffer] Clip saved: {} ({} segments, ~{}s)",
                output_path.display(),
                segments_to_use.len(),
                actual_duration
            );
            Ok(output_path)
        } else {
            Err(format!("FFmpeg concat failed with exit code: {:?}", status.code()).into())
        }
    }

    /// Clear all segments from the buffer directory.
    pub fn clear(&self) {
        for seg in self.list_segments() {
            let _ = fs::remove_file(seg);
        }
        println!("[ClipSync Buffer] Buffer cleared");
    }
}

impl Drop for ReplayBuffer {
    fn drop(&mut self) {
        // Clean up buffer directory on shutdown
        self.clear();
    }
}
