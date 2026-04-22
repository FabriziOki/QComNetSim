/// End-to-end demonstration of a 3-node quantum repeater chain.
///
/// Topology:  Alice (0) ──── Bob (1) ──── Carol (2)
///            10 km, 0.2 dB/km per link
///
/// Protocol:
///   1. Barrett-Kok heralded entanglement generation on each link
///   2. Entanglement swapping at Bob to produce Alice–Carol end-to-end pairs
///
/// Run with:  cargo run --example three_node_chain
use qcomnetsim::network::{find_shortest_path, NetworkTopology};
use qcomnetsim::protocols::barrett_kok::BarrettKokProtocol;
use qcomnetsim::protocols::swapping::EntanglementSwappingProtocol;
use qcomnetsim::simulation::{Simulation, SimulationConfig};
use qcomnetsim::PhysicsProfile;

fn main() {
    println!("QComNetSim — 3-node linear repeater chain\n");

    // ── Physics profile ──────────────────────────────────────────
    let profile = PhysicsProfile::erbium();
    println!("Platform:   {}", profile.name);
    println!("Coherence:  {} ms", profile.coherence_time_ms);

    // ── Topology ────────────────────────────────────────────────
    let distance_km = 10.0;
    let attenuation = 0.2;
    let memory_slots = 8;

    let topology = NetworkTopology::new_linear(3, memory_slots, distance_km, attenuation);

    println!("Topology:   linear, 3 nodes");
    println!("Link:       {distance_km} km, {attenuation} dB/km each");
    println!("Memory:     {memory_slots} slots/node");

    // ── Routing ─────────────────────────────────────────────────
    let source = 0; // Alice
    let dest = 2;   // Carol

    let path = find_shortest_path(&topology, source, dest)
        .expect("No path found between Alice and Carol");

    let repeaters: Vec<usize> = path[1..path.len() - 1].to_vec();
    println!("Path:       {:?}  (repeaters: {:?})", path, repeaters);

    // ── Protocols (from profile) ─────────────────────────────────
    let gen_protocol = BarrettKokProtocol::from_profile(&profile);
    let swap_protocol = EntanglementSwappingProtocol::from_profile(&profile);

    println!(
        "Generation: Barrett-Kok  (η_mem={:.2}, η_det={:.2}, p_dark={:.2e})",
        gen_protocol.memory_efficiency,
        gen_protocol.detector_efficiency,
        gen_protocol.dark_count_rate,
    );
    println!(
        "Swapping:   BSM  (η_bsm={:.2}, gate_f={:.3})",
        swap_protocol.bsm_efficiency,
        swap_protocol.gate_fidelity,
    );

    // ── Simulation config (coherence from profile) ───────────────
    let config = SimulationConfig {
        target_pairs: 100,
        max_time_ms: 500_000.0,
        ..SimulationConfig::from_profile(&profile)
    };

    println!(
        "\nTarget:     {} end-to-end pairs",
        config.target_pairs
    );
    println!(
        "Coherence:  {} ms  |  max sim time: {} ms\n",
        config.coherence_time_ms, config.max_time_ms
    );

    // ── Run ──────────────────────────────────────────────────────
    let mut sim = Simulation::new(topology, gen_protocol, swap_protocol, config, path);
    sim.run();

    // ── Results ──────────────────────────────────────────────────
    sim.stats.print_summary();

    if let (Some(mean), Some(std)) = (
        sim.stats.mean_fidelity(),
        sim.stats.fidelity_std_dev(),
    ) {
        let min = sim
            .stats
            .delivered_fidelities
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max = sim
            .stats
            .delivered_fidelities
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        println!("Fidelity range:  [{min:.4}, {max:.4}]");
        println!("Mean ± 1σ:       {mean:.4} ± {std:.4}");
    }

    if let Some(t) = sim.stats.mean_delivery_time_ms() {
        let throughput = sim.stats.pairs_delivered as f64 / (t / 1000.0).max(1e-9);
        println!("Throughput:      {throughput:.2} pairs/s  (approx)");
    }
}
