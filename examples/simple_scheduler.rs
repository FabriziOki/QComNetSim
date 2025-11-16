use QComNetSim::simulation::{Event, EventScheduler, EventType};

fn main() {
    let mut scheduler = EventScheduler::new();

    println!("QComNetSim - Event Scheduler Demo\n");

    // Schedule some events
    scheduler.schedule(Event::new(0.0, EventType::EntanglementGeneration, 0));
    scheduler.schedule(Event::new(0.5, EventType::EntanglementGeneration, 1));
    scheduler.schedule(Event::new(1.0, EventType::EntanglementSwapping, 0));
    scheduler.schedule(Event::new(1.5, EventType::Measurement, 1));

    println!("Processing {} events:\n", scheduler.pending_events());

    while let Some(event) = scheduler.next_event() {
        println!(
            "Time {:.2}s: {:?} at node {}",
            event.time, event.event_type, event.node_id
        );
    }
}
