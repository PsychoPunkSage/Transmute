# Android Support

## Overview

Android support ships a JNI bridge (`transmute-jni`) — a Rust `cdylib` that Kotlin loads via `System.loadLibrary("transmute_jni")`. The Kotlin app lives in `android/` and calls into the Rust library for all media operations. There is no GPU dependency on Android; all processing uses CPU paths.

## Feature Status

| Feature                          | Android |  Notes                                       |
| -------------------------------- | :-----: | -------------------------------------------- |
| Image conversion (PNG/JPEG/WebP/TIFF/BMP) | ✅ | Full support                        |
| Image compression (mozjpeg/oxipng/webp) | ✅ | CPU-only                            |
| Images → PDF generation          | ✅      | printpdf/lopdf, no native lib needed         |
| Batch convert (parallel)         | ✅      | rayon thread pool                            |
| GPU-accelerated processing       | ❌      | Disabled by design — see [Limitations](#limitations--why) |
| PDF → Images extraction          | ❌      | Needs `libpdfium.so` — see [Enabling PDF Extraction](#enabling-pdf-extraction) |
| Natural language commands        | ❌      | Desktop-only (`transmute-nlp` crate)         |
| GUI (egui)                       | ❌      | Kotlin UI used instead                       |
| CLI                              | ❌      | Not applicable on Android                    |

## Building for Android

### Prerequisites

1. **Android NDK** (r25 or later recommended).
   The build script defaults to `/opt/android-ndk`. Override with `NDK_HOME`:
   ```bash
   export NDK_HOME=$HOME/Android/Sdk/ndk/27.2.12479018
   ```

2. **Rust Android targets** (one-time setup):
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
   ```

3. **`.cargo/config.toml`** at the workspace root maps each target to the NDK clang wrapper.
   The file is already committed; edit the paths only if your NDK is not at `/opt/android-ndk`.

### Build the Rust Library

Run the build script from the workspace root:

```bash
./android/build-rust.sh
```

This compiles `transmute-jni` for three ABIs and copies the `.so` files into
`android/app/src/main/jniLibs/`:

```
jniLibs/
├── arm64-v8a/libtransmute_jni.so
├── armeabi-v7a/libtransmute_jni.so
└── x86_64/libtransmute_jni.so
```

The build uses `--no-default-features` to exclude GPU and PDF-extraction dependencies.

### Build the Android App

Open `android/` in Android Studio and build/run as usual. The `.so` files in
`jniLibs/` are picked up automatically by Gradle.

### Kotlin API

```kotlin
import com.transmute.TransmuteLib

// In Activity.onCreate — required before any other call
TransmuteLib.init(context.getExternalFilesDir(null)!!.absolutePath)

// Convert a single image
val outPath = TransmuteLib.convertImage("/sdcard/photo.jpg", "webp")

// Compress with quality preset ("low" | "balanced" | "high" | "maximum")
val compressed = TransmuteLib.compressImage("/sdcard/photo.jpg", "jpg", "balanced")

// Batch convert
val results: Array<String> = TransmuteLib.batchConvert(arrayOf("/sdcard/a.png", "/sdcard/b.png"), "jpg")

// Combine images into a PDF
val pdf = TransmuteLib.imagesToPdf(arrayOf("/sdcard/p1.jpg", "/sdcard/p2.jpg"), "/sdcard/album.pdf")

// Check whether PDF extraction is available
if (TransmuteLib.pdfExtractSupported()) {
    // show extract UI
}
```

## Limitations & Why

### No GPU Acceleration

`wgpu` (the GPU backend) significantly increases APK size and requires Vulkan
driver setup at runtime. CPU paths through `mozjpeg`, `oxipng`, and `image-rs`
are fast enough for typical mobile image sizes. GPU support may be added in a
future release once the overhead trade-off justifies it.

### No PDF Extraction

PDF-to-image extraction uses `pdfium-render`, which wraps Google's
`libpdfium.so` (~30 MB per ABI). The library is not distributed via crates.io
for Android targets and must be bundled manually. See
[Enabling PDF Extraction](#enabling-pdf-extraction) for the full procedure.

### No Natural Language Commands

`transmute-nlp` depends on desktop-specific crates (`shellexpand`,
`directories`) for resolving home-directory paths. It is not compiled into the
Android build.

## Enabling PDF Extraction

Follow these steps to add PDF-to-image extraction in a custom build:

1. **Download pdfium binaries** for each Android ABI from
   [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases).
   You need the `android-arm64`, `android-arm`, and `android-x64` archives.

2. **Extract and place `libpdfium.so`** into `jniLibs/`:
   ```
   android/app/src/main/jniLibs/
   ├── arm64-v8a/libpdfium.so
   ├── armeabi-v7a/libpdfium.so
   └── x86_64/libpdfium.so
   ```

3. **Enable the feature flag** in `build-rust.sh`. Change:
   ```bash
   cargo build --release \
       --target "$TARGET" \
       --no-default-features \
       -p transmute-jni \
       ...
   ```
   to:
   ```bash
   cargo build --release \
       --target "$TARGET" \
       --no-default-features \
       --features pdf-extract \
       -p transmute-jni \
       ...
   ```

4. **Update `pdfExtractSupported()`** in `TransmuteLib.kt` to return `true` (or
   implement the check in Rust against a compiled-in constant).

5. Rebuild with `./android/build-rust.sh` and rebuild the Android app.

## Output Location

All output files are written to the directory passed to `TransmuteLib.init()`.
The recommended value is `context.getExternalFilesDir(null)?.absolutePath` —
the app's external storage sandbox. No `WRITE_EXTERNAL_STORAGE` permission is
required on Android API 29 (Android 10) and above.
