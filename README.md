# ASCII Player

A responsive, color-enabled ASCII video player for the terminal, optimized for Nix-based dotfiles environments.

![ASCII Player Demo](https://img.shields.io/badge/demo-coming_soon-blue)
[![Build Status](https://img.shields.io/github/actions/workflow/status/gapul/ascii-player/ci.yml?branch=main)](https://github.com/gapul/ascii-player/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Nix Flake](https://img.shields.io/badge/nix-flake-blue)](https://nixos.org/)

## Features

- 🎬 **Video to ASCII conversion** with multiple character sets
- 🌈 **24-bit True Color support** for vibrant ASCII art
- 📱 **Responsive terminal handling** - automatically adapts to window resizing
- 🔍 **Transparent background support** for terminal backgrounds to show through
- ⚡ **Hardware-accelerated video decoding** using FFmpeg
- 🎛️ **Playback controls** - pause, speed adjustment, looping
- 📊 **SketchyBar integration** for macOS status bar updates
- 🛠️ **Nix-first development** with reproducible builds
- 🎨 **Multiple color palettes** - ASCII, Grayscale, Full Color

## Installation

### Using Nix Flakes (Recommended)

```bash
# Install directly from GitHub
nix profile install github:gapul/ascii-player

# Or add to your system configuration
# In your flake.nix:
inputs.ascii-player.url = "github:gapul/ascii-player";
```

### From Source

```bash
# Clone the repository
git clone https://github.com/gapul/ascii-player.git
cd ascii-player

# Enter development environment
nix develop

# Build and install
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Play a video file
ascii-player video.mp4

# Play with transparent background
ascii-player --transparent video.mp4

# Loop playback at 2x speed
ascii-player --loop --speed 2.0 video.mp4

# Use grayscale palette
ascii-player --palette grayscale video.mp4
```

### Advanced Options

```bash
# Set custom terminal dimensions
ascii-player --width 80 --height 24 video.mp4

# Play specific time range
ascii-player --start-time 30 --end-time 90 video.mp4

# Enable SketchyBar integration
ascii-player --sketchybar-item media_player video.mp4

# Set maximum frame rate
ascii-player --fps 30 video.mp4

# Enable verbose logging
ascii-player --verbose video.mp4
```

### Interactive Controls

| Key | Action |
|-----|--------|
| `SPACE` | Pause/Resume |
| `Q` / `ESC` | Quit |
| `+` / `=` | Increase speed |
| `-` | Decrease speed |
| `L` | Toggle loop |
| `R` | Restart video |
| `H` / `F1` | Toggle help |

## Development

This project uses Nix for development environment management and builds.

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- [direnv](https://direnv.net/) (optional but recommended)

### Setup

```bash
# Clone the repository
git clone https://github.com/gapul/ascii-player.git
cd ascii-player

# Enter development shell
nix develop

# Or if using direnv
direnv allow
```

### Available Commands

```bash
# Show all available tasks
just --list

# Build the project
just build

# Run with a test video
just run tests/assets/sample.mp4

# Run tests
just test

# Format code
just fmt

# Run linting
just clippy

# Full quality check
just check

# Create test video (requires ffmpeg)
just create-test-video
```

### Integration with Dotfiles

This project is designed to integrate seamlessly with Nix-based dotfiles. To add it to your dotfiles:

1. Add as a flake input in your `flake.nix`:
```nix
inputs.ascii-player.url = "github:gapul/ascii-player";
```

2. Include in your system packages:
```nix
environment.systemPackages = with pkgs; [
  inputs.ascii-player.packages.${system}.default
  # ... other packages
];
```

3. Configure SketchyBar integration in your SketchyBar config:
```bash
# In your sketchybarrc
sketchybar --add item media_player right \
          --set media_player update_freq=1 \
                             script="echo ''"
```

## Architecture

The player is built with a modular architecture:

- **CLI Module** (`src/cli.rs`) - Command line argument parsing and validation
- **Decoder Module** (`src/decoder.rs`) - Video file decoding using FFmpeg
- **Converter Module** (`src/converter.rs`) - Frame to ASCII conversion with color support
- **Renderer Module** (`src/renderer.rs`) - Terminal rendering with crossterm
- **Main Application** (`src/main.rs`) - Orchestrates all modules with async event handling

## Performance

- **Memory Efficient**: Streams video frames without loading entire files
- **CPU Optimized**: Efficient ASCII conversion algorithms
- **Terminal Responsive**: Sub-50ms response to terminal resize events
- **Frame Rate Control**: Adaptive timing to maintain smooth playback

## Supported Formats

Thanks to FFmpeg integration, ASCII Player supports virtually all video formats:

- MP4, AVI, MKV, MOV, WMV
- WebM, OGV, FLV
- And many more...

## WezTerm Integration

ASCII Player is optimized for [WezTerm](https://wezfurlong.org/wezterm/) and supports:

- True color rendering
- Transparent backgrounds
- Font ligatures and Unicode
- High refresh rate displays

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run the full test suite: `just check`
5. Commit your changes: `git commit -am 'Add feature'`
6. Push to the branch: `git push origin feature-name`
7. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [FFmpeg](https://ffmpeg.org/) for video decoding capabilities
- [crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal manipulation
- [clap](https://github.com/clap-rs/clap) for CLI argument parsing
- The Nix community for the reproducible build system

## Related Projects

- [Terminal Media Players](https://github.com/topics/terminal-media-player)
- [ASCII Art Tools](https://github.com/topics/ascii-art)
- [Nix Flakes](https://nixos.wiki/wiki/Flakes)