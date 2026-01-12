use std::path::PathBuf;
use transmute_common::MediaFormat;

/// Parsed command intent
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    Convert(ConvertIntent),
    Compress(CompressIntent),
    Enhance(EnhanceIntent),
    Batch(BatchIntent),
    CombineToPdf(CombineToPdfIntent),
}

/// Convert one format to another
#[derive(Debug, Clone, PartialEq)]
pub struct ConvertIntent {
    pub input: PathBuf,
    pub target_format: MediaFormat,
    pub output: Option<PathBuf>,
}

/// Compress/optimize file
#[derive(Debug, Clone, PartialEq)]
pub struct CompressIntent {
    pub input: PathBuf,
    pub target_format: Option<MediaFormat>,
    pub quality: QualitySpec,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QualitySpec {
    Percentage(u8),        // e.g., "80%"
    Preset(QualityPreset), // e.g., "high"
    Default,               // No specification
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QualityPreset {
    Maximum,
    High,
    Medium,
    Low,
}

impl QualityPreset {
    pub fn to_settings(&self) -> transmute_compress::QualitySettings {
        match self {
            Self::Maximum => transmute_compress::QualitySettings::Maximum,
            Self::High => transmute_compress::QualitySettings::High,
            Self::Medium => transmute_compress::QualitySettings::Balanced,
            Self::Low => transmute_compress::QualitySettings::Low,
        }
    }
}

impl QualitySpec {
    pub fn to_settings(&self) -> transmute_compress::QualitySettings {
        match self {
            Self::Percentage(p) => transmute_compress::QualitySettings::Custom(*p),
            Self::Preset(preset) => preset.to_settings(),
            Self::Default => transmute_compress::QualitySettings::High,
        }
    }
}

/// Enhance/upscale image
#[derive(Debug, Clone, PartialEq)]
pub struct EnhanceIntent {
    pub input: PathBuf,
    pub scale_factor: u32, // 2 or 4
    pub output: Option<PathBuf>,
}

/// Batch process files
#[derive(Debug, Clone, PartialEq)]
pub struct BatchIntent {
    pub pattern: String, // e.g., "*.png" or "./photos/*"
    pub target_format: MediaFormat,
    pub output: Option<PathBuf>,
}

/// Combine multiple images into a single PDF
#[derive(Debug, Clone, PartialEq)]
pub struct CombineToPdfIntent {
    pub inputs: Vec<PathBuf>,
    pub output: PathBuf,
}

impl Intent {
    /// Get input path from any intent
    pub fn input_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Convert(i) => Some(&i.input),
            Self::Compress(i) => Some(&i.input),
            Self::Enhance(i) => Some(&i.input),
            Self::Batch(_) => None, // Batch uses pattern
            Self::CombineToPdf(i) => i.inputs.first(), // Return first input
        }
    }

    /// Get output path if specified
    pub fn output_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Convert(i) => i.output.as_ref(),
            Self::Compress(i) => i.output.as_ref(),
            Self::Enhance(i) => i.output.as_ref(),
            Self::Batch(i) => i.output.as_ref(),
            Self::CombineToPdf(i) => Some(&i.output),
        }
    }
}
