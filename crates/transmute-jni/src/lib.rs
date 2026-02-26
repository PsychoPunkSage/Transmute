//! JNI bridge — exposes Transmute's Rust core to Kotlin/Android.
//!
//! # Naming convention
//! JNI function names follow the pattern:
//!   `Java_<package>_<Class>_<method>`
//! where dots in the package name become underscores.
//! Kotlin package: `com.transmute`, class: `TransmuteLib`.
//!
//! # Safety
//! JNI functions are `unsafe` because they cross the JVM boundary.
//! Panics must never cross this boundary — all Results are converted
//! to Java RuntimeExceptions via `env.throw_new()`.

use jni::objects::{JClass, JObject, JObjectArray, JString};
use jni::sys::{jstring, jobjectArray};
use jni::JNIEnv;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use transmute_common::{MediaFormat, PathManager};
use transmute_compress::{ImageCompressor, QualitySettings};
use transmute_formats::{ImageDecoder, ImageEncoder, PdfGenerator, PdfOptions};

// ---------------------------------------------------------------------------
// Global PathManager — set once by `init()`, read by all other functions.
// ---------------------------------------------------------------------------

static PATH_MANAGER: OnceLock<Arc<PathManager>> = OnceLock::new();

fn get_path_manager() -> Option<Arc<PathManager>> {
    PATH_MANAGER.get().cloned()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a JString to a Rust String, throwing a RuntimeException on failure.
fn jstring_to_string(env: &mut JNIEnv, js: JString) -> Result<String, ()> {
    env.get_string(&js)
        .map(|s| s.into())
        .map_err(|e| {
            let _ = env.throw_new("java/lang/RuntimeException", format!("JNI string conversion failed: {e}"));
        })
}

/// Convert a Rust String to a jstring, throwing a RuntimeException on failure.
/// Returns a null jstring on error (caller should return immediately).
fn string_to_jstring(env: &mut JNIEnv, s: &str) -> jstring {
    match env.new_string(s) {
        Ok(js) => js.into_raw(),
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("JNI string creation failed: {e}"));
            JObject::null().into_raw() as jstring
        }
    }
}

/// Throw a RuntimeException and return the given fallback value.
macro_rules! throw {
    ($env:expr, $msg:expr, $ret:expr) => {{
        let _ = $env.throw_new("java/lang/RuntimeException", $msg);
        return $ret;
    }};
}

// ---------------------------------------------------------------------------
// 1. init(outputDir: String)
// ---------------------------------------------------------------------------

/// Initialise the Transmute library.
/// Must be called once before any other function, typically from `onCreate`.
///
/// `output_dir` — path to a writable directory (e.g. `getExternalFilesDir(null).absolutePath`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_transmute_TransmuteLib_init(
    mut env: JNIEnv,
    _class: JClass,
    output_dir: JString,
) {
    // Initialise android_logger so Rust log! calls appear in logcat
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("transmute"),
    );

    let dir_str = match jstring_to_string(&mut env, output_dir) {
        Ok(s) => s,
        Err(()) => return,
    };

    let output_path = PathBuf::from(dir_str);
    let pm = Arc::new(PathManager::with_output_dir(output_path));

    // OnceLock::set fails silently if already initialised — that's fine.
    let _ = PATH_MANAGER.set(pm);

    log::info!("Transmute JNI initialised");
}

// ---------------------------------------------------------------------------
// 2. convertImage(inputPath: String, format: String): String
// ---------------------------------------------------------------------------

/// Convert a single image to the requested format.
///
/// Returns the path of the output file, or throws RuntimeException on error.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_transmute_TransmuteLib_convertImage(
    mut env: JNIEnv,
    _class: JClass,
    input_path: JString,
    format: JString,
) -> jstring {
    let null_ret = JObject::null().into_raw() as jstring;

    let input_str = match jstring_to_string(&mut env, input_path) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };
    let format_str = match jstring_to_string(&mut env, format) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };

    let pm = match get_path_manager() {
        Some(pm) => pm,
        None => throw!(env, "TransmuteLib.init() was not called", null_ret),
    };

    let target_format = match MediaFormat::from_extension(&format_str) {
        Some(f) => f,
        None => throw!(env, format!("Unsupported format: {format_str}"), null_ret),
    };

    // Decode
    let input_path = PathBuf::from(&input_str);
    let (img, _meta) = match ImageDecoder::decode(&input_path) {
        Ok(r) => r,
        Err(e) => throw!(env, format!("Decode failed: {e}"), null_ret),
    };

    // Generate output path
    let output_path = match pm.generate_unique_path(&input_path, target_format.extension(), None) {
        Ok(p) => p,
        Err(e) => throw!(env, format!("Path generation failed: {e}"), null_ret),
    };

    // Encode
    if let Err(e) = ImageEncoder::encode(&img, &output_path, target_format) {
        throw!(env, format!("Encode failed: {e}"), null_ret);
    }

    log::info!("convertImage: {:?} → {:?}", input_path, output_path);
    string_to_jstring(&mut env, output_path.to_string_lossy().as_ref())
}

