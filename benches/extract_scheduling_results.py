import json
import csv
from pathlib import Path

def extract_criterion_to_csv():
    criterion_dir = Path("../target/criterion/Event Scheduling")
    
    results = []
    
    for benchmark_type in ["Insert", "Insert+Remove"]:
        for size in [100, 1000, 10000, 100000]:
            json_path = criterion_dir / benchmark_type / str(size) / "rust_results" / "estimates.json"
            
            if json_path.exists():
                with open(json_path) as f:
                    data = json.load(f)
                    # Mean time in nanoseconds, convert to milliseconds
                    mean_ns = data["mean"]["point_estimate"]
                    mean_ms = mean_ns / 1_000_000
                    
                    results.append({
                        "language": "Rust",
                        "benchmark": benchmark_type,
                        "size": size,
                        "time_ms": mean_ms
                    })
    
    # Write to CSV
    with open("../benches/results/rust_scheduling_results.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["language", "benchmark", "size", "time_ms"])
        writer.writeheader()
        writer.writerows(results)
    
    print("Saved Rust results to benches/results/rust_scheduling_results.csv")

if __name__ == "__main__":
    extract_criterion_to_csv()
