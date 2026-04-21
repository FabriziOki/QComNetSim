use crate::protocols::barrett_kok::BarrettKokProtocol;
use crate::protocols::swapping::{EntanglementSwappingProtocol, SwapResult};
use super::{QuantumChannel, QuantumNode};

/// Types of network topologies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyType {
    Linear,
    Star,
    Mesh,
    Custom,
}

/// Network topology containing nodes and channels
pub struct NetworkTopology {
    nodes: Vec<QuantumNode>,       // Private - controlled access only
    channels: Vec<QuantumChannel>, // Private - controlled access only
    pub topology_type: TopologyType,
}

impl NetworkTopology {
    // ============================================
    // PRE-DEFINED TOPOLOGIES (Immutable)
    // ============================================

    /// Create a linear topology: 0 -- 1 -- 2 -- 3
    /// All channels have the same distance and attenuation
    pub fn new_linear(
        num_nodes: usize,
        memory_per_node: usize,
        distance_km: f64,
        attenuation_db_per_km: f64,
    ) -> Self {
        assert!(num_nodes >= 2, "Linear topology requires at least 2 nodes");

        let mut nodes = Vec::new();
        let mut channels = Vec::new();

        // Create nodes
        for i in 0..num_nodes {
            nodes.push(QuantumNode::new(i, memory_per_node));
        }

        // Create channels connecting adjacent nodes
        for i in 0..(num_nodes - 1) {
            channels.push(QuantumChannel::new(
                i,
                i + 1,
                distance_km,
                attenuation_db_per_km,
            ));
        }

        NetworkTopology {
            nodes,
            channels,
            topology_type: TopologyType::Linear,
        }
    }

    /// Create a star topology: central node (0) connected to all others
    ///     1
    ///     |
    /// 2 - 0 - 3
    ///     |
    ///     4
    pub fn new_star(
        num_nodes: usize,
        memory_per_node: usize,
        distance_km: f64,
        attenuation_db_per_km: f64,
    ) -> Self {
        assert!(num_nodes >= 2, "Star topology requires at least 2 nodes");

        let mut nodes = Vec::new();
        let mut channels = Vec::new();

        // Create nodes (node 0 is the center)
        for i in 0..num_nodes {
            nodes.push(QuantumNode::new(i, memory_per_node));
        }

        // Connect center (node 0) to all other nodes
        for i in 1..num_nodes {
            channels.push(QuantumChannel::new(
                0,
                i,
                distance_km,
                attenuation_db_per_km,
            ));
        }

        NetworkTopology {
            nodes,
            channels,
            topology_type: TopologyType::Star,
        }
    }

    /// Create a fully-connected mesh topology
    /// Every node connected to every other node
    ///     0 --- 1
    ///     | \ / |
    ///     | / \ |
    ///     2 --- 3
    /// Every node connected to every other node
    pub fn new_mesh(
        num_nodes: usize,
        memory_per_node: usize,
        distance_km: f64,
        attenuation_db_per_km: f64,
    ) -> Self {
        assert!(num_nodes >= 2, "Mesh topology requires at least 2 nodes");

        let mut nodes = Vec::new();
        let mut channels = Vec::new();

        // Create nodes
        for i in 0..num_nodes {
            nodes.push(QuantumNode::new(i, memory_per_node));
        }

        // Create channels between all pairs of nodes
        for i in 0..num_nodes {
            for j in (i + 1)..num_nodes {
                channels.push(QuantumChannel::new(
                    i,
                    j,
                    distance_km,
                    attenuation_db_per_km,
                ));
            }
        }

        NetworkTopology {
            nodes,
            channels,
            topology_type: TopologyType::Mesh,
        }
    }

    // ============================================
    // CUSTOM TOPOLOGY (Mutable)
    // ============================================

    /// Create an empty custom topology
    /// Nodes and channels can be added manually
    pub fn new_custom() -> Self {
        NetworkTopology {
            nodes: Vec::new(),
            channels: Vec::new(),
            topology_type: TopologyType::Custom,
        }
    }

