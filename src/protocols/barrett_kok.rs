use crate::network::node::StoredPair;
use crate::network::{QuantumChannel, QuantumNode};
use crate::quantum::TwoQubitState;
use rand::Rng;

/// Barrett-Kok entanglement generation protocol
///
/// Heralded scheme with:
/// - Photon emission from both nodes
/// - Midpoint BSM (Bell State Measurement)
/// - Detector clicks signal success
pub struct BarrettKokProtocol {
    /// BSM (beam splitter) success rate (0.5 for single-atom, 1.0 for ideal)
    pub bsm_efficiency: f64,

    /// Detector efficiency (0.0 to 1.0)
    pub detector_efficiency: f64,

    /// Detector dark count rate (false positives)
    pub dark_count_rate: f64,

    /// Initial fidelity after generation (accounting for imperfections)
    pub initial_fidelity: f64,
}

impl BarrettKokProtocol {
    /// Create protocol matching SeQUeNCe parameters
    pub fn sequence_parameters() -> Self {
        BarrettKokProtocol {
            bsm_efficiency: 0.5,       // Single-atom BSM
            detector_efficiency: 0.90, // From SeQUeNCe
            dark_count_rate: 0.0,      // SeQUeNCe doesn't model this
            initial_fidelity: 0.95,    // From SeQUeNCe
        }
    }

    /// Create realistic protocol (QComNetSim)
    pub fn realistic() -> Self {
        BarrettKokProtocol {
            bsm_efficiency: 0.5,
            detector_efficiency: 0.90,
            dark_count_rate: 0.01, // 1% dark counts (realistic)
            initial_fidelity: 0.95,
        }
    }

    /// Attempt entanglement generation
    pub fn attempt_generation(
        &self,
        node_a: &mut QuantumNode,
        node_b: &mut QuantumNode,
        channel: &QuantumChannel,
        current_time: f64,
        coherence_time_ms: f64,
    ) -> Result<bool, String> {
        let mut rng = rand::rng();

        // Memory checks
        if !node_a.has_memory_available() {
            return Err(format!("Node {} memory full", node_a.id));
        }
        if !node_b.has_memory_available() {
            return Err(format!("Node {} memory full", node_b.id));
        }

        // Match SeQUeNCe's complete model:
        let transmission_prob = channel.success_probability();
        let memory_efficiency = 0.9; // From SeQUeNCe Memory parameter

        // Step 1: Memory emission (both nodes must emit successfully)
        if rng.random::<f64>() >= memory_efficiency {
            return Ok(false); // Node A emission failed
        }
        if rng.random::<f64>() >= memory_efficiency {
            return Ok(false); // Node B emission failed
        }

        // Step 2: Channel transmission (both photons travel to BSM)
        if rng.random::<f64>() >= transmission_prob {
            return Ok(false); // Photon A lost
        }
        if rng.random::<f64>() >= transmission_prob {
            return Ok(false); // Photon B lost
        }

        // Step 3: BSM measurement
        if rng.random::<f64>() >= self.bsm_efficiency {
            return Ok(false); // BSM failed
        }

        // Step 4: Detector clicks (both detectors)
        if rng.random::<f64>() >= self.detector_efficiency {
            return Ok(false); // Detector A failed
        }
        if rng.random::<f64>() >= self.detector_efficiency {
            return Ok(false); // Detector B failed
        }

        // Success! Create entangled pair
        let bell_state = TwoQubitState::new_bell_phi_plus();

        let mut pair_a = StoredPair::new(
            node_b.id,
            bell_state.clone(),
            current_time,
            coherence_time_ms,
        );
        let mut pair_b = StoredPair::new(node_a.id, bell_state, current_time, coherence_time_ms);

        pair_a.fidelity = self.initial_fidelity;
        pair_b.fidelity = self.initial_fidelity;

        node_a.store_pair(pair_a)?;
        node_b.store_pair(pair_b)?;

        Ok(true)
    }

    /// Calculate theoretical success probability
    pub fn theoretical_success_rate(&self, channel: &QuantumChannel) -> f64 {
        let p_trans = channel.success_probability();

        // Both photons arrive × BSM works × both detectors click
        let p_success = p_trans
            * p_trans
            * self.bsm_efficiency
            * self.detector_efficiency
            * self.detector_efficiency;

        p_success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theoretical_rate() {
        let protocol = BarrettKokProtocol::sequence_parameters();
        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);

        let rate = protocol.theoretical_success_rate(&channel);
        assert!(rate > 0.0 && rate < 1.0);
    }
}
