# Transmute

> **Privacy-focused, GPU-accelerated media converter** built in Rust

Transmute is a high-performance media conversion tool that processes images and PDFs locally on your machine with optional GPU acceleration. No cloud services, no telemetry, no network calls - your files never leave your device.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.91.0-orange.svg)](https://www.rust-lang.org/)

## Features

- **Format Conversion**: Convert between PNG, JPEG, WebP, TIFF, BMP, and PDF
- **Image Compression**: Optimize images with adaptive quality settings (low, medium, high, maximum)
- **Multi-Image to PDF**: Merge multiple images into a single PDF document
- **Batch Processing**: Convert multiple files in parallel with progress tracking
- **GPU Acceleration**: Optional GPU-accelerated processing via wgpu (Vulkan, Metal, DX12)
- **Natural Language Commands**: Execute conversions using natural language (e.g., "convert my photos to PDF")
- **CLI & GUI**: Choose between terminal interface or graphical application
- **Cross-Platform**: Linux, macOS, and Windows support
- **Privacy-First**: Zero network I/O, all processing happens locally
- **Configurable**: Persistent settings with TOML configuration file

## Supported Formats

| Format | Input | Output | Compression | Notes |
|--------|-------|--------|-------------|-------|
| PNG    | Yes   | Yes    | Lossless    | Optimized with oxipng |
| JPEG   | Yes   | Yes    | Lossy       | High-quality encoding with mozjpeg |
| WebP   | Yes   | Yes    | Both        | Modern compression format |
| TIFF   | Yes   | Yes    | Both        | Supports multi-page documents |
| BMP    | Yes   | Yes    | Lossless    | Uncompressed bitmap format |
| GIF    | Yes   | Yes    | Lossless    | Animated GIF support |
| PDF    | Yes   | Yes    | Document    | GPU-accelerated rasterization |

## Installation

### Platform Support

- **Linux**: Full CLI and GUI support (primary development platform)
- **macOS**: Full CLI and GUI support
- **Windows**: Full CLI and GUI support

### Build Requirements

<details>
<summary><b>Linux</b></summary>

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# For GUI support, install GTK3
sudo apt install libgtk-3-dev

# For GPU acceleration (optional)
# Vulkan is recommended on Linux
sudo apt install libvulkan-dev vulkan-tools
```

</details>

<details>
<summary><b>macOS</b></summary>

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# GPU acceleration uses Metal (built into macOS)
```

</details>

<details>
<summary><b>Windows</b></summary>

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ support
2. Install Rust from [rustup.rs](https://rustup.rs/)
3. GPU acceleration uses DirectX 12 (Windows 10+)

</details>

### Install from Source

```bash
# Clone the repository
git clone https://github.com/PsychoPunkSage/transmute.git
cd transmute

# Build and install CLI (transmute binary)
cargo install --path crates/transmute-cli

# Build and install GUI (transmute-gui binary)
cargo install --path crates/transmute-gui

# Or build both in development mode
cargo build --release
```

Binaries will be available at:
- CLI: `target/release/transmute`
- GUI: `target/release/transmute-gui`

## Usage

### CLI Usage

#### Basic Conversion

```bash
# Convert single image
transmute convert input.png --format jpg

# Convert with custom output path
transmute convert photo.jpg --format webp --output compressed.webp

# Multi-image to PDF
transmute convert img1.jpg img2.png img3.webp --format pdf --output album.pdf
```

#### Compression

```bash
# Compress with quality preset
transmute compress large_photo.jpg --quality high

# Compress with percentage (1-100)
transmute compress image.png --quality 85

# Compress and change format
transmute compress photo.png --format jpg --quality medium
```

Available quality presets: `low`, `medium` (balanced), `high`, `maximum` (max)

#### Batch Processing

```bash
# Convert all PNGs in current directory to JPEG
transmute batch "*.png" --format jpg

# Convert all images in a folder
transmute batch "./photos/*.jpg" --format webp --output ./compressed/

# Use glob patterns
transmute batch "**/*.png" --format pdf --output combined.pdf
```

#### Natural Language Commands

```bash
# Natural language interface
transmute natural convert all my photos to PDF

# More examples
transmute natural compress images in Desktop to 80% quality
transmute natural merge vacation photos into album.pdf
```

#### Configuration Management

```bash
# Show current configuration
transmute config show

# Set default quality
transmute config set default_quality high

# Enable GPU acceleration
transmute config set use_gpu true

# Set parallel jobs (0 = auto-detect)
transmute config set parallel_jobs 4

# Reset to defaults
transmute config reset

# Show config file path
transmute config path
```

#### Global Options

```bash
# Verbose logging
transmute --verbose convert input.png --format jpg

# Disable GPU acceleration
transmute --no-gpu convert input.png --format jpg

# Set parallel jobs
transmute --jobs 8 batch "*.png" --format jpg

# Disable colored output
transmute --no-color convert input.png --format jpg

# Disable progress bars
transmute --no-progress batch "*.jpg" --format webp
```

### GUI Usage

Launch the graphical interface:

```bash
transmute-gui
```

#### Features

1. **Drag & Drop**: Drop files or folders directly into the application
2. **Operation Selector**: Choose between Convert, Compress, or Enhance
3. **Format Selection**: Pick target format from dropdown
4. **Quality Control**: Adjust compression quality with visual slider
5. **Batch Queue**: Process multiple files with progress tracking
6. **Settings Panel**: Configure GPU usage, default quality, and output paths

#### Keyboard Shortcuts

- `Ctrl+O`: Open file picker
- `Ctrl+S`: Start processing
- `Ctrl+,`: Open settings
- `Ctrl+Q`: Quit application

## GPU Acceleration

Transmute automatically detects available GPU backends:

**Priority Order**:
1. **Vulkan** (Linux, Windows, Android)
2. **Metal** (macOS, iOS)
3. **DirectX 12** (Windows 10+)
4. **OpenGL** (Fallback for older systems)

**When GPU acceleration helps most**:
- Images larger than 2MP (1920x1080)
- Batch processing 10+ images
- PDF generation with many pages
- Color space conversions (YCbCr, CMYK)

**Disable GPU if**:
- Running on systems without dedicated GPU
- Experiencing driver compatibility issues
- Processing small images (<500KB)

```bash
# Disable GPU for single operation
transmute --no-gpu convert input.jpg --format png

# Disable GPU permanently
transmute config set use_gpu false
```

## Configuration

Configuration file location:
- **Linux**: `~/.config/transmute/config.toml`
- **macOS**: `~/Library/Application Support/transmute/config.toml`
- **Windows**: `%APPDATA%\transmute\config.toml`

<details>
<summary><b>Example Configuration</b></summary>

```toml
# Default quality setting for compression
default_quality = "high"

# Enable GPU acceleration
use_gpu = true

# Number of parallel jobs (0 = auto-detect based on CPU cores)
parallel_jobs = 0

# Show progress bars in CLI
show_progress = true

# Enable colored output
colored_output = true
```

</details>

## License

MIT License - see [LICENSE](LICENSE) for details

**Author**: [PsychoPunkSage](https://github.com/PsychoPunkSage)
