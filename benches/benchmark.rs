use std::{fs::File, path::{Path, PathBuf}};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn render() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("benches/sample.md");

    print!("{:?}", d);
    let _ = markterm::render_file_to_stdout(&d, None);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(render));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
