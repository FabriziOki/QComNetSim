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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
