use crate::network::topology::NetworkTopology;
use crate::physics::PhysicsProfile;
use crate::protocols::barrett_kok::BarrettKokProtocol;
use crate::protocols::swapping::EntanglementSwappingProtocol;
use super::event::{Event, EventType};
use super::scheduler::EventScheduler;

// ============================================================
// CONFIG
// ============================================================

/// Parameters that drive a single simulation run
pub struct SimulationConfig {
    /// Qubit memory coherence time (ms)
    pub coherence_time_ms: f64,
    /// Time between entanglement generation retries (ms)
    pub retry_interval_ms: f64,
    /// Classical communication delay before a swap event fires (ms).
    /// Models the time for the BSM-herald to reach the endpoints.
    pub classical_delay_ms: f64,
    /// Minimum fidelity for a pair to remain in memory
    pub fidelity_threshold: f64,
    /// Stop after delivering this many end-to-end pairs
    pub target_pairs: usize,
    /// Hard simulation time cutoff (ms)
    pub max_time_ms: f64,
    /// How often to run the decoherence sweep (ms)
    pub decoherence_check_interval_ms: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            coherence_time_ms: 1_000.0,
            retry_interval_ms: 0.1,
            classical_delay_ms: 0.05,
            fidelity_threshold: 0.5,
            target_pairs: 100,
            max_time_ms: 100_000.0,
            decoherence_check_interval_ms: 10.0,
        }
    }
}

impl SimulationConfig {
    /// Build a config seeded from a `PhysicsProfile`, with all other
    /// parameters left at their defaults.
    pub fn from_profile(profile: &PhysicsProfile) -> Self {
        SimulationConfig {
            coherence_time_ms: profile.coherence_time_ms,
            ..Default::default()
        }
    }
}

// ============================================================
// STATS
// ============================================================

/// Counters and distributions collected during a run
#[derive(Default, Debug, Clone)]
pub struct SimulationStats {
    pub generation_attempts: usize,
    pub generation_successes: usize,
    pub swap_attempts: usize,
    pub swap_successes: usize,
    /// Pairs removed from memory by the decoherence sweep
    pub decoherence_removals: usize,
    /// End-to-end pairs successfully delivered to source and destination
    pub pairs_delivered: usize,
    /// Fidelity of each delivered pair, in delivery order
    pub delivered_fidelities: Vec<f64>,
    /// Simulation time (ms) at which each pair was delivered
    pub delivery_times_ms: Vec<f64>,
}

impl SimulationStats {
    pub fn generation_success_rate(&self) -> f64 {
        if self.generation_attempts == 0 {
            0.0
        } else {
            self.generation_successes as f64 / self.generation_attempts as f64
        }
    }

    pub fn swap_success_rate(&self) -> f64 {
        if self.swap_attempts == 0 {
            0.0
        } else {
            self.swap_successes as f64 / self.swap_attempts as f64
        }
    }

    pub fn mean_fidelity(&self) -> Option<f64> {
        if self.delivered_fidelities.is_empty() {
            return None;
        }
        Some(self.delivered_fidelities.iter().sum::<f64>() / self.delivered_fidelities.len() as f64)
    }

    /// Sample standard deviation of delivered pair fidelities
    pub fn fidelity_std_dev(&self) -> Option<f64> {
        let mean = self.mean_fidelity()?;
        let n = self.delivered_fidelities.len();
        if n < 2 {
            return Some(0.0);
        }
        let variance = self.delivered_fidelities.iter()
            .map(|f| (f - mean).powi(2))
            .sum::<f64>()
            / (n - 1) as f64; // Bessel's correction
        Some(variance.sqrt())
    }

    pub fn mean_delivery_time_ms(&self) -> Option<f64> {
        if self.delivery_times_ms.is_empty() {
            return None;
        }
        Some(self.delivery_times_ms.iter().sum::<f64>() / self.delivery_times_ms.len() as f64)
    }

