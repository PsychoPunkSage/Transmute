# Transmute CLI

Privacy-focused media converter with GPU acceleration.

## Installation

```bash
cargo install --path crates/transmute-cli
```

## Usage

### Convert Image Format

```bash
# Basic conversion
transmute convert input.png --format jpeg

# With custom output
transmute convert photo.jpg -f webp -o ~/Desktop/photo.webp

# Verbose mode
transmute -v convert image.tiff -f png
```

### Compress/Optimize

```bash
# Compress with percentage
transmute compress image.jpg --quality 80

# Compress with preset
transmute compress photo.png -q high

# Compress and convert format
transmute compress input.png -f jpeg -q 85
```

### Batch Operations

```bash
# Convert all PNGs to JPEG
transmute batch "*.png" --format jpeg

# Process files in directory
transmute batch "./photos/*.jpg" -f webp -o ./optimized

# With parallel jobs
transmute -j 8 batch "*.tiff" -f png
```

### Natural Language Commands

```bash
# Simple conversion
transmute natural convert ~/photo.png to jpeg

# Compression with quality
transmute natural compress image.jpg to 75%

# Batch with natural paths
transmute natural batch downloads/*.png convert to webp at desktop
```

### Configuration

```bash
# Show config
transmute config show

# Set default quality
transmute config set default_quality high

# Disable GPU
transmute config set use_gpu false

# Reset to defaults
transmute config reset

# Show config file location
transmute config path
```

## Global Flags

- `-v, --verbose`: Enable debug logging
- `--no-color`: Disable colored output
- `--no-progress`: Disable progress bars
- `-j, --jobs <N>`: Number of parallel jobs (0 = auto)
- `--no-gpu`: Disable GPU acceleration

## Configuration File

Location: `~/.config/transmute/config.toml` (Linux/macOS)

```toml
default_output_dir = "/home/user/Downloads/transmute"
default_quality = "high"
use_gpu = true
parallel_jobs = 8
show_progress = true
colored_output = true
```

## Shell Completion

### Bash

```bash
echo 'eval "$(transmute completions bash)"' >> ~/.bashrc
```

### Zsh

```bash
echo 'eval "$(transmute completions zsh)"' >> ~/.zshrc
```

### Fish

```bash
transmute completions fish | source
```

## Examples

```bash
# Convert image
transmute convert photo.jpg -f png

# Optimize PNG
transmute compress image.png -q maximum

# Batch convert with progress
transmute batch "photos/*.jpg" -f webp -o ./optimized

# Natural language
transmute natural convert desktop/image.png to jpeg at downloads

# Disable GPU for testing
transmute --no-gpu convert test.png -f jpg
```