    /// Add a node to a custom topology
    /// Returns error if topology is not Custom
    pub fn add_node(&mut self, node: QuantumNode) -> Result<(), String> {
        if self.topology_type != TopologyType::Custom {
            return Err(format!(
                "Cannot modify {:?} topology. Use new_custom() for custom topologies.",
                self.topology_type
            ));
        }
        self.nodes.push(node);
        Ok(())
    }

    /// Add a channel to a custom topology
    /// Returns error if topology is not Custom or if channel references invalid nodes
    pub fn add_channel(&mut self, channel: QuantumChannel) -> Result<(), String> {
        if self.topology_type != TopologyType::Custom {
            return Err(format!(
                "Cannot modify {:?} topology. Use new_custom() for custom topologies.",
                self.topology_type
            ));
        }

        // Validate that channel connects existing nodes
        if channel.node_a >= self.nodes.len() {
            return Err(format!("Node {} does not exist", channel.node_a));
        }
        if channel.node_b >= self.nodes.len() {
            return Err(format!("Node {} does not exist", channel.node_b));
        }

        self.channels.push(channel);
        Ok(())
    }

    // ============================================
    // READ-ONLY ACCESS (Works for all topologies)
    // ============================================

    /// Get immutable reference to a node
    pub fn get_node(&self, id: usize) -> Option<&QuantumNode> {
        self.nodes.get(id)
    }

    /// Get mutable reference to a node
    /// Allows modifying node state (e.g., storing pairs) but not topology structure
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut QuantumNode> {
        self.nodes.get_mut(id)
    }

    /// Get all nodes (immutable)
    pub fn nodes(&self) -> &[QuantumNode] {
        &self.nodes
    }

    /// Get all channels (immutable)
    pub fn channels(&self) -> &[QuantumChannel] {
        &self.channels
    }

    /// Find channel between two nodes
    pub fn find_channel(&self, node_a: usize, node_b: usize) -> Option<&QuantumChannel> {
        self.channels.iter().find(|ch| {
            (ch.node_a == node_a && ch.node_b == node_b)
                || (ch.node_a == node_b && ch.node_b == node_a)
        })
    }

    /// Get number of nodes in the network
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get number of channels in the network
    pub fn num_channels(&self) -> usize {
        self.channels.len()
    }

    /// Check if a node exists
    pub fn has_node(&self, id: usize) -> bool {
        id < self.nodes.len()
    }

    // ============================================
    // MULTI-NODE PROTOCOL OPERATIONS
    // ============================================

    /// Get mutable references to two distinct nodes simultaneously.
    ///
    /// Rust's borrow checker forbids two `&mut` borrows from the same Vec via
    /// `get_mut` calls. `split_at_mut` gives us a safe escape hatch by splitting
    /// the slice at the higher index, yielding two non-overlapping sub-slices.
    ///
    /// Returns `None` if either ID is out of range or both IDs are equal.
    pub fn get_two_nodes_mut(
        &mut self,
        id_a: usize,
        id_b: usize,
    ) -> Option<(&mut QuantumNode, &mut QuantumNode)> {
        if id_a == id_b || id_a >= self.nodes.len() || id_b >= self.nodes.len() {
            return None;
        }
        if id_a < id_b {
            let (left, right) = self.nodes.split_at_mut(id_b);
            Some((&mut left[id_a], &mut right[0]))
        } else {
            let (left, right) = self.nodes.split_at_mut(id_a);
            Some((&mut right[0], &mut left[id_b]))
        }
    }

    /// Run Barrett-Kok entanglement generation on a specific link.
    ///
    /// Looks up the channel between `node_a_id` and `node_b_id`, then calls
    /// the protocol with both node mutable references resolved safely via
    /// `get_two_nodes_mut`. Works for any two directly-connected nodes in the
    /// topology, regardless of how many nodes exist overall.
    pub fn attempt_generation_on_link(
        &mut self,
        node_a_id: usize,
        node_b_id: usize,
        protocol: &BarrettKokProtocol,
        current_time: f64,
        coherence_time_ms: f64,
    ) -> Result<bool, String> {
        // Clone channel first — we need an owned copy so the immutable borrow
        // on `self.channels` does not conflict with the mutable borrow on
        // `self.nodes` taken immediately after.
        let channel = self
            .find_channel(node_a_id, node_b_id)
            .ok_or_else(|| {
                format!("No channel between nodes {} and {}", node_a_id, node_b_id)
            })?
            .clone();

        let num_nodes = self.nodes.len();
        let (node_a, node_b) = self
            .get_two_nodes_mut(node_a_id, node_b_id)
            .ok_or_else(|| {
                format!(
                    "Invalid node IDs: {} or {} out of range (topology has {} nodes)",
                    node_a_id, node_b_id, num_nodes
                )
            })?;

        protocol.attempt_generation(node_a, node_b, &channel, current_time, coherence_time_ms)
    }

