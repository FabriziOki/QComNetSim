use crate::network::node::{LinkOrigin, QuantumNode, StoredPair};
use crate::quantum::TwoQubitState;
use rand::Rng;

/// Output of a successful entanglement swap
pub struct SwapResult {
    /// New StoredPair to be placed in node A's memory (partner = node C)
    pub pair_for_a: StoredPair,
    /// New StoredPair to be placed in node C's memory (partner = node A)
    pub pair_for_c: StoredPair,
    /// Fidelity of the resulting end-to-end A–C pair
    pub fidelity: f64,
}

/// Entanglement swapping protocol
///
/// A middle (repeater) node B that holds:
///   - pair A–B  (entangled with node A)
///   - pair B–C  (entangled with node C)
///
/// performs a local Bell State Measurement (BSM) on its two qubits.
/// This projects nodes A and C into an entangled state, extending
/// entanglement across the A–B–C chain to produce an A–C link.
///
/// Fidelity model uses the Werner-state formula:
///   F_out = [ F_AB * F_BC + (1 - F_AB)(1 - F_BC) / 9 ] * gate_fidelity
pub struct EntanglementSwappingProtocol {
    /// Probability that the local BSM succeeds (0.0–1.0)
    pub bsm_efficiency: f64,
    /// Multiplicative fidelity penalty from local gate imperfections (1.0 = ideal)
    pub gate_fidelity: f64,
}

impl EntanglementSwappingProtocol {
    /// Parameters matching SeQUeNCe's default repeater model (ideal local operations)
    pub fn sequence_parameters() -> Self {
        EntanglementSwappingProtocol {
            bsm_efficiency: 1.0,
            gate_fidelity: 1.0,
        }
    }

    /// Realistic parameters for cavity-QED (Er³⁺) repeater nodes
    pub fn realistic() -> Self {
        EntanglementSwappingProtocol {
            bsm_efficiency: 0.90,
            gate_fidelity: 0.98,
        }
    }

    /// Attempt entanglement swapping at `middle_node`.
    ///
    /// Consumes the A–B and B–C pairs from `middle_node`'s memory.
    /// On success returns new `StoredPair` instances for nodes A and C.
    ///
    /// Returns:
    ///   `Ok(Some(SwapResult))` — swap succeeded; caller must store the pairs at A and C
    ///   `Ok(None)`             — BSM failed probabilistically (pairs consumed)
    ///   `Err(String)`          — prerequisite pairs missing or decohered below threshold
    pub fn attempt_swap(
        &self,
        middle_node: &mut QuantumNode,
        node_a_id: usize,
        node_c_id: usize,
        current_time: f64,
        coherence_time_ms: f64,
    ) -> Result<Option<SwapResult>, String> {
        // Verify both required pairs are present before consuming anything
        if middle_node.find_pair_with(node_a_id).is_none() {
            return Err(format!(
                "Node {} has no pair with node {} (A–B link missing)",
                middle_node.id, node_a_id
            ));
        }
        if middle_node.find_pair_with(node_c_id).is_none() {
            return Err(format!(
                "Node {} has no pair with node {} (B–C link missing)",
                middle_node.id, node_c_id
            ));
        }

        // Consume both pairs (BSM measures and destroys the qubits regardless of outcome)
        let mut pair_ab = middle_node.remove_pair_with(node_a_id).unwrap();
        let mut pair_bc = middle_node.remove_pair_with(node_c_id).unwrap();

        // Apply decoherence: account for time elapsed since each pair was created
        pair_ab.update_fidelity(current_time);
        pair_bc.update_fidelity(current_time);

        // Reject pairs that have decohered below the minimum useful fidelity
        const FIDELITY_THRESHOLD: f64 = 0.5;
        if !pair_ab.is_usable(FIDELITY_THRESHOLD) {
            return Err(format!(
                "A–B pair at node {} decohered (F={:.3} < {:.3})",
                middle_node.id, pair_ab.fidelity, FIDELITY_THRESHOLD
            ));
        }
        if !pair_bc.is_usable(FIDELITY_THRESHOLD) {
            return Err(format!(
                "B–C pair at node {} decohered (F={:.3} < {:.3})",
                middle_node.id, pair_bc.fidelity, FIDELITY_THRESHOLD
            ));
        }

        // Probabilistic BSM — failure consumes the qubits without producing a link
        let mut rng = rand::rng();
        if rng.random::<f64>() >= self.bsm_efficiency {
            return Ok(None);
        }

        // Werner-state fidelity formula for ideal BSM:
        //   F_out = F_AB * F_BC + (1 - F_AB)(1 - F_BC) / 9
        // scaled by gate_fidelity to account for local operation errors.
        let f_out = self.theoretical_output_fidelity(pair_ab.fidelity, pair_bc.fidelity);

        // Extend provenance: each new pair records the full chain of repeaters
        // that performed BSMs to establish the A–C link.
        let origin_for_a = pair_ab.origin.clone().extend(middle_node.id);
        let origin_for_c = pair_bc.origin.clone().extend(middle_node.id);

        let swapped_state = TwoQubitState::new_bell_phi_plus();

        let mut pair_for_a = StoredPair::new_swapped(
            node_c_id,
            swapped_state.clone(),
            current_time,
            coherence_time_ms,
            origin_for_a,
        );
        let mut pair_for_c = StoredPair::new_swapped(
            node_a_id,
            swapped_state,
            current_time,
            coherence_time_ms,
            origin_for_c,
        );

        pair_for_a.fidelity = f_out;
        pair_for_c.fidelity = f_out;

        Ok(Some(SwapResult {
            pair_for_a,
            pair_for_c,
            fidelity: f_out,
        }))
    }

