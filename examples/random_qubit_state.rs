use qcomnetsim::quantum::*;

fn main() {
    println!("QComNetSim - Random Qubit States\n");
    let mut q = Qubit::new_random();
    println!(
        "Starting state: P(0)={:.3}, P(1)={:.3}",
        q.prob_zero(),
        q.prob_one()
    );
    pauli_x(&mut q);
    println!(
        "After X gate:       P(0)={:.3}, P(1)={:.3}",
        q.prob_zero(),
        q.prob_one()
    );
    pauli_y(&mut q);
    println!(
        "After Y gate:       P(0)={:.3}, P(1)={:.3}",
        q.prob_zero(),
        q.prob_one()
    );
    pauli_z(&mut q);
    print!(
        "After Z gate:       P(0)={:.3}, P(1)={:.3}",
        q.prob_zero(),
        q.prob_one()
    );
    print!(" -> Probabilities unchanged but phase flipped!\n");
}