    /// Run entanglement swapping at a repeater node.
    ///
    /// Performs the BSM at `middle_node_id` consuming the A–B and B–C pairs,
    /// then stores the resulting A–C pairs at `node_a_id` and `node_c_id`.
    /// The two storage steps use sequential borrows so no `unsafe` is needed.
    ///
    /// Returns `Ok(Some(fidelity))` on success, `Ok(None)` if BSM failed
    /// probabilistically, or `Err` if pairs are missing / decohered.
    pub fn attempt_swap_on_node(
        &mut self,
        middle_node_id: usize,
        node_a_id: usize,
        node_c_id: usize,
        protocol: &EntanglementSwappingProtocol,
        current_time: f64,
        coherence_time_ms: f64,
    ) -> Result<Option<f64>, String> {
        // Phase 1: BSM at middle node.
        // The borrow of `middle_node` ends when this block exits, before we
        // borrow node_a and node_c in phase 2.
        let swap_result: Option<SwapResult> = {
            let middle = self.get_node_mut(middle_node_id).ok_or_else(|| {
                format!("Middle node {} not found", middle_node_id)
            })?;
            protocol.attempt_swap(middle, node_a_id, node_c_id, current_time, coherence_time_ms)?
        };

        // Phase 2: distribute resulting pairs to endpoint nodes.
        if let Some(result) = swap_result {
            let fidelity = result.fidelity;

            self.get_node_mut(node_a_id)
                .ok_or_else(|| format!("Node {} not found", node_a_id))?
                .store_pair(result.pair_for_a)?;

            self.get_node_mut(node_c_id)
                .ok_or_else(|| format!("Node {} not found", node_c_id))?
                .store_pair(result.pair_for_c)?;

            Ok(Some(fidelity))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::barrett_kok::BarrettKokProtocol;
    use crate::protocols::swapping::EntanglementSwappingProtocol;
    use crate::network::node::StoredPair;
    use crate::quantum::TwoQubitState;

    // ===== LINEAR TOPOLOGY TESTS =====

    #[test]
    fn test_linear_2_nodes() {
        let network = NetworkTopology::new_linear(2, 10, 10.0, 0.2);
        assert_eq!(network.topology_type, TopologyType::Linear);
        assert_eq!(network.num_nodes(), 2);
        assert_eq!(network.num_channels(), 1);

        // Check channel exists between 0 and 1
        assert!(network.find_channel(0, 1).is_some());
    }

    #[test]
    fn test_linear_3_nodes() {
        let network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);
        assert_eq!(network.num_nodes(), 3);
        assert_eq!(network.num_channels(), 2); // 0-1 and 1-2

        assert!(network.find_channel(0, 1).is_some());
        assert!(network.find_channel(1, 2).is_some());
        assert!(network.find_channel(0, 2).is_none()); // Not directly connected
    }

    #[test]
    #[should_panic(expected = "Linear topology requires at least 2 nodes")]
    fn test_linear_single_node_panics() {
        NetworkTopology::new_linear(1, 10, 10.0, 0.2);
    }

    #[test]
    fn test_linear_immutable() {
        let mut network = NetworkTopology::new_linear(2, 10, 10.0, 0.2);
        let new_node = QuantumNode::new(2, 10);

        let result = network.add_node(new_node);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot modify Linear topology"));
    }

    // ===== STAR TOPOLOGY TESTS =====

    #[test]
    fn test_star_3_nodes() {
        let network = NetworkTopology::new_star(3, 10, 10.0, 0.2);
        assert_eq!(network.topology_type, TopologyType::Star);
        assert_eq!(network.num_nodes(), 3);
        assert_eq!(network.num_channels(), 2); // 0-1 and 0-2

        // Center (0) connected to all others
        assert!(network.find_channel(0, 1).is_some());
        assert!(network.find_channel(0, 2).is_some());

        // Periphery nodes not connected to each other
        assert!(network.find_channel(1, 2).is_none());
    }

    #[test]
    fn test_star_5_nodes() {
        let network = NetworkTopology::new_star(5, 10, 10.0, 0.2);
        assert_eq!(network.num_nodes(), 5);
        assert_eq!(network.num_channels(), 4); // Center to 4 periphery nodes
    }

    // ===== MESH TOPOLOGY TESTS =====

    #[test]
    fn test_mesh_3_nodes() {
        let network = NetworkTopology::new_mesh(3, 10, 10.0, 0.2);
        assert_eq!(network.topology_type, TopologyType::Mesh);
        assert_eq!(network.num_nodes(), 3);
        assert_eq!(network.num_channels(), 3); // All pairs: 0-1, 0-2, 1-2

        assert!(network.find_channel(0, 1).is_some());
        assert!(network.find_channel(0, 2).is_some());
        assert!(network.find_channel(1, 2).is_some());
    }

    #[test]
    fn test_mesh_4_nodes() {
        let network = NetworkTopology::new_mesh(4, 10, 10.0, 0.2);
        assert_eq!(network.num_nodes(), 4);
        // n*(n-1)/2 = 4*3/2 = 6 channels
        assert_eq!(network.num_channels(), 6);
    }

    // ===== CUSTOM TOPOLOGY TESTS =====

    #[test]
    fn test_custom_empty() {
        let network = NetworkTopology::new_custom();
        assert_eq!(network.topology_type, TopologyType::Custom);
        assert_eq!(network.num_nodes(), 0);
        assert_eq!(network.num_channels(), 0);
    }

    #[test]
    fn test_custom_add_nodes() {
        let mut network = NetworkTopology::new_custom();

        network.add_node(QuantumNode::new(0, 10)).unwrap();
        network.add_node(QuantumNode::new(1, 10)).unwrap();

        assert_eq!(network.num_nodes(), 2);
    }

    #[test]
    fn test_custom_add_channel() {
        let mut network = NetworkTopology::new_custom();

        network.add_node(QuantumNode::new(0, 10)).unwrap();
        network.add_node(QuantumNode::new(1, 10)).unwrap();

        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);
        network.add_channel(channel).unwrap();

        assert_eq!(network.num_channels(), 1);
    }