    /// Theoretical fidelity of the swapped pair given input fidelities.
    ///
    /// Werner-state result for ideal BSM, scaled by gate fidelity:
    ///   F_out = [ F_AB * F_BC + (1 - F_AB)(1 - F_BC) / 9 ] * gate_fidelity
    pub fn theoretical_output_fidelity(&self, f_ab: f64, f_bc: f64) -> f64 {
        (f_ab * f_bc + (1.0 - f_ab) * (1.0 - f_bc) / 9.0) * self.gate_fidelity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node_with_pair(node_id: usize, partner_id: usize, fidelity: f64) -> QuantumNode {
        let mut node = QuantumNode::new(node_id, 4);
        let state = TwoQubitState::new_bell_phi_plus();
        let mut pair = StoredPair::new(partner_id, state, 0.0, 1000.0);
        pair.fidelity = fidelity;
        node.store_pair(pair).unwrap();
        node
    }

    #[test]
    fn test_swap_succeeds_with_ideal_bsm() {
        let protocol = EntanglementSwappingProtocol::sequence_parameters();

        // Middle node B holds pair with A (id=0) and pair with C (id=2)
        let mut node_b = QuantumNode::new(1, 4);
        let state = TwoQubitState::new_bell_phi_plus();

        let mut pair_ab = StoredPair::new(0, state.clone(), 0.0, 1000.0);
        pair_ab.fidelity = 0.95;
        let mut pair_bc = StoredPair::new(2, state, 0.0, 1000.0);
        pair_bc.fidelity = 0.95;

        node_b.store_pair(pair_ab).unwrap();
        node_b.store_pair(pair_bc).unwrap();

        let result = protocol.attempt_swap(&mut node_b, 0, 2, 1.0, 1000.0);
        assert!(result.is_ok());

        let swap = result.unwrap();
        assert!(swap.is_some(), "Ideal BSM should always succeed");

        let swap = swap.unwrap();
        // New pairs must point A↔C
        assert_eq!(swap.pair_for_a.partner_node_id, 2);
        assert_eq!(swap.pair_for_c.partner_node_id, 0);

        // Middle node memory must be empty after swap
        assert_eq!(node_b.num_stored_pairs(), 0);

        // Provenance: both pairs should record node B (id=1) as the repeater
        assert!(swap.pair_for_a.is_swapped());
        assert!(swap.pair_for_c.is_swapped());
        assert_eq!(swap.pair_for_a.origin.swap_count(), 1);
        assert_eq!(swap.pair_for_c.origin.swap_count(), 1);
        match &swap.pair_for_a.origin {
            LinkOrigin::Swapped { repeaters } => assert_eq!(repeaters, &[1]),
            _ => panic!("Expected Swapped origin"),
        }
    }

    #[test]
    fn test_swap_provenance_chained() {
        // Two-hop chain: A(0)–B(1)–C(2)–D(3)
        // First swap at B produces A–C (1 repeater: B)
        // Second swap at C produces A–D (2 repeaters: B, C)
        let protocol = EntanglementSwappingProtocol::sequence_parameters();
        let state = TwoQubitState::new_bell_phi_plus();

        // --- First swap at B (id=1): consumes A-B and B-C ---
        let mut node_b = QuantumNode::new(1, 4);
        let mut pair_ab = StoredPair::new(0, state.clone(), 0.0, 1000.0);
        pair_ab.fidelity = 0.95;
        let mut pair_bc = StoredPair::new(2, state.clone(), 0.0, 1000.0);
        pair_bc.fidelity = 0.95;
        node_b.store_pair(pair_ab).unwrap();
        node_b.store_pair(pair_bc).unwrap();

        let result_b = protocol.attempt_swap(&mut node_b, 0, 2, 1.0, 1000.0)
            .unwrap().unwrap();
        // pair_for_a has origin extended by B(1)
        assert_eq!(result_b.pair_for_a.origin.swap_count(), 1);

        // --- Second swap at C (id=2): consumes A-C (from first swap) and C-D ---
        let mut node_c = QuantumNode::new(2, 4);
        // The A-C pair as seen from C's side is result_b.pair_for_c (partner=A, origin extended by B)
        node_c.store_pair(result_b.pair_for_c).unwrap();
        let mut pair_cd = StoredPair::new(3, state.clone(), 0.0, 1000.0);
        pair_cd.fidelity = 0.95;
        node_c.store_pair(pair_cd).unwrap();

        let result_c = protocol.attempt_swap(&mut node_c, 0, 3, 2.0, 1000.0)
            .unwrap().unwrap();

        // A–D pair must show 2 repeaters (B then C)
        assert_eq!(result_c.pair_for_a.origin.swap_count(), 2);
        match &result_c.pair_for_a.origin {
            LinkOrigin::Swapped { repeaters } => assert_eq!(repeaters, &[1, 2]),
            _ => panic!("Expected Swapped origin"),
        }
    }

    #[test]
    fn test_swap_fidelity_degradation() {
        let protocol = EntanglementSwappingProtocol::sequence_parameters();
        let f_ab = 0.95_f64;
        let f_bc = 0.90_f64;

        let expected = f_ab * f_bc + (1.0 - f_ab) * (1.0 - f_bc) / 9.0;
        let computed = protocol.theoretical_output_fidelity(f_ab, f_bc);

        assert!((computed - expected).abs() < 1e-10);
        // Swapped fidelity must be strictly less than both inputs
        assert!(computed < f_ab);
        assert!(computed < f_bc);
    }

    #[test]
    fn test_swap_fails_missing_pair() {
        let protocol = EntanglementSwappingProtocol::sequence_parameters();
        // Node B only has pair with A; B–C is missing
        let mut node_b = make_node_with_pair(1, 0, 0.95);

        let result = protocol.attempt_swap(&mut node_b, 0, 2, 0.0, 1000.0);
        assert!(result.is_err());
        // Existing pair must NOT be consumed on error
        assert_eq!(node_b.num_stored_pairs(), 1);
    }

    #[test]
    fn test_swap_fails_decohered_pair() {
        let protocol = EntanglementSwappingProtocol::sequence_parameters();
        let mut node_b = QuantumNode::new(1, 4);
        let state = TwoQubitState::new_bell_phi_plus();

        // pair_ab: created at t=0 with very short coherence time — will be decohered by t=1000
        let mut pair_ab = StoredPair::new(0, state.clone(), 0.0, 0.001);
        pair_ab.fidelity = 0.95;
        let mut pair_bc = StoredPair::new(2, state, 0.0, 1000.0);
        pair_bc.fidelity = 0.95;

        node_b.store_pair(pair_ab).unwrap();
        node_b.store_pair(pair_bc).unwrap();

        // current_time = 1000ms >> coherence_time 0.001ms → pair_ab decohered
        let result = protocol.attempt_swap(&mut node_b, 0, 2, 1000.0, 1000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_gate_fidelity_penalty() {
        let protocol = EntanglementSwappingProtocol {
            bsm_efficiency: 1.0,
            gate_fidelity: 0.98,
        };

        let f_ab = 0.95_f64;
        let f_bc = 0.95_f64;
        let ideal = f_ab * f_bc + (1.0 - f_ab) * (1.0 - f_bc) / 9.0;
        let with_penalty = protocol.theoretical_output_fidelity(f_ab, f_bc);

        assert!((with_penalty - ideal * 0.98).abs() < 1e-10);
    }
}
