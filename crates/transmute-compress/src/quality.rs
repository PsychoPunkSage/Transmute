use image::DynamicImage;
use transmute_common::{Error, Result};

/// Quality assessment metrics
#[derive(Debug, Clone, Copy)]
pub struct QualityMetric {
    /// Structural Similarity Index (0.0-1.0, higher is better)
    pub ssim: f64,

    /// Peak Signal-to-Noise Ratio (dB, higher is better)
    pub psnr: f64,

    /// Mean Squared Error (lower is better)
    pub mse: f64,
}

impl QualityMetric {
    /// Calculate quality metrics between original and compressed
    pub fn calculate(original: &DynamicImage, compressed: &DynamicImage) -> Result<Self> {
        if original.dimensions() != compressed.dimensions() {
            return Err(Error::ConversionError(
                "Images must have same dimensions for quality comparison".into(),
            ));
        }

        let orig_rgb = original.to_rgb8();
        let comp_rgb = compressed.to_rgb8();

        let mse = Self::calculate_mse(&orig_rgb, &comp_rgb);
        let psnr = Self::calculate_psnr(mse);
        let ssim = Self::calculate_ssim(&orig_rgb, &comp_rgb);

        Ok(Self { ssim, psnr, mse })
    }

    /// Mean Squared Error between two images
    fn calculate_mse(img1: &image::RgbImage, img2: &image::RgbImage) -> f64 {
        let mut sum = 0.0;
        let pixel_count = (img1.width() * img1.height() * 3) as f64; // 3 = R/G/B

        for (p1, p2) in img1.pixels().zip(img2.pixels()) {
            for i in 0..3 {
                let diff = p1[i] as f64 - p2[i] as f64;
                sum += diff * diff;
            }
        }

        sum / pixel_count
    }

    /// Peak Signal-to-Noise Ratio in dB
    fn calculate_psnr(mse: f64) -> f64 {
        if mse == 0.0 {
            return f64::INFINITY;
        }
        let max_pixel = 255.0;
        20.0 * (max_pixel / mse.sqrt()).log10()
    }

    /// Simplified SSIM calculation (full SSIM requires windowing)
    fn calculate_ssim(img1: &image::RgbImage, img2: &image::RgbImage) -> f64 {
        // Constants for stability
        let c1 = (0.01 * 255.0).powi(2);
        let c2 = (0.03 * 255.0).powi(2);

        // Calculate means
        let mean1 = Self::mean_intensity(img1);
        let mean2 = Self::mean_intensity(img2);

        // Calculate variances and covariance
        let (var1, var2, covar) = Self::calculate_variances(img1, img2, mean1, mean2);

        // SSIM formula
        let numerator = (2.0 * mean1 * mean2 + c1) * (2.0 * covar + c2);
        let denominator = (mean1.powi(2) + mean2.powi(2) + c1) * (var1 + var2 + c2);

        numerator / denominator
    }

    fn mean_intensity(img: &image::RgbImage) -> f64 {
        let mut sum = 0.0;
        let pixel_count = (img.width() * img.height() * 3) as f64;

        for pixel in img.pixels() {
            for &channel in pixel.0.iter() {
                sum += channel as f64;
            }
        }

        sum / pixel_count
    }

    fn calculate_variances(
        img1: &image::RgbImage,
        img2: &image::RgbImage,
        mean1: f64,
        mean2: f64,
    ) -> (f64, f64, f64) {
        let mut var1 = 0.0;
        let mut var2 = 0.0;
        let mut covar = 0.0;
        let pixel_count = (img1.width() * img1.height() * 3) as f64;

        for (p1, p2) in img1.pixels().zip(img2.pixels()) {
            for i in 0..3 {
                let v1 = p1[i] as f64 - mean1;
                let v2 = p2[i] as f64 - mean2;
                var1 += v1 * v1;
                var2 += v2 * v2;
                covar += v1 * v2;
            }
        }

        (var1 / pixel_count, var2 / pixel_count, covar / pixel_count)
    }

    /// Check if quality meets minimum threshold
    pub fn meets_threshold(&self, min_ssim: f64) -> bool {
        self.ssim >= min_ssim
    }
}

/// Compression quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualitySettings {
    /// Maximum quality (SSIM > 0.98, JPEG ~98)
    Maximum,

    /// High quality (SSIM > 0.95, JPEG ~95)
    High,

    /// Balanced quality/size (SSIM > 0.90, JPEG ~85)
    Balanced,

    /// Lower quality, smaller size (SSIM > 0.85, JPEG ~75)
    Low,

    /// Custom quality value (0-100 for JPEG)
    Custom(u8),
}

impl QualitySettings {
    /// Get JPEG quality value
    pub fn jpeg_quality(&self) -> u8 {
        match self {
            Self::Maximum => 98,
            Self::High => 95,
            Self::Balanced => 85,
            Self::Low => 75,
            Self::Custom(q) => (*q).clamp(1, 100),
        }
    }

    /// Get PNG compression level (0-6, oxipng)
    pub fn png_level(&self) -> u8 {
        match self {
            Self::Maximum => 6,
            Self::High => 5,
            Self::Balanced => 3,
            Self::Low => 2,
            Self::Custom(q) => ((*q as f32 / 100.0) * 6.0) as u8,
        }
    }

    /// Get WebP quality (0-100)
    pub fn webp_quality(&self) -> f32 {
        match self {
            Self::Maximum => 98.0,
            Self::High => 90.0,
            Self::Balanced => 80.0,
            Self::Low => 70.0,
            Self::Custom(q) => *q as f32,
        }
    }

    /// Get target SSIM threshold
    pub fn target_ssim(&self) -> f64 {
        match self {
            Self::Maximum => 0.98,
            Self::High => 0.95,
            Self::Balanced => 0.90,
            Self::Low => 0.85,
            Self::Custom(_) => 0.85, // Minimum acceptable
        }
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self::High
    }
}
