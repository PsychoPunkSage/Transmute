# Transmute: Privacy-Focused GPU-Accelerated Media Converter - Personal Project

## Project Overview

Developed a comprehensive privacy-focused, GPU-accelerated media conversion tool built entirely in Rust. This personal project demonstrates advanced software engineering capabilities across GPU compute programming, parallel processing architectures, systems programming, and modern Rust development practices. The project comprises ~7,400 lines of hand-crafted Rust code organized in a modular Cargo workspace with 8 crates, showcasing production-grade software architecture suitable for professional portfolio demonstration.

## Core Technical Achievements

### 1. GPU Compute Pipeline Development

**Challenge**: Traditional image processing libraries perform color space conversions on CPU, creating bottlenecks for high-resolution images (4K+). Implementing efficient GPU compute requires deep understanding of shader programming, buffer management, and CPU-GPU synchronization patterns.

**Implementation**: Built a complete GPU compute pipeline using wgpu (cross-platform abstraction over Vulkan/Metal/DX12) with custom WGSL compute shaders for RGB→YCbCr color space conversion. The pipeline implements ITU-R BT.601 standard compliance with 16×16 workgroup sizing (256 threads per dispatch group).

**Technical Details**: Implemented buffer pooling strategy with separate input, output, staging, and parameter buffers that are reused across operations of matching resolution. Developed SIMD-optimized data packing processing 4 pixels (12 bytes) per iteration with `unsafe` blocks protected by explicit bounds verification. Async buffer mapping uses `futures::channel::oneshot` with 5-second timeout for robustness, with automatic CPU fallback on GPU failure.

**Key Innovation**: Created zero-copy color space conversion achieving ~3× speedup over CPU for 4K images (3840×2160), with intelligent threshold detection (GPU activates only for >2MP images where transfer overhead is amortized).

### 2. Natural Language Processing Parser

**Challenge**: Users expect intuitive command interfaces beyond traditional CLI flags. Building a robust natural language parser requires handling ambiguous inputs, complex path specifications with globs and environment variables, and graceful degradation when parsing fails.

**Implementation**: Developed a two-tier parsing architecture using PEG (Parsing Expression Grammar) via the pest crate as primary parser, with regex fallback for common patterns. The 82-line context-free grammar handles complex patterns including convert commands, compression specifications, batch operations, and PDF merging.

**Technical Details**: Grammar supports case-insensitive verb matching (`^"convert"`, `^"compress"`), quality specifications (percentage and named presets), output path resolution, and multi-file glob patterns. Path resolver integrates `shellexpand` for `~`, `$HOME`, and environment variable expansion, with `glob` crate for wildcard pattern matching. Nine regex patterns provide fallback parsing for common command structures.

**Key Innovation**: Produced a parser that accepts natural language like "compress photo.jpg to 85% at output.jpg" and "batch *.png convert to webp", transforming English commands into structured operation specifications with proper error messages on parse failure.

### 3. PDF Operations Pipeline

**Challenge**: Converting multiple images to PDF while preserving quality requires careful memory management, intelligent compression decisions, and handling of diverse input formats without quality degradation through unnecessary re-encoding cycles.

**Implementation**: Built a complete PDF generation pipeline using printpdf for PDF creation with smart image handling including JPEG passthrough (zero generation loss for JPEG sources), automatic downscaling for images exceeding 2400px dimension, and Lanczos3 resampling for high-quality resizing.

**Technical Details**: Implemented parallel image loading via rayon's `par_iter()` for concurrent decoding across CPU cores. PDF extraction uses pdfium-render for GPU-accelerated rasterization with configurable DPI (default 300). Memory-mapped I/O activates for files exceeding 10MB threshold, using kernel demand paging for efficient large batch processing.

**Key Innovation**: Created an intelligent image embedding system that detects source format and applies JPEG passthrough when appropriate, preventing quality degradation while maximizing compression efficiency for mixed-format batch operations.

### 4. Compression Engine with Quality Metrics

**Challenge**: Balancing file size reduction against visual quality requires format-specific optimization strategies and objective quality measurement. Users need predictable quality outcomes across different compression presets.

**Implementation**: Developed a comprehensive compression engine with format-specific strategies: mozjpeg for high-quality JPEG encoding (superior to standard libjpeg), oxipng for aggressive PNG optimization with multi-level compression, and native WebP encoding with lossy/lossless mode selection.

**Technical Details**: GPU pipeline provides RGB→YCbCr conversion feeding into JPEG encoding. Quality presets map to specific parameters: Maximum (JPEG q98, PNG level 0), High (JPEG q95, PNG level 2), Balanced (JPEG q85, PNG level 4), Low (JPEG q75, PNG level 6). Implemented SSIM, PSNR, and MSE quality metrics for objective compression evaluation against target thresholds.

**Key Innovation**: Built quality-aware compression with measurable outcomes—Maximum preset targets SSIM >0.98, High targets >0.95, Balanced targets >0.90—enabling predictable quality guarantees across batch operations.

### 5. egui Immediate-Mode GUI

**Challenge**: Building responsive GUI for media processing requires efficient texture management, background task execution without UI blocking, and proper state synchronization between UI thread and worker threads.

**Implementation**: Developed a complete immediate-mode GUI using eframe/egui with Arc<Mutex> shared state between UI and background workers. Implemented LRU texture cache (50 full-size, 100 thumbnails) with lazy GPU upload, background image loader thread with channel communication, and native file dialogs via rfd.

