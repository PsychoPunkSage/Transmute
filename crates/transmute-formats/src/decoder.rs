use crate::metadata::ImageMetadata;
use image::{DynamicImage, ImageReader};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use transmute_common::{Error, MediaFormat, Result};

const TEN_MB_IN_BYTES: u64 = 10 * 1024 * 1024;

/// High-performance image decoder with memory-mapped I/O
pub struct ImageDecoder;

impl ImageDecoder {
    /// Decode image from path using memory-mapped file for large images
    pub fn decode(path: &Path) -> Result<(DynamicImage, ImageMetadata)> {
        let format = MediaFormat::from_path(path).ok_or_else(|| {
            Error::UnsupportedFormat(
                path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        })?;

        if !format.is_image() {
            return Err(Error::UnsupportedFormat(format.to_string()));
        }

        tracing::debug!("Decoding {format:?} from {path:?}");

        // Use memory-mapped I/O for files > 10MB
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        let img = if metadata.len() > TEN_MB_IN_BYTES {
            // Memory-mapped decoding for large files
            tracing::debug!("Using memory-mapped I/O for large file");
            /*
             * MMAP Theory
             * - It tells kernel to MAP a particular file to its Virtual Address Space.
             * - Kernel replies with the `fake memory range` that corresponds to that file. <NO BYTES IS LOADED YET>
             *
             * -> Whenever code touched That Address
             *   * kernel loads disk page into RAM
             *   * kernel maps RAM page into your address space
             *
             * => i.e. bytes load only when accessed and only for the pages you actually touch.
             * */
            let mmap = unsafe { Mmap::map(&file)? };
            ImageReader::new(std::io::Cursor::new(&mmap[..]))
                .with_guessed_format()?
                .decode()?
        } else {
            // Standard decoding for smaller files
            image::open(path)?
        };

        let img_metadata = ImageMetadata {
            width: img.width(),
            height: img.height(),
            format,
            color_type: img.color(),
            has_alpha: img.color().has_alpha(),
        };

        tracing::info!(
            "Decoded {}x{} {} image ({:.2}MB in memory)",
            img_metadata.width,
            img_metadata.height,
            img_metadata.format,
            img_metadata.estimated_memory_mb()
        );

        Ok((img, img_metadata))
    }

    /// Quick metadata extraction without full decode
    pub fn probe(path: &Path) -> Result<ImageMetadata> {
        let reader = ImageReader::open(path)?;
        let format = MediaFormat::from_path(path)
            .ok_or_else(|| Error::UnsupportedFormat("unknown".to_string()))?;

        if let Ok((width, height)) = reader.into_dimensions() {
            // Try to get color info without full decode
            // FIX: DANGER, will cause OOM (Out of Memory).
            let img = image::open(path)?; // Fallback to full decode for color type
            Ok(ImageMetadata {
                width,
                height,
                format,
                color_type: img.color(),
                has_alpha: img.color().has_alpha(),
            })
        } else {
            Err(Error::ConversionError(
                "Failed to probe image dimensions".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_png() {
        // Create a simple 1x1 PNG for testing
        let temp = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .unwrap();
        let img = image::DynamicImage::new_rgb8(1, 1);
        img.save_with_format(temp.path(), image::ImageFormat::Png)
            .unwrap();

        let (_, metadata) = ImageDecoder::decode(temp.path()).unwrap();
        assert_eq!(metadata.width, 1);
        assert_eq!(metadata.height, 1);
        assert_eq!(metadata.format, MediaFormat::Png);
    }
}
