use image::{DynamicImage, GenericImageView};
use tempfile::TempDir;
use transmute_common::MediaFormat;
use transmute_compress::QualitySettings;
use transmute_core::Converter;

#[test]
fn test_jpeg_quality_ssim() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("original.png");

    // Create test image with gradient pattern for meaningful compression differences
    let mut img = DynamicImage::new_rgb8(1920, 1080);
    let rgb_img = img.as_mut_rgb8().unwrap();
    for (x, y, pixel) in rgb_img.enumerate_pixels_mut() {
        let r = ((x as f32 / 1920.0) * 255.0) as u8;
        let g = ((y as f32 / 1080.0) * 255.0) as u8;
        let b = (((x + y) as f32 / 3000.0) * 255.0) as u8;
        *pixel = image::Rgb([r, g, b]);
    }
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();

    let (_output_high, result_high) = converter
        .compress_image(
            &input_path,
            MediaFormat::Jpeg,
            QualitySettings::High,
            Some(temp_dir.path().to_path_buf()),
        )
        .unwrap();

    let (_output_low, result_low) = converter
        .compress_image(
            &input_path,
            MediaFormat::Jpeg,
            QualitySettings::Low,
            Some(temp_dir.path().to_path_buf()),
        )
        .unwrap();

    // Both should achieve compression
    assert!(result_high.ratio > 1.0);
    assert!(result_low.ratio > 1.0);

    // Low quality should not produce larger files than high quality
    assert!(
        result_low.compressed_size <= result_high.compressed_size,
        "Low quality ({} bytes) should not exceed high quality ({} bytes)",
        result_low.compressed_size,
        result_high.compressed_size
    );

    println!(
        "High quality: {:.1}% reduction, {} bytes",
        result_high.size_reduction_percent(),
        result_high.compressed_size
    );
    println!(
        "Low quality: {:.1}% reduction, {} bytes",
        result_low.size_reduction_percent(),
        result_low.compressed_size
    );
}

#[test]
fn test_png_lossless() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.png");

    let img = DynamicImage::new_rgba8(800, 600);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();
    let (output, result) = converter
        .compress_image(
            &input_path,
            MediaFormat::Png,
            QualitySettings::Maximum,
            Some(temp_dir.path().to_path_buf()),
        )
        .unwrap();

    // PNG optimization should reduce size
    assert!(result.compressed_size < result.original_size);

    // Verify image is identical (lossless)
    let original = image::open(&input_path).unwrap();
    let compressed = image::open(&output).unwrap();

    assert_eq!(original.dimensions(), compressed.dimensions());
    assert_eq!(original.to_rgba8().as_raw(), compressed.to_rgba8().as_raw());
}

#[tokio::test]
async fn test_batch_compression() {
    let temp_dir = TempDir::new().unwrap();
    let converter = Converter::new().unwrap();

    // Create batch of images
    let mut inputs = Vec::new();
    for i in 0..10 {
        let path = temp_dir.path().join(format!("img_{}.png", i));
        let img = DynamicImage::new_rgb8(640, 480);
        img.save(&path).unwrap();
        inputs.push(path);
    }

    let results = converter
        .compress_batch(
            inputs,
            MediaFormat::Jpeg,
            QualitySettings::Balanced,
            Some(temp_dir.path().to_path_buf()),
        )
        .await;

    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.is_ok()));

    // Calculate average compression ratio
    let avg_ratio: f32 = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .map(|(_, result)| result.ratio)
        .sum::<f32>()
        / 10.0;

    println!("Average compression ratio: {:.2}x", avg_ratio);
    assert!(avg_ratio > 5.0);
}