    pub fn print_summary(&self) {
        println!("\n=== Simulation Statistics ===");
        println!(
            "Generation:  {} attempts, {} successes ({:.1}%)",
            self.generation_attempts,
            self.generation_successes,
            self.generation_success_rate() * 100.0
        );
        println!(
            "Swaps:       {} attempts, {} successes ({:.1}%)",
            self.swap_attempts,
            self.swap_successes,
            self.swap_success_rate() * 100.0
        );
        println!("Decoherence: {} pairs removed", self.decoherence_removals);
        println!("Delivered:   {} end-to-end pairs", self.pairs_delivered);
        match (self.mean_fidelity(), self.fidelity_std_dev()) {
            (Some(mean), Some(std)) => {
                println!("Fidelity:    {:.4} ± {:.4} (1σ)", mean, std);
            }
            _ => {}
        }
        if let Some(t) = self.mean_delivery_time_ms() {
            println!("Mean delivery time: {:.2} ms", t);
        }
        println!("=============================\n");
    }
}

// ============================================================
// SIMULATION ENGINE
// ============================================================

/// Discrete-event simulation engine for a quantum repeater chain.
///
/// Drives entanglement generation, swapping, and decoherence events along
/// a fixed ordered path through the network topology.
///
/// **Scope**: currently handles a single linear path with one or more
/// intermediate repeater nodes. Multi-path routing (step 4) will make the
/// path selection dynamic; for now the path is provided at construction time.
///
/// **Event encoding for swapping**:
///   `node_id`        = middle (BSM) node
///   `target_node_id` = left endpoint
///   `resource_id`    = right endpoint
pub struct Simulation {
    pub topology: NetworkTopology,
    scheduler: EventScheduler,
    generation_protocol: BarrettKokProtocol,
    swapping_protocol: EntanglementSwappingProtocol,
    pub config: SimulationConfig,
    pub stats: SimulationStats,
    /// Node IDs from source to destination, e.g. [0, 1, 2]
    path: Vec<usize>,
    /// Repeater node IDs that already have a swap event in the queue.
    /// Prevents duplicate swaps being scheduled when check_and_schedule_swaps
    /// is called multiple times in the same simulation tick.
    pending_swaps: std::collections::HashSet<usize>,
}

impl Simulation {
    pub fn new(
        topology: NetworkTopology,
        generation_protocol: BarrettKokProtocol,
        swapping_protocol: EntanglementSwappingProtocol,
        config: SimulationConfig,
        path: Vec<usize>,
    ) -> Self {
        assert!(path.len() >= 2, "Path must contain at least two nodes");
        Simulation {
            topology,
            scheduler: EventScheduler::new(),
            generation_protocol,
            swapping_protocol,
            config,
            stats: SimulationStats::default(),
            path,
            pending_swaps: std::collections::HashSet::new(),
        }
    }

    /// Run until `target_pairs` delivered or `max_time_ms` elapsed.
    pub fn run(&mut self) {
        // Seed the queue: one generation attempt per link in the path
        self.schedule_all_link_generation(0.0);

        // First periodic decoherence sweep
        self.scheduler.schedule(Event::new(
            self.config.decoherence_check_interval_ms,
            EventType::Decoherence,
            0,
        ));

        loop {
            if self.stats.pairs_delivered >= self.config.target_pairs {
                break;
            }

            let event = match self.scheduler.next_event() {
                Some(e) => e,
                None => break,
            };

            if event.time > self.config.max_time_ms {
                break;
            }

            match event.event_type {
                EventType::EntanglementGeneration => self.handle_generation(event),
                EventType::EntanglementSwapping => self.handle_swap(event),
                EventType::Decoherence => self.handle_decoherence(event),
                // Purification and Measurement are future work
                EventType::Purification | EventType::Measurement => {}
            }
        }
    }

    // ============================================================
    // EVENT HANDLERS
    // ============================================================

