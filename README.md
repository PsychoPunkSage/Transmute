# Transmute: Privacy-Focused Media Converter
## 7-Phase Development Plan

---

## Project Structure (Cargo Workspace)

```
transmute/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── transmute-core/          # Conversion engine + GPU kernels
│   ├── transmute-formats/       # Image/PDF codec implementations
│   ├── transmute-compress/      # Compression/enhancement algorithms
│   ├── transmute-nlp/           # Natural language command parser
│   ├── transmute-cli/           # Terminal interface
│   ├── transmute-gui/           # egui application
│   └── transmute-common/        # Shared types, errors, utilities
├── gpu-shaders/                 # WGSL compute shaders
├── models/                      # Local ML models (upscaling, NLP)
└── tests/                       # Integration tests
```

---

## Phase 1: Core Infrastructure & Image Conversion
**Duration**: 1-2 weeks

### Deliverables
- **Workspace setup** with shared dependencies, feature flags, and build optimizations
- **`transmute-common`**: Error types, format enums, file path utilities, unique naming
- **`transmute-formats`**: Image codec abstraction layer (PNG, JPEG, WebP, TIFF, BMP)
- **`transmute-core`**: Synchronous conversion pipeline with memory-mapped I/O
- **GPU foundation**: WGPU context initialization, basic compute shader infrastructure

### Testing
- Unit tests for format detection and validation
- Round-trip conversion tests (PNG→JPEG→PNG quality preservation)
- Benchmark suite for CPU vs GPU image decoding/encoding
- Memory leak detection with valgrind/heaptrack

### Key Technologies
- `image` crate with optimized feature flags
- `wgpu` for GPU context management
- `rayon` for CPU parallelism
- `criterion` for benchmarking

---

## Phase 2: PDF Operations & Batch Processing
**Duration**: 1-2 weeks

### Deliverables
- **PDF generation**: Multi-image to PDF with configurable DPI, page sizes
- **PDF parsing**: Extract pages as images with GPU-accelerated rasterization
- **Batch processor**: Parallel conversion queue with progress tracking
- **Output management**: Automatic directory creation, collision handling, metadata preservation

### Testing
- PDF spec compliance tests (PDF/A validation)
- Large batch processing (1000+ files) stress tests
- Memory usage profiling for PDF operations
- Cross-platform path handling tests

### Key Technologies
- `printpdf` for generation
- `pdfium-render` for parsing (GPU rasterization)
- `tokio` for async batch orchestration
- Smart chunking to prevent OOM on large batches

---

## Phase 3: GPU-Accelerated Compression
**Duration**: 2 weeks

### Deliverables
- **Image compression**: GPU-optimized JPEG encoding, PNG optimization, WebP compression
- **Quality control**: Adaptive quality settings with SSIM/PSNR validation
- **PDF compression**: Lossless stream compression, image downsampling
- **Compute shaders**: WGSL kernels for color space conversion, chroma subsampling

### Testing
- Compression ratio validation against reference tools (mozjpeg, oxipng)
- Quality metric tests (ensure SSIM > 0.95 for "high" quality)
- GPU shader correctness vs CPU reference implementation
- Performance benchmarks (target: 3x faster than CPU for >2MP images)

### Key Technologies
- Custom WGSL shaders for YCbCr conversion, DCT approximations
- `oxipng` integration with custom prefiltering
- SIMD-optimized CPU fallback paths
- Adaptive algorithm selection based on image characteristics

---

## Phase 4: GPU-Accelerated Enhancement
**Duration**: 2-3 weeks

### Deliverables
- **Upscaling**: 2x/4x image enhancement using ONNX Runtime with GPU inference
- **Model integration**: Quantized RealESRGAN/ESPCN models (<50MB total)
- **Smart enhancement**: Auto-detect optimal scale factor, denoise preprocessing
- **Tile-based processing**: Handle arbitrarily large images via GPU tiling

