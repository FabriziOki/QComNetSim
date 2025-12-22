use qcomnetsim::network::{QuantumChannel, QuantumNode};
use qcomnetsim::protocols::barrett_kok::BarrettKokProtocol;
use qcomnetsim::simulation::{Event, EventScheduler, EventType};
use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;

fn main() {
    println!("QComNetSim - Barrett-Kok Protocol Comparison\n");

    // Parameters matching SeQUeNCe
    let distances = vec![1, 5, 10, 20, 50];
    let attenuation_db_per_km = 0.2;
    let coherence_time_ms = 100.0;
    let memory_size = 200; // SeQUeNCe uses 1 qubit/node
    let simulation_time_sec = 10.0;
    let generation_frequency_khz = 2.0; // 2 kHz from SeQUeNCe

    println!("=== Configuration ===");
    println!("Attenuation: {} dB/km", attenuation_db_per_km);
    println!("Coherence time: {} ms", coherence_time_ms);
    println!("Memory size: {} qubit/node", memory_size);
    println!("Generation frequency: {} kHz", generation_frequency_khz);
    println!("Simulation time: {} seconds", simulation_time_sec);
    println!();

    // Create CSV file
    fs::create_dir_all("data").unwrap();
    let mut csv = File::create("data/qcomnetsim_results.csv").unwrap();
    writeln!(csv, "distance_km,success_rate,throughput,runtime_ms").unwrap();

    let protocol = BarrettKokProtocol::sequence_parameters();

    // DEBUG: Print protocol config
    println!("DEBUG - Protocol config:");
    println!("  BSM efficiency: {}", protocol.bsm_efficiency);
    println!("  Detector efficiency: {}", protocol.detector_efficiency);
    println!("  Dark count rate: {}", protocol.dark_count_rate);
    println!("  Initial fidelity: {}", protocol.initial_fidelity);

    for &distance_km in &distances {
        println!("Running simulation for {} km...", distance_km);

        let start_time = Instant::now();

        let (successes, attempts) = run_simulation(
            distance_km as f64,
            attenuation_db_per_km,
            coherence_time_ms,
            memory_size,
            simulation_time_sec,
            generation_frequency_khz,
            &protocol,
        );

        let runtime_ms = start_time.elapsed().as_millis() as f64;

        let success_rate = if attempts > 0 {
            successes as f64 / attempts as f64
        } else {
            0.0
        };
        let throughput = successes as f64 / simulation_time_sec;

        writeln!(
            csv,
            "{},{:.4},{:.4},{:.4}",
            distance_km, success_rate, throughput, runtime_ms
        )
        .unwrap();

        println!("  Distance: {} km", distance_km);
        println!("  Attempts: {}", attempts);
        println!("  Successes: {}", successes);
        println!("  Success rate: {:.4}%", success_rate * 100.0);
        println!("  Throughput: {:.4} pair/sec", throughput);
        println!("  Runtime: {:.4} ms", runtime_ms);
        println!();
    }

    println!("Results saved to qcomnetsim_results.csv");
}

fn run_simulation(
    distance_km: f64,
    attenuation_db_per_km: f64,
    coherence_time_ms: f64,
    memory_size: usize,
    simulation_time_sec: f64,
    _generation_frequency_khz: f64, // Ignore this
    protocol: &BarrettKokProtocol,
) -> (usize, usize) {
    let mut node_a = QuantumNode::new(0, memory_size);
    let mut node_b = QuantumNode::new(1, memory_size);
    let channel = QuantumChannel::new(0, 1, distance_km, attenuation_db_per_km);

    let mut scheduler = EventScheduler::new();

    // Match SeQUeNCe: 100 attempts per distance
    let num_attempts = 100;
    let attempt_interval_ms = (simulation_time_sec * 1000.0) / num_attempts as f64;

    for i in 0..num_attempts {
        let time = i as f64 * attempt_interval_ms;
        scheduler.schedule(Event::new(time, EventType::EntanglementGeneration, 0));
    }

    // Run simulation
    let mut successes = 0;
    let mut attempts = 0;

    println!(
        "DEBUG - Total events scheduled: {}",
        scheduler.pending_events()
    );
    while let Some(event) = scheduler.next_event() {
        println!("DEBUG - Processing event at time {}", event.time);
        if event.event_type == EventType::EntanglementGeneration {
            attempts += 1;

            match protocol.attempt_generation(
                &mut node_a,
                &mut node_b,
                &channel,
                event.time,
                coherence_time_ms,
            ) {
                Ok(true) => {
                    successes += 1;
                }
                Ok(false) => {
                    // Channel or protocol failure
                }
                Err(_) => {
                    // Memory full - continue trying (SeQUeNCe behavior)
                }
            }
        }
    }

    (successes, attempts)
}