    fn handle_generation(&mut self, event: Event) {
        let node_a = event.node_id;
        let node_b = match event.target_node_id {
            Some(id) => id,
            None => return,
        };

        self.stats.generation_attempts += 1;

        match self.topology.attempt_generation_on_link(
            node_a,
            node_b,
            &self.generation_protocol,
            event.time,
            self.config.coherence_time_ms,
        ) {
            Ok(true) => {
                self.stats.generation_successes += 1;

                // For a 2-node path the generated link IS the end-to-end link —
                // no swap needed; deliver immediately.
                let source = *self.path.first().unwrap();
                let dest = *self.path.last().unwrap();
                if (node_a == source && node_b == dest) || (node_a == dest && node_b == source) {
                    if let Some(fidelity) = self.take_end_to_end_fidelity(source, dest) {
                        self.record_delivery(fidelity, event.time);
                        self.schedule_all_link_generation(
                            event.time + self.config.retry_interval_ms,
                        );
                        return;
                    }
                }

                // Multi-hop path: check if any repeater can now swap
                self.check_and_schedule_swaps(event.time);
            }
            Ok(false) => {
                // Channel loss — retry after interval
                self.schedule_generation(
                    node_a,
                    node_b,
                    event.time + self.config.retry_interval_ms,
                );
            }
            Err(_) => {
                // Memory full or nodes already hold a pair — retry later
                self.schedule_generation(
                    node_a,
                    node_b,
                    event.time + self.config.retry_interval_ms,
                );
            }
        }
    }

    fn handle_swap(&mut self, event: Event) {
        let middle = event.node_id;
        let left = match event.target_node_id {
            Some(id) => id,
            None => return,
        };
        let right = match event.resource_id {
            Some(id) => id,
            None => return,
        };

        // This event is now in-flight; clear the pending flag so
        // check_and_schedule_swaps can queue a new one after this resolves.
        self.pending_swaps.remove(&middle);

        // Guard: check which pairs are present before consuming anything.
        // Only reschedule generation for the link that is actually missing —
        // rescheduling an already-present link creates duplicate pairs and
        // cascading events.
        let has_left = self
            .topology
            .get_node(middle)
            .map_or(false, |n| n.find_pair_with(left).is_some());
        let has_right = self
            .topology
            .get_node(middle)
            .map_or(false, |n| n.find_pair_with(right).is_some());

        if !has_left || !has_right {
            if !has_left {
                self.schedule_generation(left, middle, event.time + self.config.retry_interval_ms);
            }
            if !has_right {
                self.schedule_generation(middle, right, event.time + self.config.retry_interval_ms);
            }
            return;
        }

        self.stats.swap_attempts += 1;

        match self.topology.attempt_swap_on_node(
            middle,
            left,
            right,
            &self.swapping_protocol,
            event.time,
            self.config.coherence_time_ms,
        ) {
            Ok(Some(fidelity)) => {
                self.stats.swap_successes += 1;

                let source = *self.path.first().unwrap();
                let dest = *self.path.last().unwrap();

                if left == source && right == dest {
                    self.record_delivery(fidelity, event.time);
                    self.schedule_all_link_generation(
                        event.time + self.config.retry_interval_ms,
                    );
                } else {
                    // Intermediate swap — check whether more swaps can now fire
                    self.check_and_schedule_swaps(event.time);
                }
            }
            Ok(None) => {
                // BSM failed — pairs consumed; reschedule generation on both links
                self.schedule_generation(left, middle, event.time + self.config.retry_interval_ms);
                self.schedule_generation(middle, right, event.time + self.config.retry_interval_ms);
            }
            Err(_) => {
                // Pairs decohered below threshold; reschedule generation
                self.schedule_generation(left, middle, event.time + self.config.retry_interval_ms);
                self.schedule_generation(middle, right, event.time + self.config.retry_interval_ms);
            }
        }
    }

    fn handle_decoherence(&mut self, event: Event) {
        let current_time = event.time;
        let threshold = self.config.fidelity_threshold;
        let path = self.path.clone();

        // Sweep every node in the path: update fidelities, evict dead pairs
        for &node_id in &path {
            if let Some(node) = self.topology.get_node_mut(node_id) {
                let before = node.num_stored_pairs();
                node.stored_pairs.retain_mut(|pair| {
                    pair.update_fidelity(current_time);
                    pair.is_usable(threshold)
                });
                self.stats.decoherence_removals += before - node.num_stored_pairs();
            }
        }

        // Reschedule generation on any link that lost its pair
        for i in 0..(path.len() - 1) {
            let a = path[i];
            let b = path[i + 1];
            let link_ready = self
                .topology
                .get_node(a)
                .map_or(false, |n| n.find_pair_with(b).is_some());
            if !link_ready {
                self.schedule_generation(a, b, current_time + self.config.retry_interval_ms);
            }
        }

        // Schedule the next sweep
        self.scheduler.schedule(Event::new(
            current_time + self.config.decoherence_check_interval_ms,
            EventType::Decoherence,
            0,
        ));
    }

