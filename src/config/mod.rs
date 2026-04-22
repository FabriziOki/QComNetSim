use crate::network::topology::NetworkTopology;
use crate::physics::PhysicsProfile;
use crate::protocols::barrett_kok::BarrettKokProtocol;
use crate::protocols::swapping::EntanglementSwappingProtocol;
use crate::simulation::SimulationConfig;
use serde::Deserialize;
use std::fs;
use std::path::Path;

// ============================================================
// TOP-LEVEL CONFIG
// ============================================================

/// Full simulation config as read from a TOML file.
///
/// ```toml
/// [topology]
/// type = "linear"
/// nodes = 3
/// distance_km = 10.0
/// attenuation_db_per_km = 0.2
/// memory_slots = 8
/// source = 0
/// dest = 2
///
/// [hardware]
/// profile = "erbium"
///
/// [simulation]
/// target_pairs = 100
/// max_time_ms = 500000.0
/// ```

#[derive(Debug, Deserialize)]
pub struct SimulationToml {
    pub topology: TopologyConfig,
    pub hardware: HardwareConfig,
    pub simulation: SimulationSection,
}

impl SimulationToml {
    /// Load and parse a TOML file from disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {}", path.as_ref().display(), e))?;
        toml::from_str(&raw)
            .map_err(|e| format!("TOML parse error in {}: {}", path.as_ref().display(), e))
    }

    /// Resolve all sections into the runtime objects the engine needs.
    pub fn resolve(&self) -> Result<ResolvedConfig, String> {
        let profile = self.hardware.resolve_profile()?;
        let topology = self.topology.build_topology()?;
        let path =
            crate::network::find_shortest_path(&topology, self.topology.source, self.topology.dest)
                .ok_or_else(|| {
                    format!(
                        "No path from node {} to node {} in the topology",
                        self.topology.source, self.topology.dest
                    )
                })?;
        let gen_protocol = BarrettKokProtocol::from_profile(&profile);
        let swap_protocol = EntanglementSwappingProtocol::from_profile(&profile);
        let sim_config = self.simulation.to_sim_config(&profile);

        Ok(ResolvedConfig {
            topology,
            path,
            gen_protocol,
            swap_protocol,
            sim_config,
            profile,
        })
    }
}

// ============================================================
// RESOLVED CONFIG
// ============================================================

/// All runtime objects produced by resolving a `SimulationToml`.
/// Ready to be handed directly to `Simulation::new`.
pub struct ResolvedConfig {
    pub topology: NetworkTopology,
    pub path: Vec<usize>,
    pub gen_protocol: BarrettKokProtocol,
    pub swap_protocol: EntanglementSwappingProtocol,
    pub sim_config: SimulationConfig,
    pub profile: PhysicsProfile,
}

// ============================================================
// [topology]
// ============================================================

/// Supported topology shapes.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TopologyKind {
    Linear,
    Star,
    Mesh,
}

/// `[topology]` section.
#[derive(Debug, Deserialize)]
pub struct TopologyConfig {
    /// Topology shape: "linear", "star", or "mesh"
    #[serde(rename = "type")]
    pub kind: TopologyKind,
    /// Number of nodes
    pub nodes: usize,
    /// Link distance in km (applied uniformly to all links)
    pub distance_km: f64,
    /// Fiber attenuation in dB/km
    pub attenuation_db_per_km: f64,
    /// Qubit memory capacity per node
    pub memory_slots: usize,
    /// Source node ID for routing
    pub source: usize,
    /// Destination node ID for routing
    pub dest: usize,
}

impl TopologyConfig {
    fn build_topology(&self) -> Result<NetworkTopology, String> {
        if self.nodes < 2 {
            return Err(format!("nodes must be >= 2, got {}", self.nodes));
        }
        Ok(match self.kind {
            TopologyKind::Linear => NetworkTopology::new_linear(
                self.nodes,
                self.memory_slots,
                self.distance_km,
                self.attenuation_db_per_km,
            ),
            TopologyKind::Star => NetworkTopology::new_star(
                self.nodes,
                self.memory_slots,
                self.distance_km,
                self.attenuation_db_per_km,
            ),
            TopologyKind::Mesh => NetworkTopology::new_mesh(
                self.nodes,
                self.memory_slots,
                self.distance_km,
                self.attenuation_db_per_km,
            ),
        })
    }
}