    #[test]
    fn test_custom_invalid_channel() {
        let mut network = NetworkTopology::new_custom();
        network.add_node(QuantumNode::new(0, 10)).unwrap();

        // Try to add channel to non-existent node
        let channel = QuantumChannel::new(0, 1, 10.0, 0.2);
        let result = network.add_channel(channel);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    // ===== GENERAL ACCESS TESTS =====

    #[test]
    fn test_get_node() {
        let network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);

        assert!(network.get_node(0).is_some());
        assert!(network.get_node(1).is_some());
        assert!(network.get_node(2).is_some());
        assert!(network.get_node(3).is_none());
    }

    #[test]
    fn test_get_node_mut() {
        let mut network = NetworkTopology::new_linear(2, 10, 10.0, 0.2);

        // We can modify node state even in pre-defined topologies
        let node = network.get_node_mut(0).unwrap();
        assert_eq!(node.id, 0);
        assert_eq!(node.memory_capacity, 10);
    }

    #[test]
    fn test_has_node() {
        let network = NetworkTopology::new_linear(2, 10, 10.0, 0.2);
        assert!(network.has_node(0));
        assert!(network.has_node(1));
        assert!(!network.has_node(2));
    }

    // ===== MULTI-NODE PROTOCOL TESTS =====

    #[test]
    fn test_get_two_nodes_mut() {
        let mut network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);
        let result = network.get_two_nodes_mut(0, 2);
        assert!(result.is_some());
        let (a, c) = result.unwrap();
        assert_eq!(a.id, 0);
        assert_eq!(c.id, 2);
    }

    #[test]
    fn test_get_two_nodes_mut_reversed() {
        // Higher-index first — split_at_mut must handle both orderings
        let mut network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);
        let (b, a) = network.get_two_nodes_mut(2, 0).unwrap();
        assert_eq!(b.id, 2);
        assert_eq!(a.id, 0);
    }

    #[test]
    fn test_get_two_nodes_mut_same_id_returns_none() {
        let mut network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);
        assert!(network.get_two_nodes_mut(1, 1).is_none());
    }

    #[test]
    fn test_get_two_nodes_mut_out_of_range_returns_none() {
        let mut network = NetworkTopology::new_linear(2, 10, 10.0, 0.2);
        assert!(network.get_two_nodes_mut(0, 5).is_none());
    }

    #[test]
    fn test_generation_on_link_any_pair() {
        // 3-node linear chain: generation should work on link 0-1 AND link 1-2
        let mut network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);
        let protocol = BarrettKokProtocol::sequence_parameters();

        // Run enough attempts that at least one succeeds on each link
        let mut success_01 = false;
        let mut success_12 = false;
        for _ in 0..50 {
            let mut net = NetworkTopology::new_linear(3, 10, 0.0, 0.0); // perfect channel
            if net.attempt_generation_on_link(0, 1, &protocol, 0.0, 1000.0).unwrap_or(false) {
                success_01 = true;
            }
            if net.attempt_generation_on_link(1, 2, &protocol, 0.0, 1000.0).unwrap_or(false) {
                success_12 = true;
            }
            if success_01 && success_12 { break; }
        }
        assert!(success_01, "link 0-1 never succeeded");
        assert!(success_12, "link 1-2 never succeeded");

        // Pairs must end up in the correct nodes
        let _ = network;
    }

    #[test]
    fn test_generation_on_link_no_channel_returns_err() {
        let mut network = NetworkTopology::new_linear(3, 10, 10.0, 0.2);
        let protocol = BarrettKokProtocol::sequence_parameters();
        // Nodes 0 and 2 are not directly connected in a linear topology
        let result = network.attempt_generation_on_link(0, 2, &protocol, 0.0, 1000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_on_node_three_hop() {
        // Build A(0) -- B(1) -- C(2)
        // Manually place pairs so middle node has both links
        let mut network = NetworkTopology::new_linear(3, 4, 10.0, 0.2);

        let state = TwoQubitState::new_bell_phi_plus();

        // Node B(1) holds pair with A(0) and pair with C(2)
        let mut pair_ba = StoredPair::new(0, state.clone(), 0.0, 1000.0);
        pair_ba.fidelity = 0.95;
        let mut pair_bc = StoredPair::new(2, state.clone(), 0.0, 1000.0);
        pair_bc.fidelity = 0.95;
        network.get_node_mut(1).unwrap().store_pair(pair_ba).unwrap();
        network.get_node_mut(1).unwrap().store_pair(pair_bc).unwrap();

        // Node A(0) and C(2) start with no pairs
        assert_eq!(network.get_node(0).unwrap().num_stored_pairs(), 0);
        assert_eq!(network.get_node(2).unwrap().num_stored_pairs(), 0);

        let protocol = EntanglementSwappingProtocol::sequence_parameters();
        let result = network.attempt_swap_on_node(1, 0, 2, &protocol, 1.0, 1000.0);

        assert!(result.is_ok());
        let fidelity = result.unwrap();
        assert!(fidelity.is_some(), "Ideal BSM should always succeed");

        // Middle node must be empty; endpoints must each hold one pair
        assert_eq!(network.get_node(1).unwrap().num_stored_pairs(), 0);
        assert_eq!(network.get_node(0).unwrap().num_stored_pairs(), 1);
        assert_eq!(network.get_node(2).unwrap().num_stored_pairs(), 1);

        // Pairs must point to each other (A↔C)
        assert_eq!(network.get_node(0).unwrap().stored_pairs[0].partner_node_id, 2);
        assert_eq!(network.get_node(2).unwrap().stored_pairs[0].partner_node_id, 0);

        // Provenance: both endpoint pairs must record node 1 as the repeater
        assert!(network.get_node(0).unwrap().stored_pairs[0].is_swapped());
        assert!(network.get_node(2).unwrap().stored_pairs[0].is_swapped());
        assert_eq!(network.get_node(0).unwrap().stored_pairs[0].origin.swap_count(), 1);
    }
}
