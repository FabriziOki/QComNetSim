import sys
import time
from pathlib import Path

from .base import SimulatorRunner

# Allow importing the existing SeQUeNCe script directly.
_SEQUENCE_DIR = Path(__file__).parent.parent / "sequence"
if str(_SEQUENCE_DIR) not in sys.path:
    sys.path.insert(0, str(_SEQUENCE_DIR))


class SeQUeNCeRunner(SimulatorRunner):
    def name(self) -> str:
        return "SeQUeNCe"

    def run(self, params: dict, seed: int) -> list[dict]:
        from two_node_barrett_kok import run_experiment  # noqa: PLC0415

        rows = []
        for dist_km in params["distances_km"]:
            dist_m = dist_km * 1000
            t0 = time.perf_counter()
            result = run_experiment(
                distance_m=dist_m,
                attenuation=params["attenuation_db_per_km"] / 1000,  # dB/m
                num_attempts=params["num_attempts"],
            )
            wall_ms = (time.perf_counter() - t0) * 1000

            rows.append({
                "simulator": self.name(),
                "distance_km": dist_km,
                "success_rate": result["success_rate"],
                "avg_fidelity": result["avg_fidelity"],
                "throughput": result["throughput"],
                "memory_used": result["memory_used"],
                "wall_clock_ms": wall_ms,
                "seed": seed,
            })

        return rows
