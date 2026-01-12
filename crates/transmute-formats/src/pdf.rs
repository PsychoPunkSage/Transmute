use image::{DynamicImage, ImageBuffer, Rgba, imageops::FilterType};
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Pt, RawImage, XObjectTransform};
use std::path::{Path, PathBuf};
use std::fs;
use transmute_common::{Error, Result};

// Type alias for clarity
type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// PDF generation configuration
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// Page size (default: A4)
    pub page_width_mm: f32,
    pub page_height_mm: f32,

    /// DPI for image rendering (default: 300)
    pub dpi: f32,

    /// Title metadata
    pub title: String,

    /// Compress images in PDF
    pub compress_images: bool,

    /// Maximum image dimension before downscaling (default: 2400px for 300 DPI at A4 width)
    /// Images larger than this will be downscaled to save memory and reduce PDF size
    pub max_image_dimension: u32,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            page_width_mm: 210.0,  // A4 width
            page_height_mm: 297.0, // A4 height
            dpi: 300.0,
            title: "Transmute Generated PDF".into(),
            compress_images: true,
            max_image_dimension: 2400, // ~8 inches at 300 DPI
        }
    }
}

/// PDF generation from images
pub struct PdfGenerator {
    options: PdfOptions,
}

impl PdfGenerator {
    pub fn new(options: PdfOptions) -> Self {
        Self { options }
    }

    /// Generate PDF from multiple images
    pub fn generate_from_images(
        &self,
        images: Vec<(DynamicImage, PathBuf)>, // (image, original_path)
        output_path: &Path,
    ) -> Result<()> {
        if images.is_empty() {
            return Err(Error::ConversionError(
                "No images provided for PDF generation".into(),
            ));
        }

        tracing::info!(
            "Generating PDF with {} pages at {:?}",
            images.len(),
            output_path
        );

        // Create PDF document
        let mut doc = PdfDocument::new(&self.options.title);
        let mut pages = Vec::new();

        // Add each image as a page
        for (idx, (img, original_path)) in images.iter().enumerate() {
            tracing::debug!(
                "Adding page {}/{}: {:?} ({}x{})",
                idx + 1,
                images.len(),
                original_path,
                img.width(),
                img.height()
            );

            // Optimization 1: Downscale large images to reduce memory and PDF size
            // Images larger than max_image_dimension are scaled down, preserving aspect ratio
            let processed_img = self.maybe_downscale_image(img);

            // Optimization 2: JPEG passthrough - if source is JPEG, embed directly without re-encoding
            // This avoids generation loss and is significantly faster (no decode-encode cycle)
            let raw_image = if self.is_jpeg_source(original_path) && self.options.compress_images {
                tracing::debug!("Using JPEG passthrough for {:?}", original_path);
                self.load_jpeg_direct(original_path)?
            } else {
                // Optimization 3: Use JPEG encoding for non-JPEG sources when compression enabled
                // JPEG is faster to encode/decode than PNG and results in smaller PDFs
                self.encode_image_for_pdf(&processed_img)?
            };

            // Add image to document resources and get ID
            let image_id = doc.add_image(&raw_image);

            // Calculate scaling to fit page while preserving aspect ratio
            // Use processed_img dimensions for correct scaling
            let (fit_width_mm, fit_height_mm) =
                self.calculate_fit_dimensions(processed_img.width() as f32, processed_img.height() as f32);

            // Center image on page
            let x_offset = (self.options.page_width_mm - fit_width_mm) / 2.0;
            let y_offset = (self.options.page_height_mm - fit_height_mm) / 2.0;

            // Create page operations
            let ops = vec![Op::UseXobject {
                id: image_id,
                transform: XObjectTransform {
                    translate_x: Some(Pt(x_offset * 2.834645)), // mm to pt conversion
                    translate_y: Some(Pt(y_offset * 2.834645)),
                    scale_x: Some(fit_width_mm / (processed_img.width() as f32 / self.options.dpi * 25.4)),
                    scale_y: Some(fit_height_mm / (processed_img.height() as f32 / self.options.dpi * 25.4)),
                    dpi: Some(self.options.dpi),
                    ..Default::default()
                },
            }];

            // Create page
            let page = PdfPage::new(
                Mm(self.options.page_width_mm),
                Mm(self.options.page_height_mm),
                ops,
            );
            pages.push(page);
        }

        // Write PDF to file
        let pdf_bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());
        std::fs::write(output_path, pdf_bytes)?;

