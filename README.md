<div align="center">
<img src="assets/name.svg" alt="QComNetSim Logo" width="1000"/>
</div>

> **Status**: Active Development - (Fall 2024)  
> **First Target Completion**: December 24, 2024 

A high-performance quantum network simulator written in Rust, designed for educational purposes and cross-simulator validation.

---

## Project Vision

QComNetSim aims to create a small-scale, educational quantum network simulator that addresses common limitations in existing simulators while providing built-in validation capabilities. Unlike large-scale simulators, QComNetSim focuses on:

- **Educational clarity**: 4-5 node networks with well-documented, understandable code
- **Performance**: Leveraging Rust for speed and memory safety
- **Validation-first design**: Built-in comparison with established simulators
- **Modular architecture**: Easy to extend and modify for research

## Development Roadmap

### Semester 1 (Fall 2024)
- [x] Project architecture design
- [x] Core library setup
- [x] Basic quantum state representation
- [x] 2-node linear entanglement generation
- [x] Add Realistic Noise & Loss Model
- [x] Validation Engine against SeQUeNCe
- [x] TOML Configuration file support
- [x] Basic CLI interface with CSV output

### Semester 2 (Spring 2025)
- [ ] Graphical user interface
- [ ] Extended protocol support
- [ ] Performance benchmarking suite
- [ ] Comprehensive documentation
- [ ] Advanced noise models

## Motivation

Quantum networks promise revolutionary capabilities in secure communication, distributed quantum computing, and sensing. However, designing and validating quantum network protocols faces fundamental challenges:

**Physical Complexity**: Real quantum systems exhibit decoherence, photon loss, imperfect gates, and measurement errors. Simulators that ignore these effects produce unrealistic results that fail in deployment.

**Performance Requirements**: Simulating realistic noise models requires tracking density matrices, applying error channels, and Monte Carlo sampling—computationally intensive operations that become prohibitive for large networks or long timescales.

**Validation Gap**: Without cross-verification against established frameworks, new simulators risk introducing subtle physics errors that invalidate results. Yet most tools lack built-in validation mechanisms.

**Accessibility**: Researchers need tools that balance accuracy with usability—complex enough to model real quantum phenomena, yet approachable enough for rapid protocol development and iteration.

QComNetSim addresses these challenges through:
- **Realistic physics modeling** with configurable noise parameters
- **High-performance execution** via Rust's zero-cost abstractions
- **Built-in validation framework** for cross-simulator verification
- **Modular architecture** enabling protocol experimentation without sacrificing accuracy
---

## Building & Running

### Prerequisites
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- UV package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)

### Setup
```bash
git clone https://github.com/yourusername/qcomnetsim
cd qcomnetsim

# python venv (optional but recommended)
uv venv

source .venv/bin/activate

# Install Python dependencies
uv sync

# Build Rust simulator
cargo build --release
```

### Running Simulations

> **Note**: CLI and TOML configuration system is under development and testing. Current workflow uses example binaries:
```bash
# Run Barrett-Kok protocol simulation
cargo run --release --example two_node_barrett_kok

# Output: data/qcomnetsim_results.csv
```

### Cross-Simulator Validation

Validate QComNetSim against SeQUeNCe:
```bash
# Run complete validation pipeline
./run_validation.sh

# Output: 
#   data/sequence_results.csv
#   data/comparison.csv
#   data/plots/*.png
```
## Project Team

- **Developer**: Fabrizio Diaz, Undergraduate CS Student, NTUST
- **Direct Advisor**: Pankaj Kumar, PhD Scholar, NTUST
- **Faculty Advisor**: Prof. Binayak Kar, Assistant Professor, NTUST
- **Lab**: Quantum Research Lab

## Acknowledgments

This project is part of a two-semester capstone project at National Taiwan University of Science and Technology (NTUST), where I am an exchange student from Taiwan-Paraguay Polytechnic University. 

This work is conducted under the guidance of Binayak Kar and Pankaj Kumar at Quantum Research Lab, NTUST.

---

##  References

This project builds upon established research in quantum network simulation:

1. **SeQUeNCe - Simulator of Quantum Network Communication**
   - Wu, X., Kolar, A., Chung, J., Jin, D., Zhong, T., Kettimuthu, R., & Suchara, M. (2021). SeQUeNCe: a customizable discrete-event simulator of quantum networks. *Quantum Science and Technology*, 6(4). https://doi.org/10.1088/2058-9565/ac22f6
   - Repository: https://github.com/sequence-toolbox/SeQUeNCe
   - Our approach: Validate against SeQUeNCe's accurate physics models

2. **Routing Protocols for Quantum Networks**
   - Kar, B., & Kumar, P. (2023). Routing Protocols for Quantum Networks: Overview and Challenges. arXiv:2305.00708 [quant-ph]. https://arxiv.org/abs/2305.00708
   - Focus: Quantum routing protocols and network design challenges
   - Connection: QComNetSim will support routing protocol research and validation

3. **Quantum Network Simulators: A Comprehensive Review**
   - Bel, O., & Kiran, M. (2024). Simulators for Quantum Network Modelling: A Comprehensive Review. arXiv:2408.11993 [quant-ph]. https://arxiv.org/abs/2408.11993
   - Focus: Survey of quantum network simulators and validation methods
   - Connection: Informs our validation framework design and benchmarking approach

4. **Other Quantum Network Simulators**:
   - QuNetSim: https://github.com/tqsd/QuNetSim
   - SimQN: https://github.com/ertuil/SimQN

---

> **Note**: This is an active research project. Documentation, features, and APIs are subject to change as development progresses.
