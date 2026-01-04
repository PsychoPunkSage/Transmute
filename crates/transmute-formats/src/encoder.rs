use image::{DynamicImage, ImageEncoder as _, ImageFormat};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use transmute_common::{Error, MediaFormat, Result};

const EIGHT_MB_IN_BYTES: usize = 8 * 1024 * 1024;

/// Image encoder with format-specific optimizations
pub struct ImageEncoder;

impl ImageEncoder {
    /// Encode image to specified format at given path
    pub fn encode(img: &DynamicImage, output_path: &Path, format: MediaFormat) -> Result<()> {
        let image_format = format
            .to_image_format()
            .ok_or_else(|| Error::UnsupportedFormat(format.to_string()))?;

        tracing::debug!("Encoding to {:?} at {:?}", format, output_path);

        // Use buffered writer for better I/O performance
        let file = File::create(output_path)?;
        let writer = BufWriter::with_capacity(EIGHT_MB_IN_BYTES, file); // 8MB buffer

        // Format-specific encoding with optimizations
        match image_format {
            ImageFormat::Png => {
                let encoder = image::codecs::png::PngEncoder::new(writer);
                encoder.write_image(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )?;
            }
            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(writer, 95);
                encoder.write_image(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )?;
            }
            _ => {
                // Fallback to generic encoder for other formats
                img.write_to(
                    &mut std::io::BufWriter::new(File::create(output_path)?),
                    image_format,
                )?;
            }
        }

        tracing::info!("Successfully encoded to {:?}", output_path);
        Ok(())
    }

    /// Encode with custom quality (where applicable)
    pub fn encode_with_quality(
        img: &DynamicImage,
        output_path: &Path,
        format: MediaFormat,
        quality: u8,
    ) -> Result<()> {
        let image_format = format
            .to_image_format()
            .ok_or_else(|| Error::UnsupportedFormat(format.to_string()))?;

        let file = File::create(output_path)?;
        let writer = BufWriter::with_capacity(EIGHT_MB_IN_BYTES, file);

        match image_format {
            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(writer, quality);
                encoder.write_image(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )?;
            }
            ImageFormat::WebP => {
                // WebP supports quality parameter
                // FIX: Where to use qualiity parameter
                img.write_to(
                    &mut std::io::BufWriter::new(File::create(output_path)?),
                    image_format,
                )?;
            }
            _ => {
                // Formats without quality control fall back to default
                Self::encode(img, output_path, format)?;
            }
        }

        Ok(())
    }
}