**Technical Details**: UI renders at 60fps using declarative immediate-mode paradigm. TextureCache implements separate pools for full and thumbnail resolutions with automatic eviction. ImageLoader worker thread receives requests via channel, loads images off UI thread, and sends responses for non-blocking poll in render loop. Processing spawns dedicated threads with per-file status updates via shared state.

**Key Innovation**: Achieved fully responsive UI during heavy batch processing through structured threading model—UI thread never blocks on I/O or processing, with real-time progress updates and lazy texture loading.

### 6. CLI with Configuration Management

**Challenge**: Production CLI tools require persistent configuration, progress reporting for long operations, and professional output formatting while maintaining scriptability for automation workflows.

**Implementation**: Built a complete CLI using clap v4 derive API with subcommands for convert, compress, enhance, batch, natural language, and configuration management. Progress reporting uses indicatif with spinners for single operations and progress bars with ETA for batch jobs.

**Technical Details**: Configuration persists via TOML at platform-specific paths (`~/.config/transmute/config.toml` on Linux, `~/Library/Application Support/` on macOS). Supports GPU toggle, quality defaults, parallel job count, and output preferences. Global flags include `--verbose`, `--no-gpu`, `--jobs N`, `--no-color`, and `--no-progress` for CI/CD integration.

**Key Innovation**: Created a seamless CLI experience with persistent preferences, colorized output with compression ratio reporting, and natural language mode accepting English commands directly from terminal.

## Advanced Implementation Details

### Memory-Mapped File I/O Architecture

Implemented threshold-based memory mapping (>10MB files) using memmap2 for efficient large file handling. Kernel creates virtual address mapping without loading bytes, with demand paging loading only accessed pages. Benefits include O(1) initialization time, automatic OS-level caching, and reduced memory pressure during batch operations processing hundreds of images.

### Zero-Copy Optimization Patterns

Applied `Cow<'_, T>` (Clone on Write) throughout the stack for conditional allocation avoidance. Image downscaling returns `Cow::Borrowed` when no resize needed, `Cow::Owned` only when transformation required. Buffer pooling in GPU pipeline reuses allocations across same-resolution operations, avoiding allocation overhead on repeated conversions.

### Structured Concurrency Model

Implemented structured concurrency using tokio streams with `buffer_unordered` for controlled parallelism. Parent tasks await all children before completion, preventing orphaned tasks. Progress channels enable real-time updates without blocking worker threads. Proper cleanup on cancellation ensures no resource leaks.

### RAII Resource Management

All resources follow Rust's RAII pattern with automatic cleanup via Drop implementations. GPU buffers release through wgpu's Drop, temporary files via tempfile crate auto-delete on scope exit, and file handles close automatically. No manual cleanup code required, preventing resource leaks by construction.

### Upstream-Ready Code Quality

Maintained strict adherence to Rust best practices throughout all development including proper error handling with `thiserror` (library) and `anyhow` (application), comprehensive documentation comments, and modular architecture enabling independent testing. All code follows idiomatic Rust patterns with zero `unsafe` code at crate level (except FFI boundary in gpu_convert.rs with explicit SAFETY comments).

## Technology Stack

**Core Systems Programming**: Rust 1.91.0 with strict toolchain pinning, wgpu 27.0.1 for cross-platform GPU compute (Vulkan/Metal/DX12/OpenGL), rayon 1.10 for CPU parallelism with work-stealing scheduler, tokio 1.42 for async batch processing with structured concurrency.

**Image/PDF Processing**: image 0.25 for core decoding (PNG, JPEG, WebP, TIFF, BMP, GIF), mozjpeg 0.10 for high-quality JPEG encoding, oxipng 9.1 for aggressive PNG optimization, printpdf/pdfium-render for PDF generation and GPU-accelerated extraction, memmap2 0.9 for memory-mapped file I/O.

**Natural Language Processing**: pest 2.7 for PEG-based grammar parsing, regex 1.11 for fallback pattern matching, shellexpand 3.1 for path resolution with environment variable expansion.

**User Interfaces**: CLI via clap 4.5 (derive API), indicatif 0.17 (progress bars), crossterm 0.28 (terminal control); GUI via eframe/egui 0.30 (immediate-mode), rfd 0.17 (native file dialogs).

**Release Optimizations**: `opt-level = 3` (maximum LLVM optimization), `lto = "fat"` (cross-crate link-time optimization), `codegen-units = 1` (single codegen for better optimization), `strip = true` (debug symbol removal), `panic = "abort"` (smaller binary, faster panics).

## Project Outcomes

This personal project demonstrates comprehensive Rust systems programming capabilities across multiple domains including GPU compute programming with custom WGSL shaders achieving 3× speedup for high-resolution image processing, systems architecture with modular 8-crate workspace and zero network dependencies for privacy guarantee, performance engineering with memory-mapped I/O and zero-copy optimizations, user experience design with two-tier NLP parser and responsive GUI, and professional code quality with 47 tests and comprehensive documentation.

The work addresses real-world media processing challenges with cutting-edge GPU acceleration, showcasing ability to architect complex software systems from scratch, implement low-level compute pipelines, build intuitive user interfaces, and maintain production-grade code quality. These capabilities directly align with systems programming roles requiring deep Rust expertise, performance optimization skills, and end-to-end application development experience.

---

**Project Statistics:**
- **7,376 lines of Rust code**
- **197 source files**
- **47 tests (43 unit + 4 async)**
- **8 workspace crates**
- **40 external dependencies**
- **1 GPU shader (WGSL)**
- **1 PEG grammar (82 lines)**

**Repository:** https://github.com/PsychoPunkSage/transmute