### Testing
- Visual quality tests with reference datasets (DIV2K, Set5)
- PSNR/SSIM metrics vs ground truth high-res images
- GPU memory limits testing (up to 8K resolution)
- Fallback behavior when GPU unavailable

### Key Technologies
- `ort` (ONNX Runtime) with DirectML/CUDA/Metal backends
- Pre-trained quantized models (INT8 precision)
- Custom tiling algorithm to stay within VRAM limits
- Progressive enhancement for real-time preview

---

## Phase 5: Natural Language Parser
**Duration**: 1-2 weeks

### Deliverables
- **Intent extraction**: Regex + keyword-based parser for conversion commands
- **Path resolution**: Natural language paths ("my desktop", "today's photos")
- **Smart defaults**: Infer missing parameters (format, quality, output location)
- **Validation**: Sanity checks before execution (file exists, format supported)

### Testing
- 100+ command variation tests (different phrasings for same intent)
- Ambiguity handling tests (multiple valid interpretations)
- Error message clarity evaluation
- Fuzzing for malformed inputs

### Key Technologies
- `pest` parser for grammar-based matching
- Local intent classification (no external APIs)
- `shellexpand` for ~ and environment variable expansion
- Fallback to exact path parsing when NL fails

---

## Phase 6: CLI Interface
**Duration**: 1 week

### Deliverables
- **Subcommands**: `convert`, `compress`, `enhance`, `natural`, `batch`
- **Rich output**: Progress bars, ETA, parallel job status
- **Configuration**: Config file support (~/.config/transmute/config.toml)
- **Shell completion**: Bash/Zsh/Fish completion scripts

### Testing
- CLI integration tests with `assert_cmd`
- Golden file tests for output formatting
- Signal handling tests (Ctrl+C cleanup)
- Cross-platform compatibility (Linux, macOS, Windows)

### Key Technologies
- `clap` v4 with derive macros
- `indicatif` for progress visualization
- `directories` for XDG-compliant config paths
- `crossterm` for terminal control

---

## Phase 7: GUI & Final Polish
**Duration**: 2-3 weeks

### Deliverables
- **egui application**: Drag-drop interface, format selector, batch queue
- **Natural language input**: Integrated command box with autocomplete
- **Live preview**: Real-time output preview for compression/enhancement
- **Settings panel**: GPU toggle, default quality, output templates
- **Packaging**: AppImage, Flatpak, Homebrew formula, Windows installer

### Testing
- UI/UX testing with real users
- Accessibility testing (keyboard navigation, screen readers)
- Performance testing (60fps UI with background processing)
- Installation/uninstallation testing on clean systems

### Key Technologies
- `eframe` (egui framework)
- `rfd` for native file dialogs
- `tokio` channels for UI↔Core communication
- Platform-specific packaging (cargo-bundle, AppImageKit)

---

## Cross-Phase Concerns

### Performance Targets
- **Image conversion**: <100ms for 4K images (GPU), <500ms (CPU)
- **PDF generation**: <2s for 100-page document
- **Compression**: Match or exceed reference tools in speed
- **Enhancement**: Real-time preview for 2x upscaling at 1080p

### Idiomatic Practices
- `#![deny(unsafe_code)]` except in FFI boundaries
- Comprehensive error handling with `anyhow`/`thiserror`
- Zero-copy operations with `Cow`, `Arc`, memory mapping
- Async-first design with structured concurrency
- Property-based testing with `proptest`

### GPU Acceleration Strategy
- **Tier 1**: Vulkan/Metal/DX12 via `wgpu`
- **Tier 2**: CUDA via `cudarc` for NVIDIA-specific optimizations
- **Tier 3**: CPU SIMD fallback with `packed_simd`
- Runtime detection and graceful degradation

### Privacy/Security
- No network I/O (enforced via cargo feature flags)
- Secure temp file handling with `tempfile`
- Input sanitization for all file operations
- Optional output encryption with `age`

---

## Total Timeline: 10-14 weeks
## Final Binary Size Target: <15MB (with model bundling)
## Supported Platforms: Linux (primary), macOS, Windows
