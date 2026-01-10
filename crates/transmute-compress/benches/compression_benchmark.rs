use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::DynamicImage;
use std::hint::black_box;
use transmute_common::{GpuContext, MediaFormat};
use transmute_compress::{ImageCompressor, QualitySettings};

fn benchmark_jpeg_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("JPEG_Compression");

    let sizes = vec![
        (512, 512, "512x512"),
        (1920, 1080, "1080p"),
        (3840, 2160, "4K"),
    ];

    for (width, height, label) in sizes {
        let img = DynamicImage::new_rgb8(width, height);

        // CPU compression
        group.bench_with_input(BenchmarkId::new("CPU", label), &img, |b, img| {
            let compressor = ImageCompressor::new(false).unwrap();
            b.iter(|| {
                compressor.compress(
                    black_box(img),
                    MediaFormat::Jpeg,
                    QualitySettings::High,
                    false,
                )
            });
        });

        // GPU compression (if available) - initialize compressor OUTSIDE iter loop
        if GpuContext::new().is_ok() {
            let compressor = ImageCompressor::new(true).unwrap();
            group.bench_with_input(BenchmarkId::new("GPU", label), &img, |b, img| {
                b.iter(|| {
                    compressor.compress(
                        black_box(img),
                        MediaFormat::Jpeg,
                        QualitySettings::High,
                        false,
                    )
                });
            });
        }
    }

    group.finish();
}

fn benchmark_png_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("PNG_Optimization");

    let levels = vec![
        (QualitySettings::Low, "Low"),
        (QualitySettings::Balanced, "Balanced"),
        (QualitySettings::Maximum, "Maximum"),
    ];

    let img = DynamicImage::new_rgb8(1920, 1080);

    for (quality, label) in levels {
        group.bench_with_input(BenchmarkId::new("1080p", label), &quality, |b, &quality| {
            let compressor = ImageCompressor::new(false).unwrap();
            b.iter(|| compressor.compress(black_box(&img), MediaFormat::Png, quality, false));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_jpeg_compression,
    benchmark_png_optimization
);
criterion_main!(benches);
