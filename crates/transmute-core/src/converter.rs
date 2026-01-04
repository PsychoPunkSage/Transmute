use rayon::prelude::*;
use std::path::{Path, PathBuf};
use transmute_common::{MediaFormat, PathManager, Result};
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
}
