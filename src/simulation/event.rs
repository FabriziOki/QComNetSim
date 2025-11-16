use std::cmp::Ordering;

/// Types of events that can occur in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Attempt to generate entanglement on a channel
    EntanglementGeneration,
    /// Perform entanglement swapping at a node
    EntanglementSwapping,
    /// Perform purification operation
    Purification,
    /// Qubit measurement
    Measurement,
    /// Memory decoherence event
    Decoherence,
}

/// A discrete event in the quantum network simulation
#[derive(Debug, Clone)]
pub struct Event {
    /// Time when this event should be processed (in seconds)
    pub time: f64,
    /// Type of event
    pub event_type: EventType,
    /// ID of the node where this event occurs
    pub node_id: usize,
    /// Optional: ID of another node involved (e.g., for channel events)
    pub target_node_id: Option<usize>,
    /// Optional: Channel or qubit ID
    pub resource_id: Option<usize>,
}

impl Event {
    pub fn new(time: f64, event_type: EventType, node_id: usize) -> Self {
        Event {
            time,
            event_type,
            node_id,
            target_node_id: None,
            resource_id: None,
        }
    }
}

// Make events orderable by time (needed for priority queue)
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap becomes a min-heap
        other.time.partial_cmp(&self.time).unwrap()
    }
}
