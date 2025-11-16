use super::event::{Event, EventType};
use std::collections::BinaryHeap;

/// Discrete-event scheduler for quantum network simulation
pub struct EventScheduler {
    /// Priority queue of events, ordered by time
    event_queue: BinaryHeap<Event>,
    /// Current simulation time
    current_time: f64,
}

impl EventScheduler {
    pub fn new() -> Self {
        EventScheduler {
            event_queue: BinaryHeap::new(),
            current_time: 0.0,
        }
    }

    /// Schedule a new event
    pub fn schedule(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    /// Get the next event (removes it from queue)
    pub fn next_event(&mut self) -> Option<Event> {
        if let Some(event) = self.event_queue.pop() {
            self.current_time = event.time;
            Some(event)
        } else {
            None
        }
    }

    /// Peek at next event without removing it
    pub fn peek_next(&self) -> Option<&Event> {
        self.event_queue.peek()
    }

    /// Get current simulation time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Get number of pending events
    pub fn pending_events(&self) -> usize {
        self.event_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_ordering() {
        let mut scheduler = EventScheduler::new();

        // Schedule events out of order
        scheduler.schedule(Event::new(3.0, EventType::Measurement, 0));
        scheduler.schedule(Event::new(1.0, EventType::EntanglementGeneration, 0));
        scheduler.schedule(Event::new(2.0, EventType::EntanglementSwapping, 0));

        // Events should come out in time order
        assert_eq!(scheduler.next_event().unwrap().time, 1.0);
        assert_eq!(scheduler.next_event().unwrap().time, 2.0);
        assert_eq!(scheduler.next_event().unwrap().time, 3.0);
    }

    #[test]
    fn test_current_time() {
        let mut scheduler = EventScheduler::new();
        assert_eq!(scheduler.current_time(), 0.0);

        scheduler.schedule(Event::new(5.0, EventType::Measurement, 0));
        scheduler.next_event();
        assert_eq!(scheduler.current_time(), 5.0);
    }
}
