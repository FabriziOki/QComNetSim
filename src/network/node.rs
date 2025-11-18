use crate::quantum::TwoQubitState;

/// A quantum entangled pair stored in node memory
#[derive(Clone)]
pub struct StoredPair {
    /// ID of the partner node this qubit is entangled with
    pub partner_node_id: usize,
    /// The quantum state of this entangled pair
    pub state: TwoQubitState,
    /// Time when this pair was created (for decoherence tracking)
    pub creation_time: f64,
    /// Current fidelity of this pair
    pub fidelity: f64,
}

impl StoredPair {
    /// Create a new stored entangled pair
    pub fn new(partner_node_id: usize, state: TwoQubitState, creation_time: f64) -> Self {
        // Calculate initial fidelity (for now, assume perfect Bell state)
        let ideal_bell = TwoQubitState::new_bell_phi_plus();
        let fidelity = state.fidelity(&ideal_bell);

        StoredPair {
            partner_node_id,
            state,
            creation_time,
            fidelity,
        }
    }
}

/// A quantum network node (processor or repeater)
pub struct QuantumNode {
    /// Unique identifier for this node
    pub id: usize,
    /// Maximum number of qubits this node can store
    pub memory_capacity: usize,
    /// Currently stored entangled pairs
    pub stored_pairs: Vec<StoredPair>,
}

impl QuantumNode {
    /// Create a new quantum node with empty memory
    pub fn new(id: usize, memory_capacity: usize) -> Self {
        QuantumNode {
            id,
            memory_capacity,
            stored_pairs: Vec::new(),
        }
    }

    /// Check if node has available memory
    pub fn has_memory_available(&self) -> bool {
        self.stored_pairs.len() < self.memory_capacity
    }

    /// Get number of free memory slots
    pub fn free_memory(&self) -> usize {
        self.memory_capacity - self.stored_pairs.len()
    }

    /// Store an entangled pair (if memory available)
    pub fn store_pair(&mut self, pair: StoredPair) -> Result<(), String> {
        if !self.has_memory_available() {
            return Err(format!(
                "Node {} memory full ({}/{})",
                self.id,
                self.stored_pairs.len(),
                self.memory_capacity
            ));
        }

        self.stored_pairs.push(pair);
        Ok(())
    }

    /// Find a stored pair with a specific partner node
    pub fn find_pair_with(&self, partner_id: usize) -> Option<usize> {
        self.stored_pairs
            .iter()
            .position(|pair| pair.partner_node_id == partner_id)
    }

    /// Remove and return a stored pair with a specific partner
    pub fn remove_pair_with(&mut self, partner_id: usize) -> Option<StoredPair> {
        if let Some(index) = self.find_pair_with(partner_id) {
            Some(self.stored_pairs.remove(index))
        } else {
            None
        }
    }

    /// Clear all stored pairs (useful for testing or reset)
    pub fn clear_memory(&mut self) {
        self.stored_pairs.clear();
    }

    /// Get total number of stored pairs
    pub fn num_stored_pairs(&self) -> usize {
        self.stored_pairs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = QuantumNode::new(0, 10);
        assert_eq!(node.id, 0);
        assert_eq!(node.memory_capacity, 10);
        assert_eq!(node.num_stored_pairs(), 0);
        assert!(node.has_memory_available());
    }

    #[test]
    fn test_memory_tracking() {
        let node = QuantumNode::new(0, 2);
        assert_eq!(node.free_memory(), 2);
        assert!(node.has_memory_available());
    }

    #[test]
    fn test_store_pair() {
        let mut node = QuantumNode::new(0, 2);

        let bell_state = TwoQubitState::new_bell_phi_plus();
        let pair = StoredPair::new(1, bell_state, 0.0);

        assert!(node.store_pair(pair).is_ok());
        assert_eq!(node.num_stored_pairs(), 1);
        assert_eq!(node.free_memory(), 1);
    }

    #[test]
    fn test_memory_full() {
        let mut node = QuantumNode::new(0, 1);

        let bell_state = TwoQubitState::new_bell_phi_plus();
        let pair1 = StoredPair::new(1, bell_state.clone(), 0.0);
        let pair2 = StoredPair::new(2, bell_state, 0.0);

        assert!(node.store_pair(pair1).is_ok());
        assert!(!node.has_memory_available());

        let result = node.store_pair(pair2);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_pair() {
        let mut node = QuantumNode::new(0, 5);

        let bell_state = TwoQubitState::new_bell_phi_plus();
        let pair1 = StoredPair::new(1, bell_state.clone(), 0.0);
        let pair2 = StoredPair::new(2, bell_state, 0.0);

        node.store_pair(pair1).unwrap();
        node.store_pair(pair2).unwrap();

        assert!(node.find_pair_with(1).is_some());
        assert!(node.find_pair_with(2).is_some());
        assert!(node.find_pair_with(3).is_none());
    }

    #[test]
    fn test_remove_pair() {
        let mut node = QuantumNode::new(0, 5);

        let bell_state = TwoQubitState::new_bell_phi_plus();
        let pair = StoredPair::new(1, bell_state, 0.0);

        node.store_pair(pair).unwrap();
        assert_eq!(node.num_stored_pairs(), 1);

        let removed = node.remove_pair_with(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().partner_node_id, 1);
        assert_eq!(node.num_stored_pairs(), 0);
    }

    #[test]
    fn test_stored_pair_fidelity() {
        let bell_state = TwoQubitState::new_bell_phi_plus();
        let pair = StoredPair::new(1, bell_state, 0.0);

        // Perfect Bell state should have fidelity ≈ 1.0
        assert!((pair.fidelity - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_clear_memory() {
        let mut node = QuantumNode::new(0, 5);

        let bell_state = TwoQubitState::new_bell_phi_plus();
        node.store_pair(StoredPair::new(1, bell_state.clone(), 0.0))
            .unwrap();
        node.store_pair(StoredPair::new(2, bell_state, 0.0))
            .unwrap();

        assert_eq!(node.num_stored_pairs(), 2);

        node.clear_memory();
        assert_eq!(node.num_stored_pairs(), 0);
        assert_eq!(node.free_memory(), 5);
    }
}
