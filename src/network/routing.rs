use crate::network::topology::NetworkTopology;
use std::collections::{HashMap, VecDeque};

/// Find the shortest path between `source` and `dest` using BFS over the
/// topology's channel graph.
///
/// Returns `Some(path)` where `path[0] == source` and `path.last() == dest`,
/// or `None` if no path exists (disconnected graph or invalid node IDs).
///
/// All channels are treated as unit-weight undirected edges.
pub fn find_shortest_path(
    topology: &NetworkTopology,
    source: usize,
    dest: usize,
) -> Option<Vec<usize>> {
    if !topology.has_node(source) || !topology.has_node(dest) {
        return None;
    }
    if source == dest {
        return Some(vec![source]);
    }

    let n = topology.num_nodes();
    let mut visited = vec![false; n];
    let mut parent: HashMap<usize, usize> = HashMap::new();
    let mut queue = VecDeque::new();

    visited[source] = true;
    queue.push_back(source);

    'bfs: while let Some(current) = queue.pop_front() {
        for channel in topology.channels() {
            let neighbor = if channel.node_a == current {
                channel.node_b
            } else if channel.node_b == current {
                channel.node_a
            } else {
                continue;
            };

            if visited[neighbor] {
                continue;
            }
            visited[neighbor] = true;
            parent.insert(neighbor, current);

            if neighbor == dest {
                break 'bfs;
            }
            queue.push_back(neighbor);
        }
    }

    // Reconstruct path by walking parent pointers back from dest to source
    if !visited[dest] {
        return None;
    }

    let mut path = vec![dest];
    let mut node = dest;
    while node != source {
        node = *parent.get(&node)?;
        path.push(node);
    }
    path.reverse();
    Some(path)
}

/// Return all node IDs that lie strictly between source and destination on
/// the shortest path (i.e. the repeater nodes that will perform swaps).
pub fn repeaters_on_path(
    topology: &NetworkTopology,
    source: usize,
    dest: usize,
) -> Option<Vec<usize>> {
    let path = find_shortest_path(topology, source, dest)?;
    if path.len() <= 2 {
        Some(vec![]) // direct link, no repeaters
    } else {
        Some(path[1..path.len() - 1].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::topology::NetworkTopology;

    #[test]
    fn test_bfs_two_node_direct() {
        let topology = NetworkTopology::new_linear(2, 4, 10.0, 0.2);
        let path = find_shortest_path(&topology, 0, 1).unwrap();
        assert_eq!(path, vec![0, 1]);
    }

    #[test]
    fn test_bfs_three_node_linear() {
        let topology = NetworkTopology::new_linear(3, 4, 10.0, 0.2);
        let path = find_shortest_path(&topology, 0, 2).unwrap();
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn test_bfs_four_node_linear() {
        let topology = NetworkTopology::new_linear(4, 4, 10.0, 0.2);
        let path = find_shortest_path(&topology, 0, 3).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_bfs_same_node_returns_singleton() {
        let topology = NetworkTopology::new_linear(3, 4, 10.0, 0.2);
        let path = find_shortest_path(&topology, 1, 1).unwrap();
        assert_eq!(path, vec![1]);
    }

    #[test]
    fn test_bfs_invalid_node_returns_none() {
        let topology = NetworkTopology::new_linear(3, 4, 10.0, 0.2);
        assert!(find_shortest_path(&topology, 0, 99).is_none());
        assert!(find_shortest_path(&topology, 99, 0).is_none());
    }

    #[test]
    fn test_bfs_disconnected_returns_none() {
        // Two isolated nodes with no channel between them
        let mut topology = NetworkTopology::new_custom();
        topology.add_node(crate::network::QuantumNode::new(0, 4)).unwrap();
        topology.add_node(crate::network::QuantumNode::new(1, 4)).unwrap();
        // No channel added
        assert!(find_shortest_path(&topology, 0, 1).is_none());
    }

    #[test]
    fn test_bfs_star_topology_peripheral_to_peripheral() {
        // Star: node 0 is center, nodes 1-4 are periphery
        let topology = NetworkTopology::new_star(5, 4, 10.0, 0.2);
        // Peripheral-to-peripheral must go through center (0)
        let path = find_shortest_path(&topology, 1, 3).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], 1);
        assert_eq!(path[1], 0); // center
        assert_eq!(path[2], 3);
    }

    #[test]
    fn test_bfs_star_topology_center_to_peripheral() {
        let topology = NetworkTopology::new_star(4, 4, 10.0, 0.2);
        let path = find_shortest_path(&topology, 0, 2).unwrap();
        assert_eq!(path, vec![0, 2]);
    }

    #[test]
    fn test_bfs_mesh_finds_direct_link() {
        // Full mesh: every node connected to every other
        let topology = NetworkTopology::new_mesh(4, 4, 10.0, 0.2);
        // In a full mesh, any two nodes are directly connected
        let path = find_shortest_path(&topology, 0, 3).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], 0);
        assert_eq!(path[1], 3);
    }

    #[test]
    fn test_bfs_path_length_equals_hop_count_plus_one() {
        let topology = NetworkTopology::new_linear(5, 4, 10.0, 0.2);
        let path = find_shortest_path(&topology, 0, 4).unwrap();
        assert_eq!(path.len(), 5); // 4 hops = 5 nodes
    }

    #[test]
    fn test_repeaters_direct_link() {
        let topology = NetworkTopology::new_linear(2, 4, 10.0, 0.2);
        let reps = repeaters_on_path(&topology, 0, 1).unwrap();
        assert!(reps.is_empty());
    }

    #[test]
    fn test_repeaters_three_node() {
        let topology = NetworkTopology::new_linear(3, 4, 10.0, 0.2);
        let reps = repeaters_on_path(&topology, 0, 2).unwrap();
        assert_eq!(reps, vec![1]);
    }

    #[test]
    fn test_repeaters_four_node() {
        let topology = NetworkTopology::new_linear(4, 4, 10.0, 0.2);
        let reps = repeaters_on_path(&topology, 0, 3).unwrap();
        assert_eq!(reps, vec![1, 2]);
    }
}
