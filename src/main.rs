mod playlist;
mod player;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::io;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use anyhow::Result;

use playlist::Playlist;
use player::{AudioPlayer, PlaybackState};

#[derive(Parser)]
#[command(name = "rust-cli-music-player")]
struct Args {
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,
    
    #[arg(short, long, default_value = "0.5")]
    volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Player,
    Playlist,
    Help,
}

struct App {
    playlist: Arc<Mutex<Playlist>>,
    player: Arc<AudioPlayer>,
    mode: AppMode,
    list_state: ListState,
    last_tick: Instant,
}

impl App {
    fn new(playlist: Playlist, player: AudioPlayer) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            playlist: Arc::new(Mutex::new(playlist)),
            player: Arc::new(player),
            mode: AppMode::Player,
            list_state,
            last_tick: Instant::now(),
        }
    }
    
    fn on_tick(&mut self) {
        self.last_tick = Instant::now();
        
        // Auto-play next track if current finished
        if matches!(self.player.get_state(), PlaybackState::Playing) && self.player.is_finished() {
            let mut playlist = self.playlist.lock().unwrap();
            if let Some(next_song) = playlist.next() {
                let _ = self.player.play_song(next_song);
                self.list_state.select(Some(playlist.current_index()));
            }
        }
    }
    
    fn next_track(&mut self) {
        let mut playlist = self.playlist.lock().unwrap();
        if let Some(next_song) = playlist.next() {
            let _ = self.player.play_song(next_song);
            self.list_state.select(Some(playlist.current_index()));
        }
    }
    
    fn prev_track(&mut self) {
        let mut playlist = self.playlist.lock().unwrap();
        if let Some(prev_song) = playlist.prev() {
            let _ = self.player.play_song(prev_song);
            self.list_state.select(Some(playlist.current_index()));
        }
    }
    
    fn play_selected(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            let mut playlist = self.playlist.lock().unwrap();
            if let Some(song) = playlist.play_index(selected) {
                let _ = self.player.play_song(song);
            }
        }
    }
    
    fn scroll_up(&mut self) {
        let playlist = self.playlist.lock().unwrap();
        let len = playlist.len();
        if len > 0 {
            let selected = self.list_state.selected().unwrap_or(0);
            let new_selected = if selected == 0 { len - 1 } else { selected - 1 };
            self.list_state.select(Some(new_selected));
        }
    }
    
    fn scroll_down(&mut self) {
        let playlist = self.playlist.lock().unwrap();
        let len = playlist.len();
        if len > 0 {
            let selected = self.list_state.selected().unwrap_or(0);
            let new_selected = if selected >= len - 1 { 0 } else { selected + 1 };
            self.list_state.select(Some(new_selected));
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let initial_volume = args.volume.clamp(0.0, 1.0);
    
    let playlist = Playlist::new_from_dir(&args.dir)?;
    let player = AudioPlayer::new()?;
    player.set_volume(initial_volume);
    
    // Play first song
    if let Some(first_song) = playlist.current() {
        let _ = player.play_song(first_song);
    }
    
    let app = App::new(playlist, player);
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let tick_rate = Duration::from_millis(50);
    let res = run_app(&mut terminal, app, tick_rate);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("{err:?}");
    }
    
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Player => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                return Ok(());
                            }
                            KeyCode::Char(' ') | KeyCode::Char('p') => {
                                app.player.toggle_pause();
                            }
                            KeyCode::Char('n') | KeyCode::Right => {
                                app.next_track();
                            }
                            KeyCode::Char('b') | KeyCode::Left => {
                                app.prev_track();
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                app.player.volume_up();
                            }
                            KeyCode::Char('-') => {
                                app.player.volume_down();
                            }
                            KeyCode::Char('l') | KeyCode::Tab => {
                                app.mode = AppMode::Playlist;
                            }
                            KeyCode::Char('h') | KeyCode::F(1) => {
                                app.mode = AppMode::Help;
                            }
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                let digit = c.to_digit(10).unwrap() as usize;
                                if digit > 0 {
                                    let mut playlist = app.playlist.lock().unwrap();
                                    if let Some(song) = playlist.play_index(digit - 1) {
                                        let _ = app.player.play_song(song);
                                        app.list_state.select(Some(digit - 1));
                                    }
                                }
                            }
                            _ => {}
                        },
                        AppMode::Playlist => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.mode = AppMode::Player;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.scroll_up();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.scroll_down();
                            }
                            KeyCode::Enter => {
                                app.play_selected();
                                app.mode = AppMode::Player;
                            }
                            KeyCode::Tab => {
                                app.mode = AppMode::Player;
                            }
                            _ => {}
                        },
                        AppMode::Help => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => {
                                app.mode = AppMode::Player;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
        
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());
    
    // Header
    let header = Paragraph::new("🎵 CLI Music Player")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);
    
    match app.mode {
        AppMode::Player => render_player_view(f, chunks[1], app),
        AppMode::Playlist => render_playlist_view(f, chunks[1], app),
        AppMode::Help => render_help_view(f, chunks[1]),
    }
    
    // Footer
    let mode_text = match app.mode {
        AppMode::Player => "Player Mode | Tab: Playlist | H: Help | Q: Quit",
        AppMode::Playlist => "Playlist Mode | ↑↓: Navigate | Enter: Play | Tab: Back | Q: Exit",
        AppMode::Help => "Help | Q/H/Esc: Back",
    };
    
    let footer = Paragraph::new(mode_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn render_player_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Now playing
            Constraint::Length(3),  // Controls
            Constraint::Min(5),     // Track list preview
        ])
        .split(area);
    
    // Now Playing
    let playlist = app.playlist.lock().unwrap();
    let current_song = playlist.current_song_name();
    let current_index = playlist.current_index() + 1;
    let total_songs = playlist.len();
    let volume = (app.player.get_volume() * 100.0) as u8;
    
    let (status_text, status_color) = match app.player.get_state() {
        PlaybackState::Playing => ("▶ Playing", Color::Green),
        PlaybackState::Paused => ("⏸ Paused", Color::Yellow),
        PlaybackState::Stopped => ("⏹ Stopped", Color::Red),
    };
    
    let now_playing_text = vec![
        Line::from(vec![
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Track: "),
            Span::styled(&current_song, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(format!("{}/{} tracks", current_index, total_songs)),
        ]),
    ];
    
    let now_playing = Paragraph::new(now_playing_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Now Playing")
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(now_playing, chunks[0]);
    
    // Volume control
    let volume_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Volume: {}%", volume)))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(volume as f64 / 100.0);
    f.render_widget(volume_gauge, chunks[1]);
    
    // Track list preview
    let tracks: Vec<ListItem> = playlist.list()
        .iter()
        .enumerate()
        .take(10)
        .map(|(_i, (idx, song))| {
            let content = format!("{}. {}", 
                idx + 1,
                song.file_stem().unwrap_or_default().to_string_lossy()
            );
            
            let style = if *idx == playlist.current_index() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();
    
    let tracks_list = List::new(tracks)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Tracks (Tab for full playlist)"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("♪ ");
    
    f.render_widget(tracks_list, chunks[2]);
}

fn render_playlist_view(f: &mut Frame, area: Rect, app: &mut App) {
    let playlist = app.playlist.lock().unwrap();
    let current_index = playlist.current_index();
    
    let tracks: Vec<ListItem> = playlist.list()
        .iter()
        .map(|(idx, song)| {
            let content = format!("{}. {}", 
                idx + 1,
                song.file_stem().unwrap_or_default().to_string_lossy()
            );
            
            let style = if *idx == current_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();
    
    let tracks_list = List::new(tracks)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Playlist"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("♪ ");
    
    f.render_stateful_widget(tracks_list, area, &mut app.list_state);
}

fn render_help_view(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Player Controls:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Space/P     - Play/Pause"),
        Line::from("  N/→         - Next track"),
        Line::from("  B/←         - Previous track"),
        Line::from("  +/=         - Volume up"),
        Line::from("  -           - Volume down"),
        Line::from("  1-9         - Play track number"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Tab/L       - Toggle playlist view"),
        Line::from("  H/F1        - Show this help"),
        Line::from("  Q/Esc       - Quit/Back"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Playlist View:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ↑↓/J/K      - Navigate tracks"),
        Line::from("  Enter       - Play selected track"),
        Line::from(""),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .title_style(Style::default().fg(Color::Cyan)));
    
    f.render_widget(help_paragraph, area);
}