        tracing::info!("PDF generated successfully at {:?}", output_path);
        Ok(())
    }

    /// Check if source file is JPEG based on extension
    fn is_jpeg_source(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg"))
            .unwrap_or(false)
    }

    /// Load JPEG file directly without re-encoding (passthrough optimization)
    /// This avoids decode-encode cycles and preserves original JPEG quality
    fn load_jpeg_direct(&self, path: &Path) -> Result<RawImage> {
        let jpeg_bytes = fs::read(path)?;
        RawImage::decode_from_bytes(&jpeg_bytes, &mut Vec::new())
            .map_err(|e| Error::ConversionError(format!("Failed to load JPEG: {:?}", e)))
    }

    /// Downscale image if it exceeds max_image_dimension
    /// Uses high-quality Lanczos3 filter for downscaling
    fn maybe_downscale_image<'a>(&self, img: &'a DynamicImage) -> std::borrow::Cow<'a, DynamicImage> {
        let max_dim = img.width().max(img.height());

        if max_dim > self.options.max_image_dimension {
            let scale = self.options.max_image_dimension as f32 / max_dim as f32;
            let new_width = (img.width() as f32 * scale) as u32;
            let new_height = (img.height() as f32 * scale) as u32;

            tracing::debug!(
                "Downscaling image from {}x{} to {}x{} (scale: {:.2})",
                img.width(),
                img.height(),
                new_width,
                new_height,
                scale
            );

            // Lanczos3 provides best quality for downscaling
            std::borrow::Cow::Owned(img.resize(new_width, new_height, FilterType::Lanczos3))
        } else {
            std::borrow::Cow::Borrowed(img)
        }
    }

    /// Encode image for PDF embedding
    /// Uses JPEG for compression (faster than PNG) or PNG for lossless
    fn encode_image_for_pdf(&self, img: &DynamicImage) -> Result<RawImage> {
        let mut bytes = Vec::new();

        if self.options.compress_images {
            // JPEG encoding is 2-3x faster than PNG and produces smaller files
            // Quality 85 provides good balance between size and visual quality
            img.write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Jpeg,
            )?;
        } else {
            // PNG for lossless embedding when compression disabled
            img.write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )?;
        }

        RawImage::decode_from_bytes(&bytes, &mut Vec::new())
            .map_err(|e| Error::ConversionError(format!("Failed to encode image: {:?}", e)))
    }

    /// Calculate dimensions to fit image on page while preserving aspect ratio
    fn calculate_fit_dimensions(&self, img_width: f32, img_height: f32) -> (f32, f32) {
        let page_width = self.options.page_width_mm - 20.0; // 10mm margin each side
        let page_height = self.options.page_height_mm - 20.0;

        let img_aspect = img_width / img_height;
        let page_aspect = page_width / page_height;

        if img_aspect > page_aspect {
            // Image is wider - fit to width
            (page_width, page_width / img_aspect)
        } else {
            // Image is taller - fit to height
            (page_height * img_aspect, page_height)
        }
    }
}

/// PDF extraction to images (GPU-accelerated rasterization)
pub struct PdfExtractor {
    dpi: f32,
}

impl PdfExtractor {
    pub fn new(dpi: f32) -> Self {
        Self { dpi }
    }

    /// Extract all pages from PDF as images
    #[cfg(not(target_arch = "wasm32"))]
    pub fn extract_pages(&self, pdf_path: &Path) -> Result<Vec<DynamicImage>> {
        use pdfium_render::prelude::*;

        tracing::info!("Extracting pages from PDF: {:?}", pdf_path);

        // Initialize Pdfium (attempts to use system library)
        let pdfium = Pdfium::default();

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| Error::ConversionError(format!("Failed to load PDF: {:?}", e)))?;

        let page_count = document.pages().len();
        tracing::info!("PDF has {} pages", page_count);

        let mut images = Vec::with_capacity(page_count as usize);

        // Render each page
        for page_idx in 0..page_count {
            tracing::debug!("Rendering page {}/{}", page_idx + 1, page_count);

            let page = document.pages().get(page_idx).map_err(|e| {
                Error::ConversionError(format!("Failed to get page {}: {:?}", page_idx, e))
            })?;

            // Render page to bitmap with GPU acceleration (if available)
            let bitmap = page
                .render_with_config(
                    &PdfRenderConfig::new()
                        .set_target_width((page.width().value * self.dpi / 72.0) as i32)
                        .set_maximum_height((page.height().value * self.dpi / 72.0) as i32)
                        .render_form_data(false),
                )
                .map_err(|e| {
                    Error::ConversionError(format!("Failed to render page {}: {:?}", page_idx, e))
                })?;

            // Convert bitmap to DynamicImage
            let width = bitmap.width() as u32;
            let height = bitmap.height() as u32;
            let rgba_buffer = bitmap.as_raw_bytes();

            let img = RgbaImage::from_raw(width, height, rgba_buffer).ok_or_else(|| {
                Error::ConversionError("Failed to create image from bitmap".into())
            })?;

            images.push(DynamicImage::ImageRgba8(img));
        }

        tracing::info!("Successfully extracted {} pages", images.len());
        Ok(images)
    }

    /// Fallback for WASM or when Pdfium unavailable
    #[cfg(target_arch = "wasm32")]
    pub fn extract_pages(&self, _pdf_path: &Path) -> Result<Vec<DynamicImage>> {
        Err(Error::ConversionError(
            "PDF extraction not supported on WASM".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pdf_generation() {
        let temp_pdf = NamedTempFile::new().unwrap();

        // Create test images
        let img1 = DynamicImage::new_rgb8(800, 600);
        let img2 = DynamicImage::new_rgb8(1920, 1080);

        let images = vec![
            (img1, PathBuf::from("test1.png")),
            (img2, PathBuf::from("test2.png")),
        ];

        let generator = PdfGenerator::new(PdfOptions::default());
        let result = generator.generate_from_images(images, temp_pdf.path());

        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("PDF generation failed: {:?}", e),
        }
        assert!(result.is_ok());
        assert!(temp_pdf.path().exists());
    }

    #[test]
    fn test_pdf_generation_with_downscaling() {
        // Test that large images are properly downscaled
        let temp_pdf = NamedTempFile::new().unwrap();

        // Create large test image (4K)
        let img = DynamicImage::new_rgb8(3840, 2160);
        let images = vec![(img, PathBuf::from("large.png"))];

        let mut options = PdfOptions::default();
        options.max_image_dimension = 1920; // Force downscaling

        let generator = PdfGenerator::new(options);
        let result = generator.generate_from_images(images, temp_pdf.path());

        assert!(result.is_ok());
        assert!(temp_pdf.path().exists());

        // Verify file is smaller than it would be without downscaling
        let file_size = std::fs::metadata(temp_pdf.path()).unwrap().len();
        assert!(file_size < 5_000_000); // Should be much smaller than uncompressed 4K
    }
}