    // ============================================================
    // HELPERS
    // ============================================================

    /// For every repeater node in the path, check if it holds one pair whose
    /// partner lies LEFT of it in the path and one pair whose partner lies
    /// RIGHT of it in the path. If so, schedule a swap after the classical
    /// communication delay.
    ///
    /// This position-based check is correct for multi-hop chains: after an
    /// intermediate swap at node B in [A, B, C, D], node C will hold a pair
    /// with A (not B). Looking up A's position (< C's position) correctly
    /// identifies it as the "left" partner without assuming adjacency.
    fn check_and_schedule_swaps(&mut self, current_time: f64) {
        // Build position map once: node_id → index in path
        let position: std::collections::HashMap<usize, usize> = self
            .path
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        for i in 1..(self.path.len() - 1) {
            let middle = self.path[i];

            // Extract the actual partner IDs while the node is borrowed,
            // then drop the borrow before scheduling (which needs &mut self).
            let (left_partner, right_partner) = {
                let node = match self.topology.get_node(middle) {
                    Some(n) => n,
                    None => continue,
                };

                let left = node
                    .stored_pairs
                    .iter()
                    .find(|p| {
                        position
                            .get(&p.partner_node_id)
                            .map_or(false, |&pos| pos < i)
                    })
                    .map(|p| p.partner_node_id);

                let right = node
                    .stored_pairs
                    .iter()
                    .find(|p| {
                        position
                            .get(&p.partner_node_id)
                            .map_or(false, |&pos| pos > i)
                    })
                    .map(|p| p.partner_node_id);

                (left, right)
            }; // node borrow ends here

            if let (Some(left), Some(right)) = (left_partner, right_partner) {
                // Skip if this node already has a swap event in the queue.
                // Without this guard, multiple successful generation events
                // at the same time each call check_and_schedule_swaps and
                // all see the same ready node, flooding the queue with
                // duplicate swap events.
                if self.pending_swaps.contains(&middle) {
                    continue;
                }
                self.pending_swaps.insert(middle);

                let mut swap_event = Event::new(
                    current_time + self.config.classical_delay_ms,
                    EventType::EntanglementSwapping,
                    middle,
                );
                swap_event.target_node_id = Some(left);
                swap_event.resource_id = Some(right);
                self.scheduler.schedule(swap_event);
            }
        }
    }

    fn schedule_generation(&mut self, node_a: usize, node_b: usize, time: f64) {
        let mut event = Event::new(time, EventType::EntanglementGeneration, node_a);
        event.target_node_id = Some(node_b);
        self.scheduler.schedule(event);
    }

    fn schedule_all_link_generation(&mut self, after_time: f64) {
        let path = self.path.clone();
        for i in 0..(path.len() - 1) {
            self.schedule_generation(path[i], path[i + 1], after_time);
        }
    }

    /// Read the fidelity of the end-to-end pair at `source` (partner = `dest`),
    /// then clear both endpoint memories so they are ready for the next round.
    /// Returns `None` if no such pair exists.
    fn take_end_to_end_fidelity(&mut self, source: usize, dest: usize) -> Option<f64> {
        let fidelity = self
            .topology
            .get_node(source)?
            .stored_pairs
            .iter()
            .find(|p| p.partner_node_id == dest)
            .map(|p| p.fidelity)?;

        self.topology.get_node_mut(source)?.clear_memory();
        self.topology.get_node_mut(dest)?.clear_memory();
        Some(fidelity)
    }

