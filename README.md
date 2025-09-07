# 🎵 CLI Music Player 

Hey there! So I built this little music player in Rust because, honestly, I got tired of switching between my terminal and Spotify. This thing lets you play your local music files right from the command line - no GUI needed!

Main Interface<img width="1689" height="502" alt="image" src="https://github.com/user-attachments/assets/507ca957-1313-4e41-8e8b-ab8a72285cd6" />

*The player in action - shows current track, volume, and your playlist*

## What it does

This music player is pretty straightforward but does everything you need:
- Plays your MP3 and WAV files (sorry, no FLAC yet - maybe later!)
- Builds playlists automatically from whatever folder you point it at
- Has all the basic controls you'd expect (play, pause, skip, volume)
- Shows you what's playing and what's coming next
- Actually looks decent in your terminal

!Help Screen <img width="1689" height="502" alt="Screenshot 2025-09-07 175836" src="https://github.com/user-attachments/assets/123e2cd7-b209-447f-8511-d0fdebbff9c5" />
*Hit 'H' anytime to see all available controls*

## How to use it

The controls are pretty intuitive if you've used any music player before:

- **Space** or **P** - Play/Pause (the classics)
- **N** or **>** - Next track
- **B** or **<** - Previous track  
- **+/-** - Volume up/down
- **Tab** or **L** - Toggle playlist view
- **1-9** - Jump to track number
- **H** - Show help (when you forget these)
- **Q** - Quit


## Getting it running

Alright, here's how to get this thing working on your machine:

### If you're on Windows (like me when I built this)

You'll need some C++ build stuff because the audio libraries are picky:
1. Grab [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. Install the "C++ build tools" (yeah, it's annoying but necessary)

Or if you hate Microsoft as much as I do sometimes:
```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

### Running the thing

```bash
# Just run it in whatever folder has your music
cargo run

# Or point it at your music folder
cargo run -- --dir ~/Music

# Want it louder from the start?
cargo run -- --dir ~/Music --volume 0.8

# Forgot how it works?
cargo run -- --help
```

## How it's organized

Nothing fancy, just three files:
```
src/
├── main.rs          # The main stuff - handles input, coordinates everything
├── player.rs        # Actually plays the music (rodio does the heavy lifting)
└── playlist.rs      # Finds your music files and manages the playlist
```

## Some examples

Once it's running, try this:
- Hit **Tab** to see your whole playlist 
- Press a number (1-9) to jump to that track
- Use **+** and **-** to adjust volume (goes by 10% each time)
- **N** and **B** to skip around
- **Q** when you're done

## What it uses

I used these crates to make life easier:
- `rodio` for actually playing the audio (works everywhere)
- `crossterm` for handling keyboard input without being weird
- `walkdir` to find all your music files
- `clap` because command-line args are annoying to parse manually
- `anyhow` for when things go wrong (which they will)

## When things break (they will)

### "It won't compile!" on Windows

Yeah, this happens. You'll see some scary `link.exe failed` error. Here's what works:

**Option 1:** Just install the Visual Studio Build Tools (linked above). Annoying but reliable.

**Option 2:** Use the GNU toolchain if you're stubborn like me:
```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
cargo build
```

### "I can't hear anything!"

- Make sure your speakers/headphones work (I know, obvious)
- Try `--volume 1.0` to max out the volume  
- Check if your music files are actually MP3 or WAV (no M4A or weird formats)

### "It says no music found!"

- Double-check the folder path actually exists
- Make sure there are .mp3 or .wav files in there
- Try using the full path instead of relative paths

## How it works under the hood

Nothing too crazy - I used three threads:
1. One for the UI and showing what's playing
2. One for listening to your keyboard mashing
3. One for the actual music playback and auto-advancing songs

They all talk to each other through some shared state wrapped in mutexes (because concurrency is hard).

## Things I might add later

- Support for FLAC, OGG, etc. (when I get around to it)
- Reading ID3 tags so you see actual song names instead of filenames
- Maybe a fancier UI with ratatui
- Shuffle mode (everyone wants shuffle mode)
- Equalizer controls
- Custom playlists that you can save
- Progress bar with scrubbing

No promises on timeline though - this was just a weekend project that got out of hand!


