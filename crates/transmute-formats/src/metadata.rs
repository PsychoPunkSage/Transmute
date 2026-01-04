use transmute_common::MediaFormat;

/// Image metadata extracted during decoding
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: MediaFormat,
    pub color_type: image::ColorType,
    pub has_alpha: bool,
}

impl ImageMetadata {
    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    pub fn estimated_memory_mb(&self) -> f32 {
        let bytes = self.pixel_count() * self.color_type.bytes_per_pixel() as usize;
        bytes as f32 / (1024.0 * 1024.0)
    }
}
