use image::{DynamicImage, ImageBuffer, Rgba};
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Pt, RawImage, XObjectTransform};
use std::path::{Path, PathBuf};
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
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            page_width_mm: 210.0,  // A4 width
            page_height_mm: 297.0, // A4 height
            dpi: 300.0,
            title: "Transmute Generated PDF".into(),
            compress_images: true,
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

            // Convert DynamicImage to PNG bytes for RawImage
            let mut png_bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )?;

            // Decode into RawImage
            let raw_image = RawImage::decode_from_bytes(&png_bytes, &mut Vec::new())
                .map_err(|e| Error::ConversionError(format!("Failed to decode image: {:?}", e)))?;

            // Add image to document resources and get ID
            let image_id = doc.add_image(&raw_image);

            // Calculate scaling to fit page while preserving aspect ratio
            let (fit_width_mm, fit_height_mm) =
                self.calculate_fit_dimensions(img.width() as f32, img.height() as f32);

            // Center image on page
            let x_offset = (self.options.page_width_mm - fit_width_mm) / 2.0;
            let y_offset = (self.options.page_height_mm - fit_height_mm) / 2.0;

            // Create page operations
            let ops = vec![Op::UseXobject {
                id: image_id,
                transform: XObjectTransform {
                    translate_x: Some(Pt(x_offset * 2.834645)), // mm to pt conversion
                    translate_y: Some(Pt(y_offset * 2.834645)),
                    scale_x: Some(fit_width_mm / (img.width() as f32 / self.options.dpi * 25.4)),
                    scale_y: Some(fit_height_mm / (img.height() as f32 / self.options.dpi * 25.4)),
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

        assert!(result.is_ok());
        assert!(temp_pdf.path().exists());
    }
}
