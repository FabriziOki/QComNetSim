use ndarray::{Array1, Array2};
use num_complex::Complex64;

/// A single qubit state represented as a state vector
#[derive(Debug, Clone)]
pub struct Qubit {
    /// State vector: [α, β] for α|0⟩ + β|1⟩
    pub state: Array1<Complex64>,
}

impl Qubit {
    /// Create a qubit in |0⟩ state (Computational basis)
    pub fn new_zero() -> Self {
        Qubit {
            state: Array1::from_vec(vec![
                Complex64::new(1.0, 0.0), // |0⟩
                Complex64::new(0.0, 0.0), // |1⟩
            ]),
        }
    }

    /// Create a qubit in |1⟩ state (Computational basis)
    pub fn new_one() -> Self {
        Qubit {
            state: Array1::from_vec(vec![
                Complex64::new(0.0, 0.0), // |0⟩
                Complex64::new(1.0, 0.0), // |1⟩
            ]),
        }
    }

    /// Create a qubit in |+⟩ state (Hadamard basos)
    /// |+⟩ = (|0⟩ + |1⟩)/√2
    pub fn new_plus() -> Self {
        let factor = 1.0 / (2.0_f64).sqrt();
        Qubit {
            state: Array1::from_vec(vec![
                Complex64::new(factor, 0.0),
                Complex64::new(factor, 0.0),
            ]),
        }
    }

    /// Create a qubit in |−⟩ state (Hadamard basis)
    /// |−⟩ = (|0⟩ - |1⟩)/√2
    pub fn new_minus() -> Self {
        let factor = 1.0 / (2.0_f64).sqrt();
        Qubit {
            state: Array1::from_vec(vec![
                Complex64::new(factor, 0.0),  //  1/√2 |0⟩
                Complex64::new(-factor, 0.0), // -1/√2 |1⟩
            ]),
        }
    }

    /// Create a qubit in |i+⟩ state (Circular basis)
    /// |i+⟩ = (|0⟩ + i|1⟩)/√2
    pub fn new_iplus() -> Self {
        let factor = 1.0 / (2.0_f64).sqrt();
        Qubit {
            state: Array1::from_vec(vec![
                Complex64::new(factor, 0.0), // 1/√2 |0⟩
                Complex64::new(0.0, factor), // i/√2 |1⟩
            ]),
        }
    }

    /// Create a qubit in |i−⟩ state (Circular basis)
    /// |i−⟩ = (|0⟩ - i|1⟩)/√2
    pub fn new_iminus() -> Self {
        let factor = 1.0 / (2.0_f64).sqrt();
        Qubit {
            state: Array1::from_vec(vec![
                Complex64::new(factor, 0.0),  // 1/√2 |0⟩
                Complex64::new(0.0, -factor), // -i/√2 |1⟩
            ]),
        }
    }

    /// Create a custom qubit state (will normalize automatically)
    pub fn new_custom(alpha: Complex64, beta: Complex64) -> Self {
        let norm = (alpha.norm_sqr() + beta.norm_sqr()).sqrt();
        Qubit {
            state: Array1::from_vec(vec![alpha / norm, beta / norm]),
        }
    }

    /// Get probability of measuring |0⟩
    pub fn prob_zero(&self) -> f64 {
        self.state[0].norm_sqr()
    }

    /// Get probability of measuring |1⟩
    pub fn prob_one(&self) -> f64 {
        self.state[1].norm_sqr()
    }

    /// Check if state is normalized (should always be ~1.0)
    pub fn is_normalized(&self) -> bool {
        let norm = self.state[0].norm_sqr() + self.state[1].norm_sqr();
        (norm - 1.0).abs() < 1e-10
    }
}

/// Two-qubit state for entangled pairs
#[derive(Debug, Clone)]
pub struct TwoQubitState {
    /// State vector of size 4: [|00⟩, |01⟩, |10⟩, |11⟩]
    pub state: Array1<Complex64>,
}

impl TwoQubitState {
    /// Create |00⟩ state
    pub fn new_zero_zero() -> Self {
        TwoQubitState {
            state: Array1::from_vec(vec![
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ]),
        }
    }

    /// Create Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2
    pub fn new_bell_phi_plus() -> Self {
        let factor = 1.0 / (2.0_f64).sqrt();
        TwoQubitState {
            state: Array1::from_vec(vec![
                Complex64::new(factor, 0.0), // |00⟩
                Complex64::new(0.0, 0.0),    // |01⟩
                Complex64::new(0.0, 0.0),    // |10⟩
                Complex64::new(factor, 0.0), // |11⟩
            ]),
        }
    }

    /// Calculate fidelity with another two-qubit state
    /// F = |⟨ψ|φ⟩|²
    pub fn fidelity(&self, other: &TwoQubitState) -> f64 {
        let mut inner_product = Complex64::new(0.0, 0.0);
        for i in 0..4 {
            inner_product += self.state[i].conj() * other.state[i];
        }
        inner_product.norm_sqr()
    }

    /// Check if normalized
    pub fn is_normalized(&self) -> bool {
        let norm: f64 = self.state.iter().map(|c| c.norm_sqr()).sum();
        (norm - 1.0).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qubit_creation() {
        let q0 = Qubit::new_zero();
        assert!(q0.is_normalized());
        assert!((q0.prob_zero() - 1.0).abs() < 1e-10);
        assert!(q0.prob_one().abs() < 1e-10);

        let q_plus = Qubit::new_plus();
        assert!(q_plus.is_normalized());
        assert!((q_plus.prob_zero() - 0.5).abs() < 1e-10);
        assert!((q_plus.prob_one() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_bell_state() {
        let bell = TwoQubitState::new_bell_phi_plus();
        assert!(bell.is_normalized());

        // Bell state should have perfect fidelity with itself
        assert!((bell.fidelity(&bell) - 1.0).abs() < 1e-10);
    }
}
