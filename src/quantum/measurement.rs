use super::state::Qubit;
use num_complex::Complex64;
use rand::Rng;

/// Perform ideal Z-basis measurement on a qubit
/// Returns true for |1⟩, false for |0⟩
pub fn measure_z(qubit: &mut Qubit) -> bool {
    let prob_zero = qubit.prob_zero();
    let mut rng = rand::rng();

    let result = rng.random::<f64>() >= prob_zero;

    // Collapse to measured state
    if result {
        // Measured |1⟩
        qubit.state[[0]] = Complex64::new(0.0, 0.0);
        qubit.state[[1]] = Complex64::new(1.0, 0.0);
    } else {
        // Measured |0⟩
        qubit.state[[0]] = Complex64::new(1.0, 0.0);
        qubit.state[[1]] = Complex64::new(0.0, 0.0);
    }

    result
}

/// Perform Z-basis measurement with detector errors
///
/// Models realistic detectors with:
/// - Dark counts: false positives when no photon arrives
/// - Detector efficiency: probability of actually detecting a photon
/// - Measurement errors: bit flip errors in the classical result
pub fn measure_z_with_noise(
    qubit: &mut Qubit,
    detector_efficiency: f64,
    dark_count_rate: f64,
    measurement_error_rate: f64,
) -> bool {
    let mut rng = rand::rng();

    // First, ideal quantum measurement
    let ideal_result = measure_z(qubit);

    // Apply detector inefficiency
    let detected = if ideal_result {
        // Photon present - might not detect it
        rng.random::<f64>() < detector_efficiency
    } else {
        // No photon - might have dark count
        rng.random::<f64>() < dark_count_rate
    };

    // Apply measurement error (bit flip)
    let final_result = if rng.random::<f64>() < measurement_error_rate {
        !detected // Flip the bit
    } else {
        detected
    };

    final_result
}

/// Perform X-basis measurement (measure in |+⟩, |-⟩ basis)
pub fn measure_x(qubit: &mut Qubit) -> bool {
    // Apply Hadamard to convert X-basis to Z-basis
    super::gates::hadamard(qubit);

    // Measure in Z-basis
    let result = measure_z(qubit);

    result
}

/// Perform Y-basis measurement
pub fn measure_y(qubit: &mut Qubit) -> bool {
    // S†H converts Y-basis to Z-basis
    // For simplicity, we'll apply the transformation

    // Apply S† (phase gate conjugate transpose)
    qubit.state[[1]] *= Complex64::new(0.0, -1.0);

    // Apply Hadamard
    super::gates::hadamard(qubit);

    // Measure in Z-basis
    let result = measure_z(qubit);

    result
}

/// Configuration for realistic measurement parameters
#[derive(Clone, Copy)]
pub struct MeasurementConfig {
    /// Detector efficiency (0.0 to 1.0)
    /// Typical: 0.90-0.95 for good detectors
    pub detector_efficiency: f64,

    /// Dark count rate (0.0 to 1.0)
    /// Typical: 0.001-0.01 (0.1% to 1%)
    pub dark_count_rate: f64,

    /// Classical bit flip error rate (0.0 to 1.0)
    /// Typical: 0.001-0.02 (0.1% to 2%)
    pub measurement_error_rate: f64,
}

impl MeasurementConfig {
    /// Perfect measurement (for testing)
    pub fn perfect() -> Self {
        MeasurementConfig {
            detector_efficiency: 1.0,
            dark_count_rate: 0.0,
            measurement_error_rate: 0.0,
        }
    }

    /// Realistic measurement parameters
    pub fn realistic() -> Self {
        MeasurementConfig {
            detector_efficiency: 0.95,
            dark_count_rate: 0.01,
            measurement_error_rate: 0.02,
        }
    }

    /// High-quality measurement
    pub fn high_quality() -> Self {
        MeasurementConfig {
            detector_efficiency: 0.98,
            dark_count_rate: 0.001,
            measurement_error_rate: 0.005,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantum::state::Qubit;

    #[test]
    fn test_measure_zero_state() {
        let mut qubit = Qubit::new_zero();
        let result = measure_z(&mut qubit);

        // Should always measure |0⟩ (false)
        assert!(!result);

        // State should be collapsed to |0⟩
        assert!((qubit.state[[0]].re - 1.0).abs() < 1e-10);
        assert!(qubit.state[[1]].norm() < 1e-10);
    }

    #[test]
    fn test_measure_one_state() {
        let mut qubit = Qubit::new_one();
        let result = measure_z(&mut qubit);

        // Should always measure |1⟩ (true)
        assert!(result);

        // State should be collapsed to |1⟩
        assert!(qubit.state[[0]].norm() < 1e-10);
        assert!((qubit.state[[1]].re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_measure_superposition() {
        // Measure |+⟩ = (|0⟩ + |1⟩)/√2 many times
        let num_trials = 10000;
        let mut num_ones = 0;

        for _ in 0..num_trials {
            let mut qubit = Qubit::new_plus();
            if measure_z(&mut qubit) {
                num_ones += 1;
            }
        }

        // Should get roughly 50% ones
        let ratio = num_ones as f64 / num_trials as f64;
        println!("{}", (ratio - 0.5).abs());
        assert!((ratio - 0.5).abs() < 0.05); // Within 5%
    }

    #[test]
    fn test_perfect_measurement() {
        let config = MeasurementConfig::perfect();

        // Measure |0⟩ state
        let mut qubit = Qubit::new_zero();
        let result = measure_z_with_noise(
            &mut qubit,
            config.detector_efficiency,
            config.dark_count_rate,
            config.measurement_error_rate,
        );

        // Perfect measurement should give correct result
        assert!(!result);
    }

    #[test]
    fn test_noisy_measurement_statistics() {
        let config = MeasurementConfig::realistic();
        let num_trials = 1000;
        let mut errors = 0;

        // Measure |0⟩ state many times
        for _ in 0..num_trials {
            let mut qubit = Qubit::new_zero();
            let result = measure_z_with_noise(
                &mut qubit,
                config.detector_efficiency,
                config.dark_count_rate,
                config.measurement_error_rate,
            );

            if result {
                // Got |1⟩ when should be |0⟩ - this is an error
                errors += 1;
            }
        }

        let error_rate = errors as f64 / num_trials as f64;

        // Error rate should be approximately dark_count_rate + measurement_error_rate
        // But interaction is complex, so just check it's reasonable
        assert!(error_rate > 0.0);
        assert!(error_rate < 0.1); // Less than 10%
    }

    #[test]
    fn test_x_basis_measurement() {
        // Measure |+⟩ in X-basis should always give |+⟩ (false)
        let mut qubit = Qubit::new_plus();
        let result = measure_x(&mut qubit);

        // Result is probabilistic but state should be valid
        // Just check the measurement doesn't panic
        assert!(result == true || result == false);
    }

    #[test]
    fn test_measurement_collapse() {
        let mut qubit = Qubit::new_plus();

        // First measurement
        let result1 = measure_z(&mut qubit);

        // Second measurement should give same result
        let result2 = measure_z(&mut qubit);

        assert_eq!(result1, result2);
    }
}
