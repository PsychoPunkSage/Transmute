package com.transmute

/**
 * Thin Kotlin wrapper around the Rust JNI library.
 *
 * The companion object's `init` block calls [System.loadLibrary] which
 * causes the JVM to load `libtransmute_jni.so` from the APK's jniLibs/.
 *
 * Each `external fun` is implemented in `crates/transmute-jni/src/lib.rs`.
 * The JNI naming convention maps:
 *   package `com.transmute`, class `TransmuteLib`, method `convertImage`
 *   → Rust export `Java_com_transmute_TransmuteLib_convertImage`
 */
object TransmuteLib {
    init {
        System.loadLibrary("transmute_jni")
    }

    /**
     * Initialise the library. **Must be called once** before any other
     * function, typically from [android.app.Activity.onCreate].
     *
     * @param outputDir Writable directory for output files.
     *   Use `context.getExternalFilesDir(null)?.absolutePath` or similar.
     */
    external fun init(outputDir: String)

    /**
     * Convert a single image to the requested format.
     *
     * @param inputPath Absolute path to the source image.
     * @param format    Target extension: `"png"`, `"jpg"`, `"webp"`, etc.
     * @return Absolute path of the converted output file.
     * @throws RuntimeException if the operation fails.
     */
    external fun convertImage(inputPath: String, format: String): String

    /**
     * Compress a single image with the given quality preset.
     *
     * @param inputPath Absolute path to the source image.
     * @param format    Target extension: `"jpg"`, `"png"`, `"webp"`.
     * @param quality   One of `"low"`, `"balanced"`, `"high"`, `"maximum"`.
     * @return Absolute path of the compressed output file.
     * @throws RuntimeException if the operation fails.
     */
    external fun compressImage(inputPath: String, format: String, quality: String): String

    /**
     * Convert multiple images in parallel.
     *
     * @param inputPaths Array of absolute paths to source images.
     * @param format     Target extension for all images.
     * @return Array of output paths in the same order as [inputPaths].
     *         Failed entries are empty strings.
     * @throws RuntimeException if a fatal error occurs.
     */
    external fun batchConvert(inputPaths: Array<String>, format: String): Array<String>

    /**
     * Combine multiple images into a single PDF.
     *
     * Uses `printpdf`/`lopdf` internally — no native pdfium library required.
     * Works on all Android targets without any extra bundling.
     *
     * @param inputPaths Array of absolute paths to source images (page order).
     * @param outputPath Absolute path for the output PDF file.
     * @return [outputPath] on success.
     * @throws RuntimeException if the operation fails.
     */
    external fun imagesToPdf(inputPaths: Array<String>, outputPath: String): String

    /**
     * Returns `true` if PDF-to-image extraction is available in this build.
     *
     * Extraction requires `pdfium-render` (wrapping Google's `libpdfium.so`),
     * which is not bundled in the Android APK. This function will return
     * `false` in all current Android builds.
     *
     * Use this to conditionally show or hide any "Extract PDF pages" UI:
     * ```kotlin
     * extractPdfButton.isVisible = TransmuteLib.pdfExtractSupported()
     * ```
     */
    external fun pdfExtractSupported(): Boolean
}
