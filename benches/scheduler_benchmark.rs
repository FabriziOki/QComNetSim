use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use QComNetSim::simulation::{Event, EventScheduler, EventType};

fn benchmark_event_scheduling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Event Scheduling");

    // Test different event counts
    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new("Insert", size), size, |b, &size| {
            b.iter(|| {
                let mut scheduler = EventScheduler::new();
                for i in 0..size {
                    let event = Event::new(
                        (i as f64) * 0.001,
                        EventType::EntanglementGeneration,
                        i % 10,
                    );
                    scheduler.schedule(black_box(event));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("Insert+Remove", size), size, |b, &size| {
            b.iter(|| {
                let mut scheduler = EventScheduler::new();

                for i in 0..size {
                    let event = Event::new(
                        (i as f64) * 0.001,
                        EventType::EntanglementGeneration,
                        i % 10,
                    );
                    scheduler.schedule(event);
                }

                while scheduler.has_events() {
                    black_box(scheduler.next_event());
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_event_scheduling);
criterion_main!(benches);
