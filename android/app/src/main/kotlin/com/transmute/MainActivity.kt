package com.transmute

import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.MediaStore
import android.view.View
import android.widget.*
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : AppCompatActivity() {

    // ── Format options shown in the spinner ────────────────────────────────
    private val formats = listOf("png", "jpg", "webp")

    // ── Selected files (absolute paths) ────────────────────────────────────
    private var selectedPaths: List<String> = emptyList()

    // ── UI references ──────────────────────────────────────────────────────
    private lateinit var statusText: TextView
    private lateinit var formatSpinner: Spinner
    private lateinit var qualitySpinner: Spinner
    private lateinit var extractPdfButton: Button

    // ── File picker ────────────────────────────────────────────────────────
    private val pickImages =
        registerForActivityResult(ActivityResultContracts.GetMultipleContents()) { uris ->
            if (uris.isNullOrEmpty()) return@registerForActivityResult
            selectedPaths = uris.mapNotNull { uri -> uriToPath(uri) }
            statusText.text = "${selectedPaths.size} file(s) selected"
        }

    // ── Storage permission request ─────────────────────────────────────────
    private val requestPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            if (granted) pickImages.launch("image/*")
            else statusText.text = "Storage permission required to pick files."
        }

    // ── Lifecycle ──────────────────────────────────────────────────────────

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Initialise the Rust library with the app's external files directory
        val outputDir = getExternalFilesDir(null)?.absolutePath
            ?: filesDir.absolutePath
        TransmuteLib.init(outputDir)

        // Bind UI
        statusText   = findViewById(R.id.statusText)
        formatSpinner = findViewById(R.id.formatSpinner)
        qualitySpinner = findViewById(R.id.qualitySpinner)

        formatSpinner.adapter = ArrayAdapter(
            this, android.R.layout.simple_spinner_item, formats
        ).also { it.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item) }

        qualitySpinner.adapter = ArrayAdapter(
            this, android.R.layout.simple_spinner_item,
            listOf("balanced", "high", "maximum", "low")
        ).also { it.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item) }

        findViewById<Button>(R.id.pickButton).setOnClickListener { onPickClicked() }
        findViewById<Button>(R.id.convertButton).setOnClickListener { onConvertClicked() }
        findViewById<Button>(R.id.toPdfButton).setOnClickListener { onToPdfClicked() }

        // PDF extraction requires pdfium (libpdfium.so), which is not bundled
        // in the Android APK. Hide the button when the feature is unavailable
        // so users are not presented with an option that will always fail.
        extractPdfButton = findViewById(R.id.extractPdfButton)
        extractPdfButton.visibility =
            if (TransmuteLib.pdfExtractSupported()) View.VISIBLE else View.GONE
    }

    // ── Button handlers ────────────────────────────────────────────────────

    private fun onPickClicked() {
        val permission = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            android.Manifest.permission.READ_MEDIA_IMAGES
        } else {
            android.Manifest.permission.READ_EXTERNAL_STORAGE
        }

        if (ContextCompat.checkSelfPermission(this, permission) ==
            PackageManager.PERMISSION_GRANTED
        ) {
            pickImages.launch("image/*")
        } else {
            requestPermission.launch(permission)
        }
    }

    private fun onConvertClicked() {
        if (selectedPaths.isEmpty()) {
            statusText.text = "Please select image(s) first."
            return
        }

        val format  = formatSpinner.selectedItem as String
        val quality = qualitySpinner.selectedItem as String
        statusText.text = "Converting…"

        lifecycleScope.launch {
            try {
                val results = withContext(Dispatchers.IO) {
                    if (selectedPaths.size == 1) {
                        // Single file: compress with quality settings
                        listOf(TransmuteLib.compressImage(selectedPaths[0], format, quality))
                    } else {
                        // Multiple files: batch convert (format only, no quality)
                        TransmuteLib.batchConvert(selectedPaths.toTypedArray(), format).toList()
                    }
                }
                val success = results.count { it.isNotEmpty() }
                statusText.text = "Done! $success/${results.size} file(s) converted.\n${results.firstOrNull()}"
            } catch (e: RuntimeException) {
                statusText.text = "Error: ${e.message}"
            }
        }
    }

    private fun onToPdfClicked() {
        if (selectedPaths.isEmpty()) {
            statusText.text = "Please select image(s) first."
            return
        }

        val outputPath = (getExternalFilesDir(null)?.absolutePath ?: filesDir.absolutePath) +
            "/transmute_${System.currentTimeMillis()}.pdf"

        statusText.text = "Generating PDF…"

        lifecycleScope.launch {
            try {
                val result = withContext(Dispatchers.IO) {
                    TransmuteLib.imagesToPdf(selectedPaths.toTypedArray(), outputPath)
                }
                statusText.text = "PDF saved:\n$result"
            } catch (e: RuntimeException) {
                statusText.text = "Error: ${e.message}"
            }
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    /** Resolve a content:// URI to a real filesystem path. */
    private fun uriToPath(uri: Uri): String? {
        if (uri.scheme == "file") return uri.path

        val projection = arrayOf(MediaStore.Images.Media.DATA)
        return contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) {
                val col = cursor.getColumnIndexOrThrow(MediaStore.Images.Media.DATA)
                cursor.getString(col)
            } else null
        }
    }
}
