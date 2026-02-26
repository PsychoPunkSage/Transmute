# GPU Acceleration

Transmute automatically detects available GPU backends.

## Backend Priority

1. **Vulkan** (Linux, Windows)
2. **Metal** (macOS)
3. **DirectX 12** (Windows 10+)
4. **OpenGL** (Fallback for older systems)

## When GPU Acceleration Helps Most

- Images larger than 2MP (1920×1080)
- Batch processing 10+ images
- PDF generation with many pages
- Color space conversions (YCbCr, CMYK)

## When to Disable GPU

- Running on systems without a dedicated GPU
- Experiencing driver compatibility issues
- Processing small images (<500KB)

## Disabling GPU

```bash
# Disable GPU for a single operation
transmute --no-gpu convert input.jpg --format png

# Disable GPU permanently via config
transmute config set use_gpu false
```
