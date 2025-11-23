use crate::network::node::StoredPair;
use crate::network::{QuantumChannel, QuantumNode};
use crate::quantum::TwoQubitState;

/// Attempt to generate an entangled pair between two nodes
///
/// Returns Ok(true) if generation succeeded, Ok(false) if failed due to channel loss
pub fn attempt_entanglement_generation(
    node_a: &mut QuantumNode,
    node_b: &mut QuantumNode,
    channel: &QuantumChannel,
    current_time: f64,
    coherence_time_ms: f64,
) -> Result<bool, String> {
    // Check if both nodes have memory available
    if !node_a.has_memory_available() {
        return Err(format!("Node {} memory full", node_a.id));
    }
    if !node_b.has_memory_available() {
        return Err(format!("Node {} memory full", node_b.id));
    }

    // Attempt generation based on channel success probability
    let success = channel.attempt_generation();

    if success {
        // Generate Bell pair |Φ+⟩ = (|00⟩ + |11⟩)/√2
        let bell_state = TwoQubitState::new_bell_phi_plus();

        // Store in both nodes
        let pair_a = StoredPair::new(
            node_b.id,
            bell_state.clone(),
            current_time,
            coherence_time_ms,
        );
        let pair_b = StoredPair::new(node_a.id, bell_state, current_time, coherence_time_ms);

        node_a.store_pair(pair_a)?;
        node_b.store_pair(pair_b)?;

        Ok(true)
    } else {
        Ok(false)
    }
}

/// Statistics for entanglement generation experiments
#[derive(Debug, Default)]
pub struct GenerationStats {
    pub attempts: usize,
    pub successes: usize,
    pub channel_failures: usize,
    pub memory_full_errors: usize,
}

impl GenerationStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn success_rate(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            self.successes as f64 / self.attempts as f64
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== Entanglement Generation Statistics ===");
        println!("Total attempts:     {}", self.attempts);
        println!(
            "Successful:         {} ({:.1}%)",
            self.successes,
            self.success_rate() * 100.0
        );
        println!("Channel failures:   {}", self.channel_failures);
        println!("Memory full:        {}", self.memory_full_errors);
        println!("==========================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::channel::QuantumChannel;

    #[test]
    fn test_successful_generation() {
        let mut node_a = QuantumNode::new(0, 10);
        let mut node_b = QuantumNode::new(1, 10);
        let channel = QuantumChannel::new(0, 1, 0.0, 0.0); // Perfect channel

        let result =
            attempt_entanglement_generation(&mut node_a, &mut node_b, &channel, 0.0, 100.0);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Should succeed
        assert_eq!(node_a.num_stored_pairs(), 1);
        assert_eq!(node_b.num_stored_pairs(), 1);
    }

    #[test]
    fn test_channel_loss() {
        let mut node_a = QuantumNode::new(0, 10);
        let mut node_b = QuantumNode::new(1, 10);
        // Very lossy channel
        let channel = QuantumChannel::new(0, 1, 100.0, 0.5);

        let mut successes = 0;
        let attempts = 100;

        for _ in 0..attempts {
            let mut test_node_a = node_a.clone();
            let mut test_node_b = node_b.clone();

            if let Ok(true) = attempt_entanglement_generation(
                &mut test_node_a,
                &mut test_node_b,
                &channel,
                0.0,
                100.0,
            ) {
                successes += 1;
            }
        }

        // Should have some failures due to loss
        assert!(successes < attempts);
        assert!(successes > 0); // But some successes
    }

    #[test]
    fn test_memory_full() {
        let mut node_a = QuantumNode::new(0, 1); // Only 1 slot
        let mut node_b = QuantumNode::new(1, 10);
        let channel = QuantumChannel::new(0, 1, 0.0, 0.0);

        // First generation should succeed
        let result1 =
            attempt_entanglement_generation(&mut node_a, &mut node_b, &channel, 0.0, 100.0);
        assert!(result1.is_ok());

        // Second should fail - memory full
        let result2 =
            attempt_entanglement_generation(&mut node_a, &mut node_b, &channel, 0.0, 100.0);
        assert!(result2.is_err());
    }
}
