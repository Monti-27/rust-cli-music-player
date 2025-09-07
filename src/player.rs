use std::path::Path;
use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStream, Sink};
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Arc<Mutex<Sink>>,
    state: Arc<Mutex<PlaybackState>>,
    volume: Arc<Mutex<f32>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        Ok(AudioPlayer {
            _stream,
            sink: Arc::new(Mutex::new(sink)),
            state: Arc::new(Mutex::new(PlaybackState::Stopped)),
            volume: Arc::new(Mutex::new(0.5)),
        })
    }
    
    pub fn play_song(&self, path: &Path) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let source = Decoder::new(file)?;
        
        {
            let sink = self.sink.lock().unwrap();
            sink.stop();
            sink.append(source);
            
            let volume = *self.volume.lock().unwrap();
            sink.set_volume(volume);
        }
        
        *self.state.lock().unwrap() = PlaybackState::Playing;
        
        Ok(())
    }
    
    pub fn pause(&self) {
        let sink = self.sink.lock().unwrap();
        sink.pause();
        *self.state.lock().unwrap() = PlaybackState::Paused;
    }
    
    pub fn resume(&self) {
        let sink = self.sink.lock().unwrap();
        sink.play();
        *self.state.lock().unwrap() = PlaybackState::Playing;
    }
    
    pub fn toggle_pause(&self) {
        let current_state = self.state.lock().unwrap().clone();
        match current_state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.resume(),
            PlaybackState::Stopped => {}
        }
    }
    
    pub fn set_volume(&self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        *self.volume.lock().unwrap() = level;
        
        let sink = self.sink.lock().unwrap();
        sink.set_volume(level);
    }
    
    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }
    
    pub fn volume_up(&self) {
        let current_volume = self.get_volume();
        let new_volume = (current_volume + 0.1).clamp(0.0, 1.0);
        self.set_volume(new_volume);
    }
    
    pub fn volume_down(&self) {
        let current_volume = self.get_volume();
        let new_volume = (current_volume - 0.1).clamp(0.0, 1.0);
        self.set_volume(new_volume);
    }
    
    pub fn get_state(&self) -> PlaybackState {
        self.state.lock().unwrap().clone()
    }
    
    pub fn is_finished(&self) -> bool {
        let sink = self.sink.lock().unwrap();
        sink.empty()
    }
    
}
