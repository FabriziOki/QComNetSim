/// Cross-platform fidelity comparison on a 3-node repeater chain.
///
/// Runs the same A–B–C topology under Erbium-167 and NV-center physics
/// profiles and compares end-to-end entanglement fidelity and throughput.
/// This directly addresses reviewer criticism #3 (physics locked to Erbium).
///
/// Run with:  cargo run --example platform_comparison
use qcomnetsim::network::{find_shortest_path, NetworkTopology};
use qcomnetsim::protocols::barrett_kok::BarrettKokProtocol;
use qcomnetsim::protocols::swapping::EntanglementSwappingProtocol;
use qcomnetsim::simulation::{Simulation, SimulationConfig, SimulationStats};
use qcomnetsim::PhysicsProfile;

const DISTANCE_KM: f64 = 10.0;
const ATTENUATION: f64 = 0.2;
const MEMORY_SLOTS: usize = 8;
const TARGET_PAIRS: usize = 100;
const MAX_TIME_MS: f64 = 10_000_000.0;

fn run_profile(profile: &PhysicsProfile) -> SimulationStats {
    let topology =
        NetworkTopology::new_linear(3, MEMORY_SLOTS, DISTANCE_KM, ATTENUATION);
    let path = find_shortest_path(&topology, 0, 2).expect("path not found");

    let mut sim = Simulation::new(
        topology,
        BarrettKokProtocol::from_profile(profile),
        EntanglementSwappingProtocol::from_profile(profile),
        SimulationConfig {
            target_pairs: TARGET_PAIRS,
            max_time_ms: MAX_TIME_MS,
            ..SimulationConfig::from_profile(profile)
        },
        path,
    );
    sim.run();
    sim.stats
}

fn print_result(profile: &PhysicsProfile, stats: &SimulationStats) {
    let mean_f = stats.mean_fidelity().unwrap_or(0.0);
    let std_f = stats.fidelity_std_dev().unwrap_or(0.0);
    let mean_t = stats.mean_delivery_time_ms().unwrap_or(0.0);
    let throughput = if mean_t > 0.0 {
        stats.pairs_delivered as f64 / (mean_t / 1000.0)
    } else {
        0.0
    };

    println!("Platform:          {}", profile.name);
    println!(
        "  Coherence time:  {:.1} ms",
        profile.coherence_time_ms
    );
    println!(
        "  Memory eff.:     {:.0}%   Detector eff.: {:.0}%",
        profile.memory_efficiency * 100.0,
        profile.detector_efficiency * 100.0
    );
    println!(
        "  Gate fidelity:   {:.3}   BSM eff. (swap): {:.2}",
        profile.gate_fidelity, profile.swap_bsm_efficiency
    );
    println!("  Delivered pairs: {}", stats.pairs_delivered);
    println!(
        "  Gen success:     {}/{} ({:.2}%)",
        stats.generation_successes,
        stats.generation_attempts,
        stats.generation_success_rate() * 100.0
    );
    println!(
        "  Swap success:    {}/{} ({:.1}%)",
        stats.swap_successes,
        stats.swap_attempts,
        stats.swap_success_rate() * 100.0
    );
    println!("  Fidelity:        {:.4} ± {:.4} (1σ)", mean_f, std_f);
    println!("  Mean delivery:   {:.1} ms", mean_t);
    println!("  Throughput:      {:.2} pairs/s  (approx)", throughput);
}

fn main() {
    println!("QComNetSim — Platform Comparison (3-node linear chain)\n");
    println!(
        "Scenario: {DISTANCE_KM} km links, {ATTENUATION} dB/km, \
         {MEMORY_SLOTS} memory slots/node, {TARGET_PAIRS} target pairs\n"
    );
    println!("{}", "=".repeat(55));

    let profiles = [PhysicsProfile::erbium(), PhysicsProfile::nv_center()];

    let mut results: Vec<(&PhysicsProfile, SimulationStats)> = Vec::new();
    for profile in &profiles {
        let stats = run_profile(profile);
        print_result(profile, &stats);
        println!("{}", "-".repeat(55));
        results.push((profile, stats));
    }

    // ── Summary table ────────────────────────────────────────────
    println!("\n{:<20} {:>12} {:>12} {:>12}", "Platform", "Fidelity", "±1σ", "Throughput");
    println!("{}", "-".repeat(58));
    for (profile, stats) in &results {
        let mean_f = stats.mean_fidelity().unwrap_or(0.0);
        let std_f = stats.fidelity_std_dev().unwrap_or(0.0);
        let mean_t = stats.mean_delivery_time_ms().unwrap_or(1.0);
        let tp = stats.pairs_delivered as f64 / (mean_t / 1000.0).max(1e-9);
        println!(
            "{:<20} {:>12.4} {:>12.4} {:>11.2}/s",
            profile.name, mean_f, std_f, tp
        );
    }
    println!();
}
