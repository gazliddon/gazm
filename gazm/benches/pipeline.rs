use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gazm::{
    assembler::Assembler,
    opts::{BuildType, Opts},
};

fn check_fixture(c: &mut Criterion) {
    let mut opts = Opts::default();
    opts.project_file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/test_src/newfe.gazm");
    opts.build_type = BuildType::Check;
    opts.no_async = true;

    c.bench_function("assemble_newfe", |b| {
        b.iter(|| {
            let mut assembler = Assembler::new(opts.clone());
            black_box(assembler.assemble().is_ok());
        });
    });

    let mut cached = Assembler::new(opts);
    let _ = cached.assemble();
    c.bench_function("reassemble_newfe_cached", |b| {
        b.iter(|| black_box(cached.reassemble().is_ok()));
    });
}

criterion_group!(benches, check_fixture);
criterion_main!(benches);
