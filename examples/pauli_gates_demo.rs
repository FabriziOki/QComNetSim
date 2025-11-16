use QComNetSim::quantum::{hadamard, pauli_x, pauli_y, pauli_z, Qubit};

fn main() {
    println!("QComNetSim - Pauli Gates Demo\n");

    // Pauli-X (NOT gate)
    println!("=== Pauli-X Gate ===");
    let mut q = Qubit::new_zero();
    println!(
        "Starting state |0⟩: P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    pauli_x(&mut q);
    println!(
        "After X gate:       P(0)={:.2}, P(1)={:.2}\n",
        q.prob_zero(),
        q.prob_one()
    );

    // Pauli-Y
    println!("=== Pauli-Y Gate ===");
    let mut q = Qubit::new_zero();
    println!(
        "Starting state |0⟩: P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    pauli_y(&mut q);
    println!(
        "After Y gate:       P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    println!("State: {:.3} + {:.3}i", q.state[1].re, q.state[1].im);
    println!("(Should be i|1⟩)\n");

    // Pauli-Z
    println!("=== Pauli-Z Gate ===");
    let mut q = Qubit::new_plus();
    println!(
        "Starting state |+⟩: P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    pauli_z(&mut q);
    println!(
        "After Z gate:       P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    println!("(|+⟩ → |−⟩, probabilities unchanged but phase flipped)\n");

    // Hadamard
    println!("=== Hadamard Gate ===");
    let mut q = Qubit::new_zero();
    println!(
        "Starting state |0⟩: P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    hadamard(&mut q);
    println!(
        "After H gate:       P(0)={:.2}, P(1)={:.2}",
        q.prob_zero(),
        q.prob_one()
    );
    println!("(Created superposition!)\n");

    // Gate composition
    println!("=== Gate Composition ===");
    let mut q = Qubit::new_zero();
    println!("Starting: |0⟩");
    hadamard(&mut q);
    println!("After H:  |+⟩ (superposition)");
    pauli_z(&mut q);
    println!("After Z:  |−⟩ (phase flip)");
    hadamard(&mut q);
    println!("After H:  |1⟩");
    println!("Final: P(0)={:.2}, P(1)={:.2}", q.prob_zero(), q.prob_one());
    println!("(HZH|0⟩ = |1⟩)");
}
