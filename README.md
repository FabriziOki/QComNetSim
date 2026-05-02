<div align="center">
<img src="assets/name.svg" alt="QComNetSim Logo" width="1000"/>
</div>

> **Status**: GlobeCom 2026 submission — Spring 2026

A high-performance quantum network simulator written in Rust with an integrated cross-simulator validation engine.

---

## Overview

QComNetSim provides automated cross-simulator benchmarking by translating configurations, orchestrating concurrent execution, and performing statistical comparison across simulators. It addresses the validation gap in quantum network research: without cross-verification, it is difficult to determine whether result discrepancies reflect implementation bugs, valid modeling trade-offs, or incorrect physics assumptions.

**Key capabilities:**
- Multi-hop BFS routing with entanglement swapping over linear chains
- Configurable hardware physics profiles (Erbium, NV-center) via TOML
- Automated validation pipeline comparing QComNetSim against SeQUeNCe and SimQN
- ~45× wall-clock speedup over SeQUeNCe on identical scenarios
- Full reproducibility: every result in the paper is reproduced with `qcomnetsim -c configurations/<name>.toml`

---

## Quick Start

### Prerequisites

- Rust 1.75+ — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- uv (Python env manager) — `curl -LsSf https://astral.sh/uv/install.sh | sh`

### Build

```bash
git clone https://github.com/FabriziOki/QComNetSim
cd QComNetSim
cargo build --release
uv sync --python 3.12   # installs SeQUeNCe, SimQN, and validation deps
```

### Run a simulation

```bash
# 2-node entanglement generation (Erbium, 10 km)
cargo run --release -- -c configurations/two_node_entanglement.toml

# 3-node chain with entanglement swapping (Erbium)
cargo run --release -- -c configurations/three_node_erbium.toml -o results.csv

# List available hardware profiles
cargo run --release -- --profiles
```

### Run the cross-simulator validation pipeline

```bash
uv run python src/validation/orchestrator.py validation.toml
# Output: data/comparison.csv and data/plots/
```

---

## TOML Configuration

All experiments are declared in a single TOML file — no recompilation required.

```toml
# 3-node chain — Erbium-167 platform
[topology]
type        = "linear"
nodes       = 3
distance_km = 10.0
attenuation_db_per_km = 0.2
memory_slots = 8
source      = 0
dest        = 2

[hardware]
profile = "erbium"       # or "nv_center", "ideal"

[simulation]
target_pairs  = 100
max_time_ms   = 10000000.0
```

**Available profiles:** `erbium` (t_c=1.3 s, C=500, γ=14 Hz), `nv_center` (t_c=1.0 s, C=1000, γ=13.3 MHz), `ideal`.

Individual profile fields can be overridden inline:

```toml
[hardware]
profile     = "erbium"
coherence_time_s = 2.0   # override a single parameter
```

---

## Implemented Protocols

| Protocol | Description |
|---|---|
| Barrett-Kok | Heralded entanglement generation via BSM |
| BBPSSW | Entanglement purification (fidelity improvement at pair cost) |
| Entanglement Swapping | Werner-state BSM for multi-hop extension |

Routing uses BFS shortest-path over the network graph; repeater nodes are extracted automatically from the path.

---

## Reproducibility

Every result in the paper corresponds to one of the following configurations:

| Figure | Configuration file |
|---|---|
| Success rate / Fidelity (cross-simulator) | `configurations/two_node_entanglement.toml` + validation pipeline |
| Multi-hop fidelity | `configurations/three_node_erbium.toml` |
| Hardware comparison | `configurations/three_node_erbium.toml` vs `configurations/three_node_nv_center.toml` |
| Wall-clock benchmark | `configurations/two_node_entanglement.toml` + SeQUeNCe runner |

---

## Citing This Work

If you use QComNetSim in your research, please cite:

```bibtex
@inproceedings{diaz2026qcomnetsim,
  title     = {{QComNetSim}: A Validated Quantum Network Simulator with Cross-Platform Benchmarking},
  author    = {Diaz, Fabrizio and Kar, Binayak and Kumar, Pankaj and Shen, Shan-Hsiang},
  booktitle = {Proceedings of IEEE Global Communications Conference (GlobeCom)},
  year      = {2026}
}
```

---

## Project Team

| Role | Name | Affiliation |
|---|---|---|
| Developer | Fabrizio Diaz | NTUST / UPTP (exchange) |
| Direct Advisor | Pankaj Kumar, PhD | Quantum Research Lab, NTUST |
| Faculty Advisor | Prof. Binayak Kar | Quantum Research Lab, NTUST |

---

## Development Roadmap

### Semester 1 (Fall 2024)
- [x] Core library and quantum state representation
- [x] 2-node Barrett-Kok entanglement generation
- [x] Realistic multi-factor noise and loss model
- [x] Validation engine against SeQUeNCe
- [x] TOML configuration system and CLI

### Capstone II (Spring 2026)
- [x] Multi-hop topology (3-node chain, BFS routing, entanglement swapping)
- [x] Configurable physics profiles (`PhysicsProfile` — Erbium, NV-center, ideal)
- [x] End-to-end wall-clock benchmarks vs. SeQUeNCe (~45× speedup)
- [x] Metric Aligner for cross-simulator definition alignment
- [x] Statistical rigor: 15-seed mean ± std across all distance sweeps
- [ ] Resource contention modeling across concurrent paths *(future work)*
- [ ] QuISP and NetSquid validation runners *(future work)*

---

## References

1. Wu et al. (2021). *SeQUeNCe*. Quantum Science and Technology. https://doi.org/10.1088/2058-9565/ac22f6
2. Bel & Kiran (2024). *Simulators for Quantum Network Modelling: A Comprehensive Review*. arXiv:2408.11993
3. Chung et al. (2025). *Cross-Validating Quantum Network Simulators*. arXiv:2504.01290
4. Kar & Kumar (2023). *Routing Protocols for Quantum Networks*. arXiv:2305.00708
