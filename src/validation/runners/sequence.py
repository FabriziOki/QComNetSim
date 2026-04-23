import sys
import time
from pathlib import Path

from .base import SimulatorRunner

_SEQUENCE_DIR = Path(__file__).parent.parent / "sequence"
if str(_SEQUENCE_DIR) not in sys.path:
    sys.path.insert(0, str(_SEQUENCE_DIR))


class SeQUeNCeRunner(SimulatorRunner):
    def name(self) -> str:
        return "SeQUeNCe"

    def run(self, params: dict, seed: int) -> list[dict]:
        scenario = params.get("scenario", "two_node")
        if scenario == "three_node":
            return self._run_three_node(params, seed)
        return self._run_two_node(params, seed)

    def _run_two_node(self, params: dict, seed: int) -> list[dict]:
        from two_node_barrett_kok import run_experiment  # noqa: PLC0415

        rows = []
        for dist_km in params["distances_km"]:
            t0 = time.perf_counter()
            result = run_experiment(
                distance_m=dist_km * 1000,
                attenuation=params["attenuation_db_per_km"] / 1000,
                num_attempts=params["num_attempts"],
            )
            wall_ms = (time.perf_counter() - t0) * 1000
            rows.append(self._make_row(dist_km, result, wall_ms, seed))
        return rows

    def _run_three_node(self, params: dict, seed: int) -> list[dict]:
        from three_node_swapping import run_experiment  # noqa: PLC0415

        rows = []
        for dist_km in params["distances_km"]:
            t0 = time.perf_counter()
            result = run_experiment(
                link_distance_m=dist_km * 1000,
                attenuation=params["attenuation_db_per_km"] / 1000,
                num_attempts=params["num_attempts"],
                params=params,
            )
            wall_ms = (time.perf_counter() - t0) * 1000
            rows.append(self._make_row(dist_km, result, wall_ms, seed))
        return rows

    def _make_row(self, dist_km: float, result: dict, wall_ms: float, seed: int) -> dict:
        return {
            "simulator":    self.name(),
            "distance_km":  dist_km,
            "success_rate": result["success_rate"],
            "avg_fidelity": result["avg_fidelity"],
            "throughput":   result["throughput"],
            "memory_used":  result["memory_used"],
            "wall_clock_ms": wall_ms,
            "seed":         seed,
        }