// ---------------------------------------------------------------------------
// 3. compressImage(inputPath: String, format: String, quality: String): String
// ---------------------------------------------------------------------------

/// Compress a single image with the given quality preset.
///
/// `quality` — one of: `"low"`, `"medium"`, `"high"`, `"maximum"`.
/// Returns the path of the compressed output file.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_transmute_TransmuteLib_compressImage(
    mut env: JNIEnv,
    _class: JClass,
    input_path: JString,
    format: JString,
    quality: JString,
) -> jstring {
    let null_ret = JObject::null().into_raw() as jstring;

    let input_str = match jstring_to_string(&mut env, input_path) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };
    let format_str = match jstring_to_string(&mut env, format) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };
    let quality_str = match jstring_to_string(&mut env, quality) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };

    let pm = match get_path_manager() {
        Some(pm) => pm,
        None => throw!(env, "TransmuteLib.init() was not called", null_ret),
    };

    let target_format = match MediaFormat::from_extension(&format_str) {
        Some(f) => f,
        None => throw!(env, format!("Unsupported format: {format_str}"), null_ret),
    };

    let quality_setting = match quality_str.to_lowercase().as_str() {
        "low" => QualitySettings::Low,
        "high" => QualitySettings::High,
        "maximum" => QualitySettings::Maximum,
        _ => QualitySettings::Balanced,
    };

    let input_path = PathBuf::from(&input_str);
    let (img, _meta) = match ImageDecoder::decode(&input_path) {
        Ok(r) => r,
        Err(e) => throw!(env, format!("Decode failed: {e}"), null_ret),
    };

    let output_path = match pm.generate_unique_path(&input_path, target_format.extension(), None) {
        Ok(p) => p,
        Err(e) => throw!(env, format!("Path generation failed: {e}"), null_ret),
    };

    // CPU-only compressor (use_gpu = false — no GPU on Android builds)
    let compressor = match ImageCompressor::new(false) {
        Ok(c) => c,
        Err(e) => throw!(env, format!("Compressor init failed: {e}"), null_ret),
    };

    if let Err(e) = compressor.compress_to_file(&img, &output_path, target_format, quality_setting) {
        throw!(env, format!("Compression failed: {e}"), null_ret);
    }

    log::info!("compressImage: {:?} → {:?}", input_path, output_path);
    string_to_jstring(&mut env, output_path.to_string_lossy().as_ref())
}

// ---------------------------------------------------------------------------
// 4. batchConvert(inputPaths: Array<String>, format: String): Array<String>
// ---------------------------------------------------------------------------

/// Convert multiple images in parallel (rayon).
///
/// Returns a Java String[] of output paths in the same order as input.
/// Individual failures produce an empty string at that index rather than
/// aborting the whole batch.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_transmute_TransmuteLib_batchConvert(
    mut env: JNIEnv,
    _class: JClass,
    input_paths: JObjectArray,
    format: JString,
) -> jobjectArray {
    let null_ret = JObject::null().into_raw() as jobjectArray;

    let format_str = match jstring_to_string(&mut env, format) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };

    let pm = match get_path_manager() {
        Some(pm) => pm,
        None => throw!(env, "TransmuteLib.init() was not called", null_ret),
    };

    let target_format = match MediaFormat::from_extension(&format_str) {
        Some(f) => f,
        None => throw!(env, format!("Unsupported format: {format_str}"), null_ret),
    };

    // Collect input paths from the Java array
    let len = match env.get_array_length(&input_paths) {
        Ok(n) => n,
        Err(e) => throw!(env, format!("Array length failed: {e}"), null_ret),
    };

    let mut inputs: Vec<String> = Vec::with_capacity(len as usize);
    for i in 0..len {
        let elem = match env.get_object_array_element(&input_paths, i) {
            Ok(o) => o,
            Err(e) => throw!(env, format!("Array element {i} failed: {e}"), null_ret),
        };
        let js: JString = elem.into();
        match jstring_to_string(&mut env, js) {
            Ok(s) => inputs.push(s),
            Err(()) => return null_ret,
        }
    }

    // Process in parallel, collecting results (failures → empty string)
    use rayon::prelude::*;
    let pm_ref = pm.as_ref();
    let results: Vec<String> = inputs
        .par_iter()
        .map(|input_str| {
            let input_path = PathBuf::from(input_str);
            let (img, _meta) = match ImageDecoder::decode(&input_path) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("batchConvert decode error for {:?}: {e}", input_path);
                    return String::new();
                }
            };
            let output_path = match pm_ref.generate_unique_path(&input_path, target_format.extension(), None) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("batchConvert path error for {:?}: {e}", input_path);
                    return String::new();
                }
            };
            if let Err(e) = ImageEncoder::encode(&img, &output_path, target_format) {
                log::error!("batchConvert encode error for {:?}: {e}", input_path);
                return String::new();
            }
            output_path.to_string_lossy().into_owned()
        })
        .collect();

    // Build a Java String[] with the results
    let string_class = match env.find_class("java/lang/String") {
        Ok(c) => c,
        Err(e) => throw!(env, format!("Class lookup failed: {e}"), null_ret),
    };
    let out_array = match env.new_object_array(results.len() as i32, &string_class, JObject::null()) {
        Ok(a) => a,
        Err(e) => throw!(env, format!("Array creation failed: {e}"), null_ret),
    };

    for (i, result) in results.iter().enumerate() {
        let js = match env.new_string(result) {
            Ok(s) => s,
            Err(e) => throw!(env, format!("String creation failed: {e}"), null_ret),
        };
        if let Err(e) = env.set_object_array_element(&out_array, i as i32, js) {
            throw!(env, format!("Array set failed: {e}"), null_ret);
        }
    }

    out_array.into_raw()
}

