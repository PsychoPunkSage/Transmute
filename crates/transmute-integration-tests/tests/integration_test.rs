use image::DynamicImage;
use tempfile::TempDir;
use transmute_common::MediaFormat;
use transmute_core::{batch::BatchJob, BatchProcessor, Converter};
use transmute_formats::PdfOptions;

/// Check if pdfium library is available on the system
fn is_pdfium_available() -> bool {
    use std::path::PathBuf;

    // Common library search paths
    let search_paths = [
        "/usr/lib/libpdfium.so",
        "/usr/local/lib/libpdfium.so",
        "/usr/lib/x86_64-linux-gnu/libpdfium.so",
    ];

    search_paths.iter().any(|p| PathBuf::from(p).exists())
        || std::env::var("LD_LIBRARY_PATH")
            .ok()
            .and_then(|paths| {
                paths.split(':')
                    .map(|p| PathBuf::from(p).join("libpdfium.so"))
                    .find(|p| p.exists())
            })
            .is_some()
}

#[test]
fn test_end_to_end_pdf_workflow() {
    if !is_pdfium_available() {
        eprintln!("Skipping test_end_to_end_pdf_workflow: libpdfium.so not available");
        eprintln!("Install libpdfium or set LD_LIBRARY_PATH to enable PDF tests");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let converter = Converter::new().unwrap();

    // Step 1: Create test images
    let mut image_paths = Vec::new();
    for i in 0..5 {
        let path = temp_dir.path().join(format!("input_{}.png", i));
        let img = DynamicImage::new_rgb8(1920, 1080);
        img.save(&path).unwrap();
        image_paths.push(path);
    }

    // Step 2: Convert to PDF
    let pdf_path = temp_dir.path().join("combined.pdf");
    let pdf_result = converter.images_to_pdf(
        image_paths.clone(),
        pdf_path.clone(),
        Some(PdfOptions {
            dpi: 150.0,
            ..Default::default()
        }),
    );
    assert!(pdf_result.is_ok());
    assert!(pdf_path.exists());

    // Step 3: Extract back to images
    let extracted = converter.pdf_to_images(
        &pdf_path,
        MediaFormat::Jpeg,
        Some(temp_dir.path().to_path_buf()),
        Some(150.0),
    );
    assert!(extracted.is_ok());
    assert_eq!(extracted.unwrap().len(), 5);
}

#[tokio::test]
async fn test_large_batch_processing() {
    let temp_dir = TempDir::new().unwrap();

    // Create 100 test images
    let mut jobs = Vec::new();
    for i in 0..100 {
        let input_path = temp_dir.path().join(format!("img_{:03}.png", i));
        let img = DynamicImage::new_rgb8(640, 480);
        img.save(&input_path).unwrap();

        jobs.push(BatchJob {
            input: input_path,
            output_format: MediaFormat::Jpeg,
            output_path: Some(temp_dir.path().to_path_buf()),
        });
    }

    let processor = BatchProcessor::new(8); // 8 concurrent
    let results = processor.process_batch_sync(jobs).await.unwrap();

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 100);
}

#[test]
fn test_memory_preservation() {
    if !is_pdfium_available() {
        eprintln!("Skipping test_memory_preservation: libpdfium.so not available");
        eprintln!("Install libpdfium or set LD_LIBRARY_PATH to enable PDF tests");
        return;
    }

    // Verify metadata preserved through PDF round-trip
    let temp_dir = TempDir::new().unwrap();
    let converter = Converter::new().unwrap();

    let original = DynamicImage::new_rgb8(800, 600);
    let input_path = temp_dir.path().join("original.png");
    original.save(&input_path).unwrap();

    let pdf_path = temp_dir.path().join("test.pdf");
    converter
        .images_to_pdf(vec![input_path], pdf_path.clone(), None)
        .unwrap();

    let extracted = converter
        .pdf_to_images(
            &pdf_path,
            MediaFormat::Png,
            Some(temp_dir.path().to_path_buf()),
            Some(300.0),
        )
        .unwrap();

    assert_eq!(extracted.len(), 1);

    let reconstructed = image::open(&extracted[0]).unwrap();
    // Allow ±10% dimension variance due to PDF rasterization
    assert!((reconstructed.width() as i32 - 800).abs() < 80);
    assert!((reconstructed.height() as i32 - 600).abs() < 60);
}
