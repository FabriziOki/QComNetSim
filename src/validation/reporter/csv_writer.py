import csv
from pathlib import Path

_OUT = Path(__file__).parents[4] / "data" / "comparison.csv"

_COLUMNS = [
    "simulator", "distance_km",
    "success_rate_mean", "success_rate_std",
    "avg_fidelity_mean", "avg_fidelity_std",
    "throughput_mean", "throughput_std",
    "memory_used_mean", "memory_used_std",
    "wall_clock_ms_mean", "wall_clock_ms_std",
]


def write_csv(rows: list[dict], path: Path = _OUT) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=_COLUMNS, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)
    print(f"  Saved: {path}")
