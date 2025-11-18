import numpy as np
import time
import csv

# Pauli matrices
PAULI_X = np.array([[0, 1], [1, 0]], dtype=complex)
PAULI_Y = np.array([[0, -1j], [1j, 0]], dtype=complex)
PAULI_Z = np.array([[1, 0], [0, -1]], dtype=complex)
HADAMARD = (1/np.sqrt(2)) * np.array([[1, 1], [1, -1]], dtype=complex)

def apply_gate(state, gate):
    """Apply gate to quantum state"""
    return gate @ state

def fidelity(state1, state2):
    """Calculate fidelity between two states"""
    inner = np.vdot(state1, state2)
    return np.abs(inner) ** 2

def benchmark_pauli_x(size):
    state = np.array([1, 0], dtype=complex)
    
    start = time.perf_counter()
    for _ in range(size):
        state = apply_gate(state, PAULI_X)
    end = time.perf_counter()
    
    return end - start

def benchmark_hadamard(size):
    state = np.array([1, 0], dtype=complex)
    
    start = time.perf_counter()
    for _ in range(size):
        state = apply_gate(state, HADAMARD)
    end = time.perf_counter()
    
    return end - start

def benchmark_fidelity_calc(size):
    bell1 = (1/np.sqrt(2)) * np.array([1, 0, 0, 1], dtype=complex)
    bell2 = (1/np.sqrt(2)) * np.array([1, 0, 0, 1], dtype=complex)
    
    start = time.perf_counter()
    for _ in range(size):
        f = fidelity(bell1, bell2)
    end = time.perf_counter()
    
    return end - start

def benchmark_bell_creation(size):
    start = time.perf_counter()
    for _ in range(size):
        bell = (1/np.sqrt(2)) * np.array([1, 0, 0, 1], dtype=complex)
    end = time.perf_counter()
    
    return end - start

def run_benchmarks():
    sizes = [1_000, 10_000, 100_000]
    results = []
    
    print("Python Quantum Operations Benchmark")
    print("=" * 50)
    
    benchmarks = [
        ("Pauli-X", benchmark_pauli_x),
        ("Hadamard", benchmark_hadamard),
        ("Fidelity", benchmark_fidelity_calc),
        ("Bell State", benchmark_bell_creation),
    ]
    
    for name, func in benchmarks:
        print(f"\n{name}:")
        for size in sizes:
            times = []
            for _ in range(5):
                times.append(func(size))
            
            avg_time = sum(times) / len(times)
            print(f"  {size:,}: {avg_time*1000:.3f} ms")
            
            results.append({
                "language": "Python",
                "benchmark": name,
                "size": size,
                "time_ms": avg_time * 1000
            })
    
    # Save to CSV
    with open("benches/results/python_quantum_results.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["language", "benchmark", "size", "time_ms"])
        writer.writeheader()
        writer.writerows(results)
    
    print("\nSaved to benches/results/python_quantum_results.csv")

if __name__ == "__main__":
    run_benchmarks()
