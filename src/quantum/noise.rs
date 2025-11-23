use super::state::TwoQubitState;
use num_complex::Complex64;

/// Calculate fidelity after decoherence
///
/// Decoherence causes quantum states to lose their quantum properties over time
/// This is modeled as exponential decay of fidelity
pub fn fidelity_after_decoherence(
    initial_fidelity: f64,
    elapsed_time_ms: f64,
    coherence_time_ms: f64,
) -> f64 {
    let decay_factor = (-elapsed_time_ms / coherence_time_ms).exp();

    // Fidelity decays as: F(t) = F_0 * e^(-t/T_coh)
    initial_fidelity * decay_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fidelity_calculation() {
        let initial = 0.95;
        let elapsed = 100.0;
        let coherence_time = 100.0;

        let final_fidelity = fidelity_after_decoherence(initial, elapsed, coherence_time);

        // After one coherence time: F ≈ F_0 * e^(-1) ≈ 0.368 * F_0
        assert!((final_fidelity - initial * (1.0_f64.exp().recip())).abs() < 1e-10);
    }

    #[test]
    fn test_no_time_elapsed() {
        let fidelity = fidelity_after_decoherence(1.0, 0.0, 100.0);
        assert!((fidelity - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_long_decoherence() {
        let fidelity = fidelity_after_decoherence(1.0, 500.0, 100.0);
        let expected = (-5.0_f64).exp(); // ≈ 0.0067
        assert!((fidelity - expected).abs() < 1e-10);
        assert!(fidelity < 0.01);
    }
}
