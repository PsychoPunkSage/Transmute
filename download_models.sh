#!/bin/bash
# Download pre-trained RealESRGAN models

set -e

MODELS_DIR="models"
mkdir -p "$MODELS_DIR"

echo "Downloading RealESRGAN models..."

# RealESRGAN 2× model (smaller, faster)
if [ ! -f "$MODELS_DIR/realesrgan_2x.onnx" ]; then
    echo "Downloading 2× model..."
    wget -O "$MODELS_DIR/realesrgan_2x.onnx" \
        "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesrgan-x2.onnx"
fi

# RealESRGAN 4× model (larger, higher quality)
if [ ! -f "$MODELS_DIR/realesrgan_4x.onnx" ]; then
    echo "Downloading 4× model..."
    wget -O "$MODELS_DIR/realesrgan_4x.onnx" \
        "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesrgan-x4.onnx"
fi

echo "Models downloaded successfully!"
echo "2× model: $(du -h $MODELS_DIR/realesrgan_2x.onnx | cut -f1)"
echo "4× model: $(du -h $MODELS_DIR/realesrgan_4x.onnx | cut -f1)"
