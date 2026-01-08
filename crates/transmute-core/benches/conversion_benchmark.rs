use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::DynamicImage;
use std::hint::black_box;
use transmute_common::MediaFormat;
use transmute_core::Converter;

fn benchmark_conversions(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let converter = Converter::new().unwrap();

    // Create test images of different sizes
    let sizes = vec![
        (512, 512, "512x512"),
        (1920, 1080, "1080p"),
        (3840, 2160, "4K"),
    ];

    for (width, height, label) in sizes {
        let input_path = temp_dir.path().join(format!("test_{}.png", label));
        let img = DynamicImage::new_rgb8(width, height);
        img.save(&input_path).unwrap();

        c.bench_with_input(
            BenchmarkId::new("PNG→JPEG", label),
            &input_path,
            |b, path| {
                b.iter(|| {
                    converter.convert_image(
                        black_box(path),
                        black_box(MediaFormat::Jpeg),
                        Some(temp_dir.path().to_path_buf()),
                    )
                });
            },
        );
    }
}

criterion_group!(benches, benchmark_conversions);
criterion_main!(benches);
