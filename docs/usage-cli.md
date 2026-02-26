# CLI Usage

## Basic Conversion

```bash
# Convert single image
transmute convert input.png --format jpg

# Convert with custom output path
transmute convert photo.jpg --format webp --output compressed.webp

# Multi-image to PDF
transmute convert img1.jpg img2.png img3.webp --format pdf --output album.pdf
```

## Compression

```bash
# Compress with quality preset
transmute compress large_photo.jpg --quality high

# Compress with percentage (1-100)
transmute compress image.png --quality 85

# Compress and change format
transmute compress photo.png --format jpg --quality balanced
```

Available quality presets: `low`, `balanced`, `high`, `maximum`

## Batch Processing

```bash
# Convert all PNGs in current directory to JPEG
transmute batch "*.png" --format jpg

# Convert all images in a folder
transmute batch "./photos/*.jpg" --format webp --output ./compressed/

# Use glob patterns
transmute batch "**/*.png" --format pdf --output combined.pdf
```

## Natural Language Commands

```bash
# Natural language interface
transmute natural convert all my photos to PDF

# More examples
transmute natural compress images in Desktop to 80% quality
transmute natural merge vacation photos into album.pdf
```

## Configuration Management

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

## Global Options

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