// ============================================================
// [hardware]
// ============================================================

/// `[hardware]` section.
///
/// `profile` names a built-in preset ("erbium", "nv_center", "ideal").
/// Any additional fields override the corresponding preset value,
/// so you can write `profile = "erbium"` and then add
/// `coherence_time_ms = 500.0` to tweak just that one parameter.
/// If `profile` is omitted, "ideal" is used as the base.
#[derive(Debug, Deserialize)]
pub struct HardwareConfig {
    pub profile: Option<String>,

    // Optional per-field overrides — each one shadows the preset value
    pub coherence_time_ms: Option<f64>,
    pub memory_efficiency: Option<f64>,
    pub initial_fidelity: Option<f64>,
    pub detector_efficiency: Option<f64>,
    pub dark_count_rate: Option<f64>,
    pub generation_bsm_efficiency: Option<f64>,
    pub swap_bsm_efficiency: Option<f64>,
    pub gate_fidelity: Option<f64>,
}

impl HardwareConfig {
    /// Load a named preset then apply any inline field overrides.
    pub fn resolve_profile(&self) -> Result<PhysicsProfile, String> {
        let mut p = match self.profile.as_deref() {
            Some("erbium") => PhysicsProfile::erbium(),
            Some("nv_center") => PhysicsProfile::nv_center(),
            Some("ideal") => PhysicsProfile::ideal(),
            Some(other) => {
                return Err(format!(
                    "Unknown profile \"{other}\". Valid options: erbium, nv_center, ideal"
                ))
            }
            None => PhysicsProfile::ideal(),
        };

        if let Some(v) = self.coherence_time_ms {
            p.coherence_time_ms = v;
        }
        if let Some(v) = self.memory_efficiency {
            p.memory_efficiency = v;
        }
        if let Some(v) = self.initial_fidelity {
            p.initial_fidelity = v;
        }
        if let Some(v) = self.detector_efficiency {
            p.detector_efficiency = v;
        }
        if let Some(v) = self.dark_count_rate {
            p.dark_count_rate = v;
        }
        if let Some(v) = self.generation_bsm_efficiency {
            p.generation_bsm_efficiency = v;
        }
        if let Some(v) = self.swap_bsm_efficiency {
            p.swap_bsm_efficiency = v;
        }
        if let Some(v) = self.gate_fidelity {
            p.gate_fidelity = v;
        }

        Ok(p)
    }
}

// ============================================================
// [simulation]
// ============================================================

/// `[simulation]` section.
///
/// `target_pairs` and `max_time_ms` are required.
/// All other fields fall back to `SimulationConfig::from_profile` defaults
/// when omitted.
#[derive(Debug, Deserialize)]
pub struct SimulationSection {
    pub target_pairs: usize,
    pub max_time_ms: f64,
    pub retry_interval_ms: Option<f64>,
    pub classical_delay_ms: Option<f64>,
    pub fidelity_threshold: Option<f64>,
    pub decoherence_check_interval_ms: Option<f64>,
}

impl SimulationSection {
    fn to_sim_config(&self, profile: &PhysicsProfile) -> SimulationConfig {
        let mut cfg = SimulationConfig::from_profile(profile);
        cfg.target_pairs = self.target_pairs;
        cfg.max_time_ms = self.max_time_ms;
        if let Some(v) = self.retry_interval_ms {
            cfg.retry_interval_ms = v;
        }
        if let Some(v) = self.classical_delay_ms {
            cfg.classical_delay_ms = v;
        }
        if let Some(v) = self.fidelity_threshold {
            cfg.fidelity_threshold = v;
        }
        if let Some(v) = self.decoherence_check_interval_ms {
            cfg.decoherence_check_interval_ms = v;
        }
        cfg
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml: &str) -> SimulationToml {
        toml::from_str(toml).expect("parse failed")
    }

