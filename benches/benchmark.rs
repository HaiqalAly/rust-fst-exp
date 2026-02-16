use criterion::{criterion_group, criterion_main, Criterion};
use rust_exp_fst::finite_state::search::Dictionary;

fn benchmark_search(c: &mut Criterion) {
    // If dict.fst doesn't exist, we can't run the benchmark.
    if !std::path::Path::new("dict.fst").exists() {
        eprintln!("dict.fst not found. Please run 'cargo run' first to generate it.");
        return;
    }

    let dict = Dictionary::new("dict.fst").expect("dict.fst failed to load");
    
    let mut group = c.benchmark_group("search");
    
    // Benchmark exact match
    group.bench_function("search_exact_apple", |b| {
        b.iter(|| dict.search("apple"))
    });

    // Benchmark fuzzy match (short)
    group.bench_function("search_fuzzy_aple", |b| {
        b.iter(|| dict.search("aple"))
    });

    // Benchmark fuzzy match (longer word with typo)
    group.bench_function("search_fuzzy_interational", |b| {
        b.iter(|| dict.search("interational")) // typo of international
    });

    group.finish();
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
