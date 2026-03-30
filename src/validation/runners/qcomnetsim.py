import csv
import subprocess
import time
from pathlib import Path

from .base import SimulatorRunner

# TODO: When the Rust binary gains CLI param support, pass params directly
# instead of relying on its hardcoded defaults. Tracked in TODO.md.
_BINARY = Path(__file__).parents[4] / "target" / "release" / "examples" / "two_node_barrett_kok"
_CSV_OUT = Path(__file__).parents[4] / "data" / "qcomnetsim_results.csv"


class QComNetSimRunner(SimulatorRunner):
    def name(self) -> str:
        return "QComNetSim"

    def run(self, params: dict, seed: int) -> list[dict]:
        if not _BINARY.exists():
            raise FileNotFoundError(
                f"QComNetSim binary not found at {_BINARY}. "
                "Run: cargo build --release --example two_node_barrett_kok"
            )

        t0 = time.perf_counter()
        subprocess.run([str(_BINARY)], check=True, capture_output=True)
        total_wall_ms = (time.perf_counter() - t0) * 1000

        rows = []
        with open(_CSV_OUT, newline="") as f:
            reader = csv.DictReader(f)
            raw = list(reader)

        # Distribute wall-clock time evenly across distances (binary runs all at once)
        per_row_ms = total_wall_ms / len(raw) if raw else 0

        for row in raw:
            dist_km = float(row["distance_km"])
            if dist_km not in params["distances_km"]:
                continue
            rows.append({
                "simulator": self.name(),
                "distance_km": dist_km,
                "success_rate": float(row["success_rate"]),
                "avg_fidelity": float(row["avg_fidelity"]),
                "throughput": float(row["throughput"]),
                "memory_used": int(row["memory_used"]),
                "wall_clock_ms": per_row_ms,
                "seed": seed,
            })

        return rows
