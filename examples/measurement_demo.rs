use qcomnetsim::quantum::measurement::{measure_z, measure_z_with_noise, MeasurementConfig};
use qcomnetsim::quantum::state::Qubit;

fn main() {
    println!("QComNetSim - Measurement Operations Demo\n");

    // Perfect measurement
    println!("=== Perfect Measurement ===");
    let config = MeasurementConfig::perfect();

    let mut correct = 0;
    let trials = 10000;

    for _ in 0..trials {
        let mut qubit = Qubit::new_zero();
        let result = measure_z_with_noise(
            &mut qubit,
            config.detector_efficiency,
            config.dark_count_rate,
            config.measurement_error_rate,
        );
        if !result {
            correct += 1;
        }
    }

    println!("Measuring |0⟩ state {} times", trials);
    println!(
        "Correct results: {} ({:.1}%)\n",
        correct,
        100.0 * correct as f64 / trials as f64
    );

    // Realistic measurement
    println!("=== Realistic Measurement ===");
    let config = MeasurementConfig::realistic();
    println!("Detector efficiency: {}", config.detector_efficiency);
    println!("Dark count rate: {}", config.dark_count_rate);
    println!(
        "Measurement error rate: {}\n",
        config.measurement_error_rate
    );

    correct = 0;
    for _ in 0..trials {
        let mut qubit = Qubit::new_zero();
        let result = measure_z_with_noise(
            &mut qubit,
            config.detector_efficiency,
            config.dark_count_rate,
            config.measurement_error_rate,
        );
        if !result {
            correct += 1;
        }
    }

    println!("Measuring |0⟩ state {} times", trials);
    println!(
        "Correct results: {} ({:.1}%)",
        correct,
        100.0 * correct as f64 / trials as f64
    );
    println!(
        "Error rate: {:.1}%\n",
        100.0 * (trials - correct) as f64 / trials as f64
    );

    // Superposition measurement
    println!("=== Superposition Measurement ===");
    let mut ones = 0;
    for _ in 0..trials {
        let mut qubit = Qubit::new_plus();
        if measure_z(&mut qubit) {
            ones += 1;
        }
    }

    println!("Measuring |+⟩ state {} times", trials);
    println!(
        "|0⟩: {} ({:.1}%)",
        trials - ones,
        100.0 * (trials - ones) as f64 / trials as f64
    );
    println!(
        "|1⟩: {} ({:.1}%)",
        ones,
        100.0 * ones as f64 / trials as f64
    );
}
