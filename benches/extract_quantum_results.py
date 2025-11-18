import json
import csv
from pathlib import Path

def extract_quantum_results():
    results = []
    
    groups = {
        "Single Qubit Gates": ["Pauli-X", "Hadamard"],
        "Fidelity Calculation": ["Fidelity"],
        "State Creation": ["Bell State"],
    }
    
    for group_name, benchmarks in groups.items():
        for benchmark in benchmarks:
            for size in [1000, 10000, 100000]:
                json_path = Path(f"../target/criterion/{group_name}/{benchmark}/{size}/rust_results/estimates.json")
                
                if json_path.exists():
                    with open(json_path) as f:
                        data = json.load(f)
                        mean_ns = data["mean"]["point_estimate"]
                        mean_ms = mean_ns / 1_000_000
                        
                        results.append({
                            "language": "Rust",
                            "benchmark": benchmark,
                            "size": size,
                            "time_ms": mean_ms
                        })
    
    # Append to CSV
    import os
    file_exists = os.path.exists("../benches/results/rust_quantum_results.csv")
    
    with open("../benches/results/rust_quantum_results.csv", "a", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["language", "benchmark", "size", "time_ms"])
        if not file_exists:
            writer.writeheader()
        writer.writerows(results)
    
    print("Extracted Rust quantum results to quantum_results.csv")

if __name__ == "__main__":
    extract_quantum_results()
