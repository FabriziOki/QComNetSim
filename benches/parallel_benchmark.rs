use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::prelude::*;
use std::hint::black_box;
use QComNetSim::quantum::TwoQubitState;

fn benchmark_parallel_fidelity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parallel Operations");

    for size in [100, 1_000, 10_000].iter() {
        // Sequential
        group.bench_with_input(BenchmarkId::new("Sequential", size), size, |b, &size| {
            b.iter(|| {
                let bell = TwoQubitState::new_bell_phi_plus();
                let results: Vec<f64> = (0..size)
                    .map(|_| {
                        let other = TwoQubitState::new_bell_phi_plus();
                        bell.fidelity(&other)
                    })
                    .collect();
                black_box(results);
            });
        });

        // Parallel
        group.bench_with_input(BenchmarkId::new("Parallel", size), size, |b, &size| {
            b.iter(|| {
                let bell = TwoQubitState::new_bell_phi_plus();
                let results: Vec<f64> = (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let other = TwoQubitState::new_bell_phi_plus();
                        bell.fidelity(&other)
                    })
                    .collect();
                black_box(results);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_parallel_fidelity);
criterion_main!(benches);
