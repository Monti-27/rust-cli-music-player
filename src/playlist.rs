use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Playlist {
    songs: Vec<PathBuf>,
    current_index: usize,
}

impl Playlist {
    pub fn new_from_dir(dir: &Path) -> Result<Self> {
        let mut songs = Vec::new();
        
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if ext == "mp3" || ext == "wav" {
                        songs.push(path.to_path_buf());
                    }
                }
            }
        }
        
        if songs.is_empty() {
            anyhow::bail!("No .mp3 or .wav files found in directory: {}", dir.display());
        }
        
        songs.sort();
        
        Ok(Playlist {
            songs,
            current_index: 0,
        })
    }
    
    pub fn current(&self) -> Option<&PathBuf> {
        self.songs.get(self.current_index)
    }
    
    pub fn next(&mut self) -> Option<&PathBuf> {
        if !self.songs.is_empty() {
            self.current_index = (self.current_index + 1) % self.songs.len();
            self.current()
        } else {
            None
        }
    }
    
    pub fn prev(&mut self) -> Option<&PathBuf> {
        if !self.songs.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.songs.len() - 1;
            } else {
                self.current_index -= 1;
            }
            self.current()
        } else {
            None
        }
    }
    
    pub fn play_index(&mut self, index: usize) -> Option<&PathBuf> {
        if index < self.songs.len() {
            self.current_index = index;
            self.current()
        } else {
            None
        }
    }
    
    pub fn current_index(&self) -> usize {
        self.current_index
    }
    
    pub fn list(&self) -> Vec<(usize, &PathBuf)> {
        self.songs.iter().enumerate().collect()
    }
    
    pub fn len(&self) -> usize {
        self.songs.len()
    }
    
    
    pub fn current_song_name(&self) -> String {
        if let Some(current_song) = self.current() {
            current_song
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            "No song".to_string()
        }
    }
}
