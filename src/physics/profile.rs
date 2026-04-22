use serde::{Deserialize, Serialize};

/// Hardware platform parameters for a quantum network node.
///
/// A `PhysicsProfile` captures all platform-specific constants that differ
/// between hardware implementations (Erbium, NV centers, trapped ions, etc.).
/// Pass a profile to `BarrettKokProtocol::from_profile` and
/// `EntanglementSwappingProtocol::from_profile` to instantiate protocols
/// tuned to a specific hardware platform.
///
/// All fields are public so profiles can be constructed inline or
/// deserialized from TOML `[hardware]` sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsProfile {
    /// Human-readable name for this platform (e.g. "Erbium-167")
    pub name: String,

    /// T2 coherence time in milliseconds.
    /// Sets the exponential fidelity-decay timescale for stored qubits.
    pub coherence_time_ms: f64,

    /// Photon-memory coupling efficiency: probability that a memory qubit
    /// successfully emits a photon on a single attempt (0.0–1.0).
    pub memory_efficiency: f64,

    /// Initial state fidelity immediately after a successful generation event.
    /// Accounts for gate errors and state-preparation imperfections.
    pub initial_fidelity: f64,

    /// Single-photon detector efficiency (0.0–1.0).
    pub detector_efficiency: f64,

    /// Detector dark count rate: probability of a false click per detection
    /// window even when no photon arrives.
    pub dark_count_rate: f64,

    /// BSM efficiency for heralded entanglement generation (beam-splitter
    /// measurement at the midpoint). Linear optics: 0.5; enhanced: up to 1.0.
    pub generation_bsm_efficiency: f64,

    /// Local BSM efficiency at repeater nodes performing entanglement swapping.
    /// Typically higher than the generation BSM because it is done locally
    /// with direct qubit access rather than optical interference.
    pub swap_bsm_efficiency: f64,

    /// Multiplicative fidelity penalty from local gate imperfections (1.0 = ideal).
    /// Applied in the Werner-state swap formula:
    ///   F_out = [F_AB·F_BC + (1-F_AB)(1-F_BC)/9] × gate_fidelity
    pub gate_fidelity: f64,
}

impl PhysicsProfile {
    /// Erbium-167 (¹⁶⁷Er³⁺) cavity-QED profile.
    ///
    /// Parameters from the Er³⁺ cavity-QED model used as QComNetSim's
    /// baseline: tc = 1.3 s, C = 500, γ = 14 Hz, γ* = 32 Hz.
    /// Generation parameters match SeQUeNCe's default memory model.
    pub fn erbium() -> Self {
        PhysicsProfile {
            name: "Erbium-167".into(),
            coherence_time_ms: 1_300.0,      // 1.3 s T2 time
            memory_efficiency: 0.90,          // SeQUeNCe memory parameter
            initial_fidelity: 0.95,           // post-generation state fidelity
            detector_efficiency: 0.90,        // superconducting nanowire detectors
            dark_count_rate: 0.01,            // 1% false-click rate
            generation_bsm_efficiency: 0.50,  // linear-optics beam splitter
            swap_bsm_efficiency: 0.90,        // local cavity-mediated BSM
            gate_fidelity: 0.98,              // cavity-QED gate fidelity
        }
    }

    /// Nitrogen-vacancy (NV) center in diamond profile.
    ///
    /// Uses the electron-spin qubit as the memory qubit.
    /// NV centers have excellent local gate fidelities but much shorter
    /// coherence times and lower photon-collection efficiency than Er³⁺.
    /// Representative values from recent NV-center network experiments.
    pub fn nv_center() -> Self {
        PhysicsProfile {
            name: "NV-center".into(),
            coherence_time_ms: 10.0,          // ~10 ms electron-spin T2
            memory_efficiency: 0.06,           // ~6% photon collection (low-NA)
            initial_fidelity: 0.92,            // lower due to spin-photon entanglement errors
            detector_efficiency: 0.85,         // silicon-vacancy / SNSPD detectors
            dark_count_rate: 0.005,            // 0.5% dark count rate
            generation_bsm_efficiency: 0.50,   // linear-optics beam splitter
            swap_bsm_efficiency: 0.98,         // microwave-driven local BSM
            gate_fidelity: 0.995,              // high-fidelity microwave gates
        }
    }

    /// Ideal (noiseless) profile for unit tests and baselines.
    ///
    /// All efficiencies are 1.0, no dark counts, infinite coherence.
    /// Matches `BarrettKokProtocol::sequence_parameters()` when used with
    /// zero-loss channels.
    pub fn ideal() -> Self {
        PhysicsProfile {
            name: "Ideal".into(),
            coherence_time_ms: 1_000_000.0,
            memory_efficiency: 1.0,
            initial_fidelity: 1.0,
            detector_efficiency: 1.0,
            dark_count_rate: 0.0,
            generation_bsm_efficiency: 1.0,
            swap_bsm_efficiency: 1.0,
            gate_fidelity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erbium_fields_in_range() {
        let p = PhysicsProfile::erbium();
        assert!(p.coherence_time_ms > 0.0);
        assert!(p.memory_efficiency > 0.0 && p.memory_efficiency <= 1.0);
        assert!(p.initial_fidelity > 0.5 && p.initial_fidelity <= 1.0);
        assert!(p.detector_efficiency > 0.0 && p.detector_efficiency <= 1.0);
        assert!(p.gate_fidelity > 0.0 && p.gate_fidelity <= 1.0);
    }

    #[test]
    fn test_nv_center_shorter_coherence_than_erbium() {
        let er = PhysicsProfile::erbium();
        let nv = PhysicsProfile::nv_center();
        assert!(
            nv.coherence_time_ms < er.coherence_time_ms,
            "NV coherence ({} ms) should be shorter than Er ({} ms)",
            nv.coherence_time_ms,
            er.coherence_time_ms
        );
    }

    #[test]
    fn test_ideal_profile_all_perfect() {
        let p = PhysicsProfile::ideal();
        assert_eq!(p.memory_efficiency, 1.0);
        assert_eq!(p.initial_fidelity, 1.0);
        assert_eq!(p.detector_efficiency, 1.0);
        assert_eq!(p.dark_count_rate, 0.0);
        assert_eq!(p.gate_fidelity, 1.0);
    }

    #[test]
    fn test_profile_clone() {
        let p = PhysicsProfile::erbium();
        let q = p.clone();
        assert_eq!(p.name, q.name);
        assert_eq!(p.coherence_time_ms, q.coherence_time_ms);
    }
}
