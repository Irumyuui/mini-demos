use criterion::{Criterion, black_box, criterion_group, criterion_main};
use simd_example::{binary_find_u8, linear_find_u8, simd_find_u8};

fn bench_linear(c: &mut Criterion) {
    let arr: Vec<u8> = (0..=u8::MAX).collect();
    c.bench_function("linear_find_u8", |b| {
        b.iter(|| {
            let _ = linear_find_u8(black_box(&arr), black_box(128));
        })
    });
}

fn bench_binary(c: &mut Criterion) {
    let arr: Vec<u8> = (0..=u8::MAX).collect();
    c.bench_function("binary_find_u8", |b| {
        b.iter(|| {
            let _ = binary_find_u8(black_box(&arr), black_box(128));
        })
    });
}

fn bench_simd(c: &mut Criterion) {
    let arr: Vec<u8> = (0..=u8::MAX).collect();
    c.bench_function("simd_find_u8", |b| {
        b.iter(|| {
            let _ = simd_find_u8(black_box(&arr), black_box(128));
        })
    });
}

criterion_group!(benches, bench_linear, bench_binary, bench_simd);
criterion_main!(benches);
