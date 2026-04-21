use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipMetadata {
    pub id: String,
    pub title: String,
    pub game_name: Option<String>,
    pub date_recorded: u64,
    pub duration_secs: u32,
    pub cloud_status: String, // "local", "syncing", "synced"
    pub is_favorite: bool,
    pub file_path: String,
    pub thumbnail_path: String,
}

pub struct ClipStore {
    app_data_dir: PathBuf,
    metadata_file: PathBuf,
}

impl ClipStore {
    pub fn new(app_data_dir: PathBuf) -> Self {
        fs::create_dir_all(&app_data_dir).ok();
        let metadata_file = app_data_dir.join("clips_metadata.json");
        Self {
            app_data_dir,
            metadata_file,
        }
    }

    pub fn get_clips(&self) -> Vec<ClipMetadata> {
        if !self.metadata_file.exists() {
            return Vec::new();
        }

        let mut clips: Vec<ClipMetadata> = match fs::read_to_string(&self.metadata_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Vec::new()),
            Err(_) => Vec::new(),
        };

        // Self-healing: remove clips where the video file no longer exists
        let original_len = clips.len();
        clips.retain(|c| Path::new(&c.file_path).exists());

        if clips.len() < original_len {
            // Re-save if we found missing files
            let _ = self.save_clips(&clips);
        }

        clips
    }

    pub fn save_clips(&self, clips: &[ClipMetadata]) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(clips)?;
        fs::write(&self.metadata_file, json)?;
        Ok(())
    }

    pub fn add_clip(&self, clip: ClipMetadata) -> Result<(), Box<dyn std::error::Error>> {
        let mut clips = self.get_clips();
        clips.push(clip);
        self.save_clips(&clips)
    }

    pub fn update_clip(&self, updated_clip: ClipMetadata) -> Result<(), Box<dyn std::error::Error>> {
        let mut clips = self.get_clips();
        if let Some(pos) = clips.iter().position(|c| c.id == updated_clip.id) {
            clips[pos] = updated_clip;
            self.save_clips(&clips)?;
        }
        Ok(())
    }

    pub fn delete_clip(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut clips = self.get_clips();
        if let Some(pos) = clips.iter().position(|c| c.id == id) {
            let clip = clips.remove(pos);
            
            // Delete actual files
            let _ = fs::remove_file(&clip.file_path);
            let _ = fs::remove_file(&clip.thumbnail_path);
            
            self.save_clips(&clips)?;
        }
        Ok(())
    }

    /// Generates a thumbnail for a given video path and returns the thumbnail path
    pub fn generate_thumbnail(&self, video_path: &Path, id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let thumbnail_path = self.app_data_dir.join(format!("thumb_{}.jpg", id));
        
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel", "error",
                "-i", &video_path.to_string_lossy(),
                "-vframes", "1",
                "-q:v", "2",
                "-y",
                &thumbnail_path.to_string_lossy(),
            ])
            .status()?;

        if status.success() {
            Ok(thumbnail_path)
        } else {
            Err(format!("FFmpeg failed to generate thumbnail with code {:?}", status.code()).into())
        }
    }
}
