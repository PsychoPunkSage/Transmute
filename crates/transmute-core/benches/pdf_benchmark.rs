use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::DynamicImage;
use std::hint::black_box;
use tempfile::TempDir;
use transmute_core::Converter;
use transmute_formats::PdfOptions;

fn benchmark_pdf_generation(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let converter = Converter::new().unwrap();

    let page_counts = vec![1, 10, 50, 100];

    for page_count in page_counts {
        // Create test images
        let mut images = Vec::new();
        for i in 0..page_count {
            let path = temp_dir.path().join(format!("page{}.png", i));
            let img = DynamicImage::new_rgb8(1920, 1080);
            img.save(&path).unwrap();
            images.push(path);
        }

        c.bench_with_input(
            BenchmarkId::new("PDF_Generation", page_count),
            &images,
            |b, imgs| {
                b.iter(|| {
                    let output = temp_dir.path().join("output.pdf");
                    converter.images_to_pdf(
                        black_box(imgs.clone()),
                        black_box(output),
                        Some(PdfOptions::default()),
                    )
                });
            },
        );
    }
}

criterion_group! {
    name = pdf_benches;
    config = Criterion::default().sample_size(10); // PDF ops are slow
    targets = benchmark_pdf_generation
}
criterion_main!(pdf_benches);
