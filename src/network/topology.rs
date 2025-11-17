use super::{QuantumChannel, QuantumNode};

/// Types of network topologies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyType {
    Linear,
    Star,
    Mesh,
}

/// Network topology containing nodes and channels
pub struct NetworkTopology {
    pub nodes: Vec<QuantumNode>,
    pub channels: Vec<QuantumChannel>,
    pub topology_type: TopologyType,
}

impl NetworkTopology {
    pub fn new(topology_type: TopologyType) -> Self {
        todo!("Create an empty network topology")
    }

    // TODO: Add methods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_topology() {
        todo!("Test creating empty topology")
    }

    #[test]
    fn test_linear_topology() {
        todo!("Test creating a 2-node linear network")
    }
}
