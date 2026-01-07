use rayon::prelude::*;
use std::path::{Path, PathBuf};
use transmute_common::{Error, MediaFormat, PathManager, Result};
use transmute_formats::{ImageDecoder, ImageEncoder};

/// Main conversion engine
pub struct Converter {
    path_manager: PathManager,
    use_gpu: bool,
}

impl Converter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            path_manager: PathManager::new()?,
            use_gpu: false, // GPU conversion in Phase 3+
        })
    }

    /// Convert single image to target format
    pub fn convert_image(
        &self,
        input: &Path,
        target_format: MediaFormat,
        output: Option<PathBuf>,
    ) -> Result<PathBuf> {
        // Validate input
        self.path_manager.validate_input(input)?;

        // Decode
        let (img, metadata) = ImageDecoder::decode(input)?;

        tracing::info!(
            "Converting {}x{} {} → {}",
            metadata.width,
            metadata.height,
            metadata.format,
            target_format
        );

        // Generate output path
        let output_path =
            self.path_manager
                .generate_unique_path(input, target_format.extension(), output)?;

        // Encode
        ImageEncoder::encode(&img, &output_path, target_format)?;

        Ok(output_path)
    }

    /// Convert batch of images in parallel
    pub fn convert_batch(
        &self,
        inputs: Vec<PathBuf>,
        target_format: MediaFormat,
        output_dir: Option<PathBuf>,
    ) -> Vec<Result<PathBuf>> {
        inputs
            .par_iter()
            .map(|input| self.convert_image(input, target_format, output_dir.clone()))
            .collect()
    }

    /// Enable/disable GPU acceleration
    pub fn set_gpu_enabled(&mut self, enabled: bool) {
        self.use_gpu = enabled;
    }

    pub fn images_to_pdf(
        &self,
        input_images: Vec<PathBuf>,
        output: PathBuf,
        pdf_options: Option<transmute_formats::PdfOptions>,
    ) -> Result<PathBuf> {
        use transmute_formats::{ImageDecoder, PdfGenerator};

        tracing::info!("Converting {} images to PDF", input_images.len());

        // Validate all inputs exist
        for input in &input_images {
            self.path_manager.validate_input(input)?;
        }

        // Decode all images
        let mut images_with_paths = Vec::new();
        for input_path in input_images {
            let (img, _metadata) = ImageDecoder::decode(&input_path)?;
            images_with_paths.push((img, input_path));
        }

        // Generate PDF
        let options = pdf_options.unwrap_or_default();
        let generator = PdfGenerator::new(options);
        generator.generate_from_images(images_with_paths, &output)?;

        tracing::info!("PDF created at {:?}", output);
        Ok(output)
    }

    /// Extract PDF pages to individual images
    pub fn pdf_to_images(
        &self,
        pdf_path: &Path,
        output_format: MediaFormat,
        output_dir: Option<PathBuf>,
        dpi: Option<f32>,
    ) -> Result<Vec<PathBuf>> {
        use transmute_formats::{ImageEncoder, PdfExtractor};

        if !output_format.is_image() {
            return Err(Error::UnsupportedFormat(format!(
                "Cannot convert PDF to non-image format: {}",
                output_format
            )));
        }

        tracing::info!("Extracting PDF pages from {:?}", pdf_path);

        // Extract pages as images
        let extractor = PdfExtractor::new(dpi.unwrap_or(300.0));
        let images = extractor.extract_pages(pdf_path)?;

        tracing::info!(
            "Extracted {} pages, encoding to {}",
            images.len(),
            output_format
        );

        // Save each page as separate image
        let mut output_paths = Vec::new();
        for (page_num, img) in images.into_iter().enumerate() {
            // Generate unique path with page number
            let base_name = pdf_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("page");

            let output_dir = output_dir
                .clone()
                .unwrap_or_else(|| self.path_manager.default_output_dir().to_path_buf());

            std::fs::create_dir_all(&output_dir)?;

            let output_path = output_dir.join(format!(
                "{}_page_{:03}.{}",
                base_name,
                page_num + 1,
                output_format.extension()
            ));

            ImageEncoder::encode(&img, &output_path, output_format)?;
            output_paths.push(output_path);
        }

        tracing::info!("Saved {} images to {:?}", output_paths.len(), output_dir);
        Ok(output_paths)
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new().expect("Failed to create Converter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use tempfile::TempDir;

    #[test]
    fn test_image_conversion() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("test.png");

        // Create test image
        let img = DynamicImage::new_rgb8(100, 100);
        img.save(&input_path).unwrap();

        let converter = Converter::new().unwrap();
        let output = converter.convert_image(
            &input_path,
            MediaFormat::Jpeg,
            Some(temp_dir.path().to_path_buf()),
        );

        assert!(output.is_ok());
        assert!(output.unwrap().exists());
    }

    #[test]
    fn test_batch_conversion() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut inputs = Vec::new();

        // Create 3 test images
        for i in 0..3 {
            let path = temp_dir.path().join(format!("test{}.png", i));
            let img = DynamicImage::new_rgb8(50, 50);
            img.save(&path).unwrap();
            inputs.push(path);
        }

        let converter = Converter::new().unwrap();
        let results = converter.convert_batch(
            inputs,
            MediaFormat::Jpeg,
            Some(temp_dir.path().to_path_buf()),
        );

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_images_to_pdf() {
        let temp_dir = TempDir::new().unwrap();
        let converter = Converter::new().unwrap();

        // Create test images
        let mut inputs = Vec::new();
        for i in 0..3 {
            let path = temp_dir.path().join(format!("page{}.png", i));
            let img = DynamicImage::new_rgb8(800, 600);
            img.save(&path).unwrap();
            inputs.push(path);
        }

        let output = temp_dir.path().join("output.pdf");
        let result = converter.images_to_pdf(inputs, output.clone(), None);

        assert!(result.is_ok());
        assert!(output.exists());
    }
}
