use std::path::Path;

/// Supported media formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaFormat {
    Png,
    Jpeg,
    Webp,
    Tiff,
    Bmp,
    Gif,
    Pdf,
}

impl MediaFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Parse from extension string
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            "tif" | "tiff" => Some(Self::Tiff),
            "bmp" => Some(Self::Bmp),
            "gif" => Some(Self::Gif),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    /// Get primary file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Tiff => "tiff",
            Self::Bmp => "bmp",
            Self::Gif => "gif",
            Self::Pdf => "pdf",
        }
    }

    /// Check if format is an image
    pub fn is_image(&self) -> bool {
        !matches!(self, Self::Pdf)
    }

    /// Convert to image crate's ImageFormat
    pub fn to_image_format(&self) -> Option<image::ImageFormat> {
        match self {
            Self::Png => Some(image::ImageFormat::Png),
            Self::Jpeg => Some(image::ImageFormat::Jpeg),
            Self::Webp => Some(image::ImageFormat::WebP),
            Self::Tiff => Some(image::ImageFormat::Tiff),
            Self::Bmp => Some(image::ImageFormat::Bmp),
            Self::Gif => Some(image::ImageFormat::Gif),
            Self::Pdf => None,
        }
    }

    /// Check if format supports multi-page documents
    pub fn supports_multipage(&self) -> bool {
        matches!(self, Self::Pdf)
    }

    /// Get MIME type for HTTP/export
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Tiff => "image/tiff",
            Self::Bmp => "image/bmp",
            Self::Gif => "image/gif",
            Self::Pdf => "application/pdf",
        }
    }
}

impl std::fmt::Display for MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension().to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(MediaFormat::from_extension("png"), Some(MediaFormat::Png));
        assert_eq!(MediaFormat::from_extension("JPG"), Some(MediaFormat::Jpeg));
        assert_eq!(MediaFormat::from_extension("unknown"), None);
    }
}
