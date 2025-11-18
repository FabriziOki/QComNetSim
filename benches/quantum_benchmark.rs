use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use QComNetSim::quantum::gates::{hadamard, pauli_x, pauli_y, pauli_z};
use QComNetSim::quantum::{Qubit, TwoQubitState};

fn benchmark_single_qubit_gates(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Qubit Gates");

    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("Pauli-X", size), size, |b, &size| {
            b.iter(|| {
                let mut qubit = Qubit::new_zero();
                for _ in 0..size {
                    pauli_x(&mut qubit);
                }
                black_box(qubit);
            });
        });

        group.bench_with_input(BenchmarkId::new("Hadamard", size), size, |b, &size| {
            b.iter(|| {
                let mut qubit = Qubit::new_zero();
                for _ in 0..size {
                    hadamard(&mut qubit);
                }
                black_box(qubit);
            });
        });
    }

    group.finish();
}

fn benchmark_fidelity_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fidelity Calculation");

    let bell1 = TwoQubitState::new_bell_phi_plus();
    let bell2 = TwoQubitState::new_bell_phi_plus();

    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("Fidelity", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    black_box(bell1.fidelity(&bell2));
                }
            });
        });
    }

    group.finish();
}

fn benchmark_state_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("State Creation");

    for size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("Bell State", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    black_box(TwoQubitState::new_bell_phi_plus());
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_qubit_gates,
    benchmark_fidelity_calculation,
    benchmark_state_creation
);
criterion_main!(benches);
