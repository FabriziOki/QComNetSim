import numpy as np
import time
import csv
from multiprocessing import Pool

def fidelity(state1, state2):
    """Calculate fidelity between two states"""
    inner = np.vdot(state1, state2)
    return np.abs(inner) ** 2

def compute_fidelity(_):
    """Single fidelity computation (for parallel mapping)"""
    bell1 = (1/np.sqrt(2)) * np.array([1, 0, 0, 1], dtype=complex)
    bell2 = (1/np.sqrt(2)) * np.array([1, 0, 0, 1], dtype=complex)
    return fidelity(bell1, bell2)

def benchmark_sequential(size):
    """Sequential fidelity calculations"""
    start = time.perf_counter()
    results = [compute_fidelity(None) for _ in range(size)]
    end = time.perf_counter()
    return end - start

def benchmark_parallel(size):
    """Parallel fidelity calculations using multiprocessing"""
    start = time.perf_counter()
    with Pool() as pool:
        results = pool.map(compute_fidelity, range(size))
    end = time.perf_counter()
    return end - start

def run_benchmarks():
    sizes = [100, 1_000, 10_000]
    results = []
    
    print("Python Parallel Operations Benchmark")
    print("=" * 50)
    
    for size in sizes:
        # Run multiple iterations
        seq_times = []
        par_times = []
        
        for _ in range(3):
            seq_times.append(benchmark_sequential(size))
            par_times.append(benchmark_parallel(size))
        
        avg_seq = sum(seq_times) / len(seq_times)
        avg_par = sum(par_times) / len(par_times)
        
        print(f"\nSize: {size:,}")
        print(f"  Sequential: {avg_seq*1000:.3f} ms")
        print(f"  Parallel:   {avg_par*1000:.3f} ms")
        print(f"  Speedup:    {avg_seq/avg_par:.2f}x")
        
        results.extend([
            {
                "language": "Python",
                "benchmark": "Sequential",
                "size": size,
                "time_ms": avg_seq * 1000
            },
            {
                "language": "Python",
                "benchmark": "Parallel",
                "size": size,
                "time_ms": avg_par * 1000
            },
        ])
    
    # Save to CSV
    with open("../benches/results/python_parallel_results.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["language", "benchmark", "size", "time_ms"])
        writer.writeheader()
        writer.writerows(results)
    
    print("\nSaved to benches/results/python_parallel_results.csv")

if __name__ == "__main__":
    run_benchmarks()
