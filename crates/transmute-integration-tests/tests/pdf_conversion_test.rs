// Integration test for PDF conversion functionality
use image::DynamicImage;
use std::path::PathBuf;
use tempfile::TempDir;
use transmute_common::MediaFormat;
use transmute_core::Converter;

#[test]
fn test_image_to_pdf_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.png");

    // Create test image
    let img = DynamicImage::new_rgb8(800, 600);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();
    let output_path = converter
        .convert_image(&input_path, MediaFormat::Pdf, None)
        .expect("Failed to convert image to PDF");

    assert!(output_path.exists());
    assert_eq!(output_path.extension().unwrap(), "pdf");
}

#[test]
fn test_image_to_pdf_with_custom_output() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.jpg");
    let output_path = temp_dir.path().join("custom_output.pdf");

    // Create test image
    let img = DynamicImage::new_rgb8(1024, 768);
    img.save(&input_path).unwrap();

    let converter = Converter::new().unwrap();
    let result = converter
        .convert_image(&input_path, MediaFormat::Pdf, Some(output_path.clone()))
        .expect("Failed to convert image to PDF");

    assert_eq!(result, output_path);
    assert!(output_path.exists());
}

#[test]
fn test_multiple_images_to_pdf() {
    let temp_dir = TempDir::new().unwrap();
    let mut inputs = Vec::new();

    // Create 3 test images
    for i in 0..3 {
        let path = temp_dir.path().join(format!("page{}.png", i));
        let img = DynamicImage::new_rgb8(800, 600);
        img.save(&path).unwrap();
        inputs.push(path);
    }

    let output = temp_dir.path().join("multi_page.pdf");
    let converter = Converter::new().unwrap();
    let result = converter
        .images_to_pdf(inputs, output.clone(), None)
        .expect("Failed to create multi-page PDF");

    assert_eq!(result, output);
    assert!(output.exists());
}