    /// Record a delivered end-to-end pair in statistics.
    fn record_delivery(&mut self, fidelity: f64, time_ms: f64) {
        self.stats.pairs_delivered += 1;
        self.stats.delivered_fidelities.push(fidelity);
        self.stats.delivery_times_ms.push(time_ms);
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::topology::NetworkTopology;
    use crate::protocols::barrett_kok::BarrettKokProtocol;
    use crate::protocols::swapping::EntanglementSwappingProtocol;

    fn two_node_sim(target: usize) -> Simulation {
        Simulation::new(
            NetworkTopology::new_linear(2, 8, 0.0, 0.0), // perfect channel
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig {
                target_pairs: target,
                max_time_ms: 1_000_000.0,
                ..Default::default()
            },
            vec![0, 1],
        )
    }

    fn three_node_sim(target: usize) -> Simulation {
        Simulation::new(
            NetworkTopology::new_linear(3, 8, 0.0, 0.0), // perfect channel
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig {
                target_pairs: target,
                max_time_ms: 1_000_000.0,
                ..Default::default()
            },
            vec![0, 1, 2],
        )
    }

    #[test]
    fn test_two_node_delivers_target_pairs() {
        let mut sim = two_node_sim(10);
        sim.run();
        assert_eq!(sim.stats.pairs_delivered, 10);
    }

    #[test]
    fn test_three_node_delivers_target_pairs() {
        let mut sim = three_node_sim(10);
        sim.run();
        assert_eq!(sim.stats.pairs_delivered, 10);
    }

    #[test]
    fn test_three_node_requires_swaps() {
        let mut sim = three_node_sim(20);
        sim.run();
        assert!(sim.stats.swap_attempts > 0, "3-node path must perform swaps");
        assert!(sim.stats.swap_successes > 0);
    }

    #[test]
    fn test_delivered_fidelities_in_valid_range() {
        let mut sim = three_node_sim(20);
        sim.run();
        for &f in &sim.stats.delivered_fidelities {
            assert!(f > 0.5 && f <= 1.0, "Fidelity {:.4} out of expected range", f);
        }
    }

    #[test]
    fn test_three_node_fidelity_lower_than_two_node() {
        let mut sim2 = two_node_sim(50);
        sim2.run();

        let mut sim3 = three_node_sim(50);
        sim3.run();

        let f2 = sim2.stats.mean_fidelity().unwrap();
        let f3 = sim3.stats.mean_fidelity().unwrap();
        assert!(
            f3 < f2,
            "3-node fidelity ({:.4}) should be lower than 2-node ({:.4}) due to swap degradation",
            f3, f2
        );
    }

    #[test]
    fn test_stats_consistency() {
        let mut sim = three_node_sim(30);
        sim.run();
        let s = &sim.stats;

        assert!(s.generation_successes <= s.generation_attempts);
        assert!(s.swap_successes <= s.swap_attempts);
        // Each delivered pair required exactly one successful swap
        assert_eq!(s.pairs_delivered, s.delivered_fidelities.len());
        assert_eq!(s.pairs_delivered, s.delivery_times_ms.len());
    }

    #[test]
    fn test_max_time_cutoff() {
        let mut sim = Simulation::new(
            NetworkTopology::new_linear(3, 8, 100.0, 2.0), // very lossy channel
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig {
                target_pairs: 1_000_000,
                max_time_ms: 1.0, // tiny budget — should stop early
                ..Default::default()
            },
            vec![0, 1, 2],
        );
        sim.run();
        // Should not deliver 1M pairs in 1ms on a very lossy channel
        assert!(sim.stats.pairs_delivered < 1_000_000);
    }

    #[test]
    fn test_four_node_delivers_target_pairs() {
        // Path [0,1,2,3]: two repeaters, two swaps per delivered pair.
        // Uses routing to compute the path.
        use crate::network::find_shortest_path;
        let topology = NetworkTopology::new_linear(4, 8, 0.0, 0.0);
        let path = find_shortest_path(&topology, 0, 3).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3]);

