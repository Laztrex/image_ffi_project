use blur_plugin::blur_pass;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_blur_pass(c: &mut Criterion) {
    let width = 1920;
    let height = 1080;
    let len = width * height * 4;
    let src = vec![128u8; len];
    let mut dst = vec![0u8; len];
    let radius = 3;

    c.bench_function("blur_pass 1080p radius3", |b| {
        b.iter(|| {
            blur_pass(
                black_box(&src),
                black_box(&mut dst),
                black_box(width),
                black_box(height),
                black_box(radius),
            );
        })
    });
}

criterion_group!(benches, bench_blur_pass);
criterion_main!(benches);
