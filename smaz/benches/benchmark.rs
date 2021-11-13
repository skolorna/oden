use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smaz::compress;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("compress", |b| {
        b.iter(|| compress(black_box(b"Fisk Bj\xF6rkeby")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
