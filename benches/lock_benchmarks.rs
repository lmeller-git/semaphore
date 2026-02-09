use criterion::{Criterion, criterion_group, criterion_main};

fn bencher(c: &mut Criterion) {
    c.bench_function("test", |b| b.iter(|| 42 + 42));
}

criterion_group!(benches, bencher);
criterion_main!(benches);
