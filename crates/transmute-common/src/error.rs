use std::path::PathBuf;

/// Unified error type for all transmute operations
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("GPU operation failed: {0}")]
    GpuError(String),

    #[error("Conversion failed: {0}")]
    ConversionError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