    #[test]
    fn test_minimal_linear_config() {
        let cfg = parse(
            r#"
            [topology]
            type = "linear"
            nodes = 3
            distance_km = 10.0
            attenuation_db_per_km = 0.2
            memory_slots = 8
            source = 0
            dest = 2

            [hardware]
            profile = "erbium"

            [simulation]
            target_pairs = 100
            max_time_ms = 500000.0
        "#,
        );
        assert_eq!(cfg.topology.kind, TopologyKind::Linear);
        assert_eq!(cfg.topology.nodes, 3);
        assert_eq!(cfg.simulation.target_pairs, 100);
    }

    #[test]
    fn test_profile_resolves_to_erbium() {
        let cfg = parse(
            r#"
            [topology]
            type = "linear"
            nodes = 2
            distance_km = 5.0
            attenuation_db_per_km = 0.2
            memory_slots = 4
            source = 0
            dest = 1

            [hardware]
            profile = "erbium"

            [simulation]
            target_pairs = 10
            max_time_ms = 10000.0
        "#,
        );
        let profile = cfg.hardware.resolve_profile().unwrap();
        assert_eq!(profile.name, "Erbium-167");
        assert!((profile.coherence_time_ms - 1300.0).abs() < 1e-9);
    }

    #[test]
    fn test_field_override_on_preset() {
        let cfg = parse(
            r#"
            [topology]
            type = "linear"
            nodes = 2
            distance_km = 5.0
            attenuation_db_per_km = 0.2
            memory_slots = 4
            source = 0
            dest = 1

            [hardware]
            profile = "erbium"
            coherence_time_ms = 500.0

            [simulation]
            target_pairs = 10
            max_time_ms = 10000.0
        "#,
        );
        let profile = cfg.hardware.resolve_profile().unwrap();
        // Override applied
        assert!((profile.coherence_time_ms - 500.0).abs() < 1e-9);
        // Other Erbium fields unchanged
        assert!((profile.memory_efficiency - 0.90).abs() < 1e-9);
    }

    #[test]
    fn test_unknown_profile_returns_err() {
        let cfg = parse(
            r#"
            [topology]
            type = "linear"
            nodes = 2
            distance_km = 5.0
            attenuation_db_per_km = 0.2
            memory_slots = 4
            source = 0
            dest = 1

            [hardware]
            profile = "unicorn"

            [simulation]
            target_pairs = 10
            max_time_ms = 10000.0
        "#,
        );
        assert!(cfg.hardware.resolve_profile().is_err());
    }

    #[test]
    fn test_resolve_builds_valid_path() {
        let cfg = parse(
            r#"
            [topology]
            type = "linear"
            nodes = 3
            distance_km = 10.0
            attenuation_db_per_km = 0.2
            memory_slots = 8
            source = 0
            dest = 2

            [hardware]
            profile = "erbium"

            [simulation]
            target_pairs = 5
            max_time_ms = 100000.0
        "#,
        );
        let resolved = cfg.resolve().unwrap();
        assert_eq!(resolved.path, vec![0, 1, 2]);
    }

    #[test]
    fn test_star_topology_parses() {
        let cfg = parse(
            r#"
            [topology]
            type = "star"
            nodes = 5
            distance_km = 10.0
            attenuation_db_per_km = 0.2
            memory_slots = 8
            source = 1
            dest = 3

            [hardware]
            profile = "ideal"

            [simulation]
            target_pairs = 5
            max_time_ms = 100000.0
        "#,
        );
        assert_eq!(cfg.topology.kind, TopologyKind::Star);
        let resolved = cfg.resolve().unwrap();
        // Star routes periphery→periphery through center (node 0)
        assert_eq!(resolved.path, vec![1, 0, 3]);
    }
}
