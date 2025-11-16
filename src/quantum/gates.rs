use super::state::Qubit;
use ndarray::Array2;
use num_complex::Complex64;

/// Pauli-X gate (NOT gate)
/// Matrix: [[0, 1],
///          [1, 0]]
/// Effect: |0⟩ ↔ |1⟩
pub fn pauli_x(qubit: &mut Qubit) {
    let new_state = ndarray::array![
        qubit.state[1], // Swap: new |0⟩ = old |1⟩
        qubit.state[0], // Swap: new |1⟩ = old |0⟩
    ];
    qubit.state = new_state;
}

/// Pauli-Y gate
/// Matrix: [[0, -i],
///          [i,  0]]
/// Effect: |0⟩ → i|1⟩, |1⟩ → -i|0⟩
pub fn pauli_y(qubit: &mut Qubit) {
    let new_state = ndarray::array![
        Complex64::new(0.0, -1.0) * qubit.state[1], // -i * |1⟩
        Complex64::new(0.0, 1.0) * qubit.state[0],  //  i * |0⟩
    ];
    qubit.state = new_state;
}

/// Pauli-Z gate (Phase flip)
/// Matrix: [[1,  0],
///          [0, -1]]
/// Effect: |0⟩ → |0⟩, |1⟩ → -|1⟩
pub fn pauli_z(qubit: &mut Qubit) {
    let new_state = ndarray::array![
        qubit.state[0],  // |0⟩ unchanged
        -qubit.state[1], // |1⟩ → -|1⟩
    ];
    qubit.state = new_state;
}

/// Hadamard gate (creates superposition)
/// Matrix: (1/√2) * [[1,  1],
///                    [1, -1]]
/// Effect: |0⟩ → |+⟩, |1⟩ → |−⟩
pub fn hadamard(qubit: &mut Qubit) {
    let factor = 1.0 / (2.0_f64).sqrt();
    let new_state = ndarray::array![
        Complex64::new(factor, 0.0) * (qubit.state[0] + qubit.state[1]),
        Complex64::new(factor, 0.0) * (qubit.state[0] - qubit.state[1]),
    ];
    qubit.state = new_state;
}

/// Identity gate (does nothing - useful for testing)
/// Matrix: [[1, 0],
///          [0, 1]]
pub fn identity(qubit: &mut Qubit) {
    // Do nothing - state remains unchanged
    // Useful for: testing, placeholder in circuits, explicit "wait"
}

/// Generic single-qubit gate application
/// Applies a 2x2 unitary matrix to the qubit state
pub fn apply_gate(qubit: &mut Qubit, gate_matrix: &Array2<Complex64>) {
    assert_eq!(gate_matrix.shape(), &[2, 2], "Gate must be 2x2 matrix");

    let new_state = gate_matrix.dot(&qubit.state);
    qubit.state = new_state;
}

/// Helper function to create Pauli-X matrix (for testing/verification)
pub fn get_pauli_x_matrix() -> Array2<Complex64> {
    Array2::from_shape_vec(
        (2, 2),
        vec![
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
        ],
    )
    .unwrap()
}

/// Helper function to create Pauli-Y matrix
pub fn get_pauli_y_matrix() -> Array2<Complex64> {
    Array2::from_shape_vec(
        (2, 2),
        vec![
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, -1.0),
            Complex64::new(0.0, 1.0),
            Complex64::new(0.0, 0.0),
        ],
    )
    .unwrap()
}

