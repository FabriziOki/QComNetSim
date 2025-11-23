use qcomnetsim::network::{
    attempt_entanglement_generation, GenerationStats, QuantumChannel, QuantumNode,
};
use qcomnetsim::simulation::{Event, EventScheduler, EventType};

fn main() {
    println!("QComNetSim - 2-Node Entanglement Generation Demo\n");

    // Parameters
    let distance_km = 5.0;
    let attenuation_db_per_km = 0.2;
    let coherence_time_ms = 100.0;
    let num_attempts = 100;
    let attempt_interval_ms = 1.0; // Try every 1ms

    println!("=== Configuration ===");
    println!("Distance: {} km", distance_km);
    println!("Attenuation: {} dB/km", attenuation_db_per_km);
    println!("Coherence time: {} ms", coherence_time_ms);
    println!("Attempts: {}", num_attempts);
    println!();

    // Create nodes
    let mut node_a = QuantumNode::new(0, 50);
    let mut node_b = QuantumNode::new(1, 50);

    // Create channel
    let channel = QuantumChannel::new(0, 1, distance_km, attenuation_db_per_km);

    println!(
        "Channel success probability: {:.1}%",
        channel.success_probability() * 100.0
    );
    println!();

    // Create event scheduler
    let mut scheduler = EventScheduler::new();

    // Schedule entanglement generation attempts
    for i in 0..num_attempts {
        let time = i as f64 * attempt_interval_ms;
        scheduler.schedule(Event::new(
            time,
            EventType::EntanglementGeneration,
            0, // node_id (not used here)
        ));
    }

    // Run simulation
    let mut stats = GenerationStats::new();

    println!("=== Running Simulation ===");
    while let Some(event) = scheduler.next_event() {
        if event.event_type == EventType::EntanglementGeneration {
            stats.attempts += 1;

            match attempt_entanglement_generation(
                &mut node_a,
                &mut node_b,
                &channel,
                event.time,
                coherence_time_ms,
            ) {
                Ok(true) => {
                    stats.successes += 1;
                    println!(
                        "[{:.1}ms] ✓ Entanglement generated (attempt #{})",
                        event.time, stats.attempts
                    );
                }
                Ok(false) => {
                    stats.channel_failures += 1;
                    println!(
                        "[{:.1}ms] ✗ Channel failure (attempt #{})",
                        event.time, stats.attempts
                    );
                }
                Err(e) => {
                    stats.memory_full_errors += 1;
                    println!(
                        "[{:.1}ms] ⚠ Memory full: {} (attempt #{})",
                        event.time, e, stats.attempts
                    );
                }
            }
        }
    }

    // Print results
    stats.print_summary();

    println!("=== Final State ===");
    println!(
        "Node A memory: {}/{} used",
        node_a.num_stored_pairs(),
        node_a.memory_capacity
    );
    println!(
        "Node B memory: {}/{} used",
        node_b.num_stored_pairs(),
        node_b.memory_capacity
    );

    // Check fidelity of stored pairs
    if node_a.num_stored_pairs() > 0 {
        println!("\n=== Checking Fidelity After Storage ===");
        let final_time = num_attempts as f64 * attempt_interval_ms;

        // Get average fidelity
        let mut total_fidelity = 0.0;
        for pair in &node_a.stored_pairs {
            let mut pair_copy = pair.clone();
            pair_copy.update_fidelity(final_time);
            total_fidelity += pair_copy.fidelity;
        }

        let avg_fidelity = total_fidelity / node_a.num_stored_pairs() as f64;
        println!("Average fidelity: {:.4}", avg_fidelity);

        if avg_fidelity < 0.9 {
            println!("⚠ Warning: Average fidelity below threshold!");
        }
    }
}