// ---------------------------------------------------------------------------
// 5. imagesToPdf(inputPaths: Array<String>, outputPath: String): String
// ---------------------------------------------------------------------------

/// Combine multiple images into a single PDF document.
///
/// This function uses `PdfGenerator`, which is built on `printpdf` and `lopdf`.
/// Neither of those crates requires native system libraries, so this function
/// works on Android without any extra bundling. The `pdf-extract` feature
/// (pdfium-render) is NOT needed here and is correctly absent from
/// `transmute-jni/Cargo.toml`.
///
/// Returns `outputPath` on success, or throws RuntimeException on error.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_transmute_TransmuteLib_imagesToPdf(
    mut env: JNIEnv,
    _class: JClass,
    input_paths: JObjectArray,
    output_path: JString,
) -> jstring {
    let null_ret = JObject::null().into_raw() as jstring;

    let out_str = match jstring_to_string(&mut env, output_path) {
        Ok(s) => s,
        Err(()) => return null_ret,
    };

    // Collect input paths
    let len = match env.get_array_length(&input_paths) {
        Ok(n) => n,
        Err(e) => throw!(env, format!("Array length failed: {e}"), null_ret),
    };

    let mut images = Vec::with_capacity(len as usize);
    for i in 0..len {
        let elem = match env.get_object_array_element(&input_paths, i) {
            Ok(o) => o,
            Err(e) => throw!(env, format!("Array element {i} failed: {e}"), null_ret),
        };
        let js: JString = elem.into();
        let path_str = match jstring_to_string(&mut env, js) {
            Ok(s) => s,
            Err(()) => return null_ret,
        };
        let input_path = PathBuf::from(&path_str);
        let (img, _meta) = match ImageDecoder::decode(&input_path) {
            Ok(r) => r,
            Err(e) => throw!(env, format!("Decode failed for {path_str}: {e}"), null_ret),
        };
        images.push((img, input_path));
    }

    let output_path = PathBuf::from(&out_str);
    let generator = PdfGenerator::new(PdfOptions::default());

    if let Err(e) = generator.generate_from_images(images, &output_path) {
        throw!(env, format!("PDF generation failed: {e}"), null_ret);
    }

    log::info!("imagesToPdf: {} images → {:?}", len, output_path);
    string_to_jstring(&mut env, &out_str)
}

// ---------------------------------------------------------------------------
// 6. pdfExtractSupported(): Boolean
// ---------------------------------------------------------------------------

/// Report whether PDF-to-image extraction is available in this build.
///
/// # Why this always returns `false` on Android
///
/// `PdfExtractor` (PDF → images) is backed by `pdfium-render`, which wraps
/// Google's `libpdfium.so`. That native library:
///   - Is ~30 MB per ABI and not distributed via crates.io for Android targets.
///   - Requires manual download, compilation, and APK bundling that is out of
///     scope for this build.
///   - Is gated behind the `pdf-extract` feature flag, which `transmute-jni`
///     does NOT enable (see `transmute-jni/Cargo.toml`).
///
/// The Kotlin UI must call this function once (e.g. in `onCreate`) and
/// hide or disable any "Extract PDF" controls when it returns `false`.
///
/// If pdfium support is ever added to the Android build, this function should
/// be updated to return `true` and the UI will automatically become visible.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_transmute_TransmuteLib_pdfExtractSupported(
    _env: JNIEnv,
    _class: JClass,
) -> jni::sys::jboolean {
    // pdfium-render is not compiled into this Android build — extraction is
    // unavailable. Return JNI_FALSE so Kotlin can gate the UI accordingly.
    jni::sys::JNI_FALSE
}
