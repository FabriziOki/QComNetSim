import json
import csv
from pathlib import Path

def extract_parallel_results():
    results = []
    
    benchmarks = ["Sequential", "Parallel"]
    sizes = [100, 1000, 10000]
    
    for benchmark in benchmarks:
        for size in sizes:
            json_path = Path(f"../target/criterion/Parallel Operations/{benchmark}/{size}/rust_results/estimates.json")
            
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
    
    # Append to CSV (or create if doesn't exist)
    import os
    file_exists = os.path.exists("../benches/results/rust_parallel_results.csv")
    
    with open("../benches/results/rust_parallel_results.csv", "a", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["language", "benchmark", "size", "time_ms"])
        if not file_exists:
            writer.writeheader()
        writer.writerows(results)
    
    print("Extracted Rust parallel results to benches/results/rust_parallel_results.csv")

if __name__ == "__main__":
    extract_parallel_results()