        let mut sim = Simulation::new(
            topology,
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig {
                target_pairs: 5,
                max_time_ms: 1_000_000.0,
                ..Default::default()
            },
            path,
        );
        sim.run();
        assert_eq!(sim.stats.pairs_delivered, 5);
    }

    #[test]
    fn test_four_node_requires_more_swaps_than_three_node() {
        use crate::network::find_shortest_path;

        let topology3 = NetworkTopology::new_linear(3, 8, 0.0, 0.0);
        let path3 = find_shortest_path(&topology3, 0, 2).unwrap();
        let mut sim3 = Simulation::new(
            topology3,
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig { target_pairs: 20, max_time_ms: 1_000_000.0, ..Default::default() },
            path3,
        );
        sim3.run();

        let topology4 = NetworkTopology::new_linear(4, 8, 0.0, 0.0);
        let path4 = find_shortest_path(&topology4, 0, 3).unwrap();
        let mut sim4 = Simulation::new(
            topology4,
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig { target_pairs: 20, max_time_ms: 1_000_000.0, ..Default::default() },
            path4,
        );
        sim4.run();

        // 4-node chain needs 2 swaps per pair; 3-node needs 1
        assert!(
            sim4.stats.swap_successes > sim3.stats.swap_successes,
            "4-node ({} swaps) should need more swaps than 3-node ({} swaps)",
            sim4.stats.swap_successes,
            sim3.stats.swap_successes
        );
    }

    #[test]
    fn test_four_node_fidelity_lower_than_three_node() {
        use crate::network::find_shortest_path;

        let make_sim = |n: usize| {
            let topology = NetworkTopology::new_linear(n, 8, 0.0, 0.0);
            let path = find_shortest_path(&topology, 0, n - 1).unwrap();
            Simulation::new(
                topology,
                BarrettKokProtocol::sequence_parameters(),
                EntanglementSwappingProtocol::sequence_parameters(),
                SimulationConfig { target_pairs: 50, max_time_ms: 1_000_000.0, ..Default::default() },
                path,
            )
        };

        let mut sim3 = make_sim(3);
        sim3.run();
        let mut sim4 = make_sim(4);
        sim4.run();

        let f3 = sim3.stats.mean_fidelity().unwrap();
        let f4 = sim4.stats.mean_fidelity().unwrap();
        assert!(
            f4 < f3,
            "4-node fidelity ({:.4}) should be lower than 3-node ({:.4})",
            f4, f3
        );
    }

    #[test]
    fn test_routing_path_used_correctly() {
        // Verify that the path BFS computes is actually what drives the simulation.
        // Use a star topology: source=1, dest=2, must go through center (node 0).
        use crate::network::find_shortest_path;
        use crate::network::TopologyType;

        let topology = NetworkTopology::new_star(3, 8, 0.0, 0.0);
        // In star(3): node 0 = center, nodes 1,2 = periphery
        // Channel 0-1 and 0-2 exist; no direct 1-2 channel.
        let path = find_shortest_path(&topology, 1, 2).unwrap();
        assert_eq!(path, vec![1, 0, 2]); // must route through center

        let mut sim = Simulation::new(
            topology,
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig { target_pairs: 5, max_time_ms: 1_000_000.0, ..Default::default() },
            path,
        );
        sim.run();
        assert_eq!(sim.stats.pairs_delivered, 5);
        assert!(sim.stats.swap_successes >= 5); // one swap per delivered pair
    }

    #[test]
    fn test_decoherence_removes_pairs() {
        // Very short coherence time forces many decoherence removals
        let mut sim = Simulation::new(
            NetworkTopology::new_linear(3, 8, 0.0, 0.0),
            BarrettKokProtocol::sequence_parameters(),
            EntanglementSwappingProtocol::sequence_parameters(),
            SimulationConfig {
                target_pairs: 5,
                coherence_time_ms: 0.001, // 1 μs — pairs die before swap fires
                decoherence_check_interval_ms: 0.0005,
                max_time_ms: 100_000.0,
                ..Default::default()
            },
            vec![0, 1, 2],
        );
        sim.run();
        assert!(
            sim.stats.decoherence_removals > 0,
            "Expected decoherence removals with 1μs coherence time"
        );
    }
}