/// Helper function to create Pauli-Z matrix
pub fn get_pauli_z_matrix() -> Array2<Complex64> {
    Array2::from_shape_vec(
        (2, 2),
        vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(-1.0, 0.0),
        ],
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pauli_x_on_zero() {
        let mut qubit = Qubit::new_zero();
        pauli_x(&mut qubit);

        // X|0⟩ = |1⟩
        assert!(qubit.is_normalized());
        assert!(qubit.prob_zero().abs() < 1e-10, "Should be |1⟩ state");
        assert!(
            (qubit.prob_one() - 1.0).abs() < 1e-10,
            "Should be |1⟩ state"
        );
    }

    #[test]
    fn test_pauli_x_on_one() {
        let mut qubit = Qubit::new_one();
        pauli_x(&mut qubit);

        // X|1⟩ = |0⟩
        assert!(qubit.is_normalized());
        assert!(
            (qubit.prob_zero() - 1.0).abs() < 1e-10,
            "Should be |0⟩ state"
        );
        assert!(qubit.prob_one().abs() < 1e-10, "Should be |0⟩ state");
    }

    #[test]
    fn test_pauli_x_twice_is_identity() {
        let mut qubit = Qubit::new_zero();
        let original_state = qubit.state.clone();

        pauli_x(&mut qubit);
        pauli_x(&mut qubit);

        // X² = I (applying X twice returns to original state)
        assert!((qubit.state[0] - original_state[0]).norm() < 1e-10);
        assert!((qubit.state[1] - original_state[1]).norm() < 1e-10);
    }

    #[test]
    fn test_pauli_y_on_zero() {
        let mut qubit = Qubit::new_zero();
        pauli_y(&mut qubit);

        // Y|0⟩ = i|1⟩
        assert!(qubit.is_normalized());
        assert!(qubit.prob_zero().abs() < 1e-10);
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);

        // Check phase: should be i|1⟩
        assert!(qubit.state[1].re.abs() < 1e-10, "Real part should be ~0");
        assert!(
            (qubit.state[1].im - 1.0).abs() < 1e-10,
            "Imaginary part should be 1"
        );
    }

    #[test]
    fn test_pauli_z_on_zero() {
        let mut qubit = Qubit::new_zero();
        let original_state = qubit.state.clone();

        pauli_z(&mut qubit);

        // Z|0⟩ = |0⟩ (unchanged)
        assert!((qubit.state[0] - original_state[0]).norm() < 1e-10);
        assert!((qubit.state[1] - original_state[1]).norm() < 1e-10);
    }

    #[test]
    fn test_pauli_z_on_one() {
        let mut qubit = Qubit::new_one();
        pauli_z(&mut qubit);

        // Z|1⟩ = -|1⟩
        assert!(qubit.is_normalized());
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);

        // Check for negative phase
        assert!((qubit.state[1].re + 1.0).abs() < 1e-10, "Should be -1");
    }

    #[test]
    fn test_pauli_z_on_plus() {
        let mut qubit = Qubit::new_plus();
        pauli_z(&mut qubit);

        // Z|+⟩ = |−⟩
        // Should still have 50-50 probabilities
        assert!(qubit.is_normalized());
        assert!((qubit.prob_zero() - 0.5).abs() < 1e-10);
        assert!((qubit.prob_one() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard_on_zero() {
        let mut qubit = Qubit::new_zero();
        hadamard(&mut qubit);

        // H|0⟩ = |+⟩
        assert!(qubit.is_normalized());
        assert!((qubit.prob_zero() - 0.5).abs() < 1e-10);
        assert!((qubit.prob_one() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard_twice_is_identity() {
        let mut qubit = Qubit::new_zero();
        let original_state = qubit.state.clone();

        hadamard(&mut qubit);
        hadamard(&mut qubit);

        // H² = I
        assert!((qubit.state[0] - original_state[0]).norm() < 1e-10);
        assert!((qubit.state[1] - original_state[1]).norm() < 1e-10);
    }

    #[test]
    fn test_all_gates_preserve_normalization() {
        let gates: Vec<fn(&mut Qubit)> = vec![pauli_x, pauli_y, pauli_z, hadamard, identity];

        for gate in gates {
            let mut qubit = Qubit::new_plus();
            gate(&mut qubit);
            assert!(qubit.is_normalized(), "Gate broke normalization!");
        }
    }

    #[test]
    fn test_generic_gate_application() {
        let mut qubit = Qubit::new_zero();
        let x_matrix = get_pauli_x_matrix();

        apply_gate(&mut qubit, &x_matrix);

        // Should behave like pauli_x
        assert!((qubit.prob_one() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_matrices_are_unitary() {
        let matrices = vec![
            get_pauli_x_matrix(),
            get_pauli_y_matrix(),
            get_pauli_z_matrix(),
        ];

        for matrix in matrices {
            // For a unitary matrix U: U†U = I
            let conjugate_transpose = matrix.t().mapv(|x| x.conj());
            let product = conjugate_transpose.dot(&matrix);

            // Check if product is identity
            assert!((product[[0, 0]].re - 1.0).abs() < 1e-10);
            assert!((product[[1, 1]].re - 1.0).abs() < 1e-10);
            assert!(product[[0, 1]].norm() < 1e-10);
            assert!(product[[1, 0]].norm() < 1e-10);
        }
    }
}
