use crate::quantum::TwoQubitState;

/// A quantum network node (processor or repeater)
pub struct QuantumNode {
    pub id: usize,
    // TODO: Add fields you think are needed
}

impl QuantumNode {
    pub fn new(id: usize) -> Self {
        todo!("Create a new quantum node")
    }

    // TODO: Add methods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        todo!("Test creating a node")
    }
}
