use QComNetSim::network::{NetworkTopology, QuantumChannel, QuantumNode};

fn main() {
    println!("QComNetSim - Network Topology Demo\n");

    // ===== Linear Topology =====
    println!("=== Linear Topology (2 nodes) ===");
    let linear = NetworkTopology::new_linear(2, 10, 10.0, 0.2);
    println!("Nodes: {}", linear.num_nodes());
    println!("Channels: {}", linear.num_channels());
    if let Some(ch) = linear.find_channel(0, 1) {
        println!(
            "Channel 0-1: {} km, success p={:.3}\n",
            ch.distance_km,
            ch.success_probability()
        );
    }

    // ===== Star Topology =====
    println!("=== Star Topology (5 nodes) ===");
    let star = NetworkTopology::new_star(5, 10, 10.0, 0.2);
    println!("Nodes: {}", star.num_nodes());
    println!("Channels: {}", star.num_channels());
    println!("(Center node 0 connected to nodes 1-4)\n");

    // ===== Mesh Topology =====
    println!("=== Mesh Topology (4 nodes) ===");
    let mesh = NetworkTopology::new_mesh(4, 10, 10.0, 0.2);
    println!("Nodes: {}", mesh.num_nodes());
    println!("Channels: {} (fully connected)\n", mesh.num_channels());

    // ===== Custom Topology =====
    println!("=== Custom Topology ===");
    let mut custom = NetworkTopology::new_custom();
    custom.add_node(QuantumNode::new(0, 15)).unwrap();
    custom.add_node(QuantumNode::new(1, 20)).unwrap();
    custom.add_node(QuantumNode::new(2, 10)).unwrap();

    custom
        .add_channel(QuantumChannel::new(0, 1, 5.0, 0.2))
        .unwrap();
    custom
        .add_channel(QuantumChannel::new(1, 2, 15.0, 0.2))
        .unwrap();
    custom
        .add_channel(QuantumChannel::new(0, 2, 25.0, 0.3))
        .unwrap();

    println!("Nodes: {}", custom.num_nodes());
    println!("Channels: {}", custom.num_channels());
    println!("(Triangle topology with varying distances)\n");

    // ===== Test Immutability =====
    println!("=== Testing Immutability ===");
    let mut linear_test = NetworkTopology::new_linear(2, 10, 10.0, 0.2);
    let new_node = QuantumNode::new(2, 10);

    match linear_test.add_node(new_node) {
        Ok(_) => println!("ERROR: Should not be able to modify linear topology!"),
        Err(e) => println!("✓ Correctly prevented modification: {}", e),
    }
}
