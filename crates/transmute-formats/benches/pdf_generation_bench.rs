// crates/transmute-formats/benches/pdf_generation_bench.rs
// Benchmark comparing old vs new PDF generation performance
// Tests across different image sizes and formats to measure optimization impact

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{DynamicImage, RgbImage};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use transmute_formats::{PdfGenerator, PdfOptions};

fn create_test_image(width: u32, height: u32) -> DynamicImage {
    // Create a gradient test image to simulate real content
    let mut img = RgbImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x as f32 / width as f32) * 255.0) as u8;
        let g = ((y as f32 / height as f32) * 255.0) as u8;
        let b = 128;
        *pixel = image::Rgb([r, g, b]);
    }
    DynamicImage::ImageRgb8(img)
}

fn bench_pdf_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_generation");

    // Test different image sizes
    let sizes = vec![
        ("1080p", 1920, 1080),
        ("4K", 3840, 2160),
        ("8K", 7680, 4320),
    ];

    for (name, width, height) in sizes {
        // Create test images (3 images per PDF to simulate batch operations)
        let images: Vec<(DynamicImage, PathBuf)> = (0..3)
            .map(|i| {
                let img = create_test_image(width, height);
                let path = PathBuf::from(format!("test_{}_{}.jpg", name, i));
                (img, path)
            })
            .collect();

        // Benchmark with default optimizations enabled
        group.bench_with_input(
            BenchmarkId::new("optimized", name),
            &images,
            |b, images| {
                b.iter(|| {
                    let temp_file = NamedTempFile::new().unwrap();
                    let generator = PdfGenerator::new(PdfOptions::default());
                    generator
                        .generate_from_images(black_box(images.clone()), temp_file.path())
                        .unwrap();
                });
            },
        );

        // Benchmark with optimizations disabled (old behavior)
        group.bench_with_input(
            BenchmarkId::new("unoptimized", name),
            &images,
            |b, images| {
                b.iter(|| {
                    let temp_file = NamedTempFile::new().unwrap();
                    let mut options = PdfOptions::default();
                    options.max_image_dimension = u32::MAX; // Disable downscaling
                    options.compress_images = false; // Use PNG (old behavior)
                    let generator = PdfGenerator::new(options);
                    generator
                        .generate_from_images(black_box(images.clone()), temp_file.path())
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_pdf_generation);
criterion_main!(benches);
