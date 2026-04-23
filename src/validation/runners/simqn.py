"""
SimQN runner for the validation engine.

Physical model: SimQN's WernerStateEntanglement uses continuous fidelity
decay w(L) = w_0 * exp(-decoherence_rate * L) rather than Barrett-Kok's
probabilistic photon loss.  As a consequence:

  - success_rate is always 1.0 (every pair arrives).
  - avg_fidelity decreases with distance.
  - Throughput is fixed at the generation frequency.

Use SimQN results to validate the Werner-state fidelity formula shared by
all three simulators.  Do not compare success_rate or throughput directly
against SeQUeNCe / QComNetSim (different protocol semantics).
"""

import sys
import time
from pathlib import Path

from .base import SimulatorRunner

_SIMQN_DIR = Path(__file__).parent.parent / "simqn"
if str(_SIMQN_DIR) not in sys.path:
    sys.path.insert(0, str(_SIMQN_DIR))


class SimQNRunner(SimulatorRunner):
    def name(self) -> str:
        return "SimQN"

    def run(self, params: dict, seed: int) -> list[dict]:
        scenario = params.get("scenario", "two_node")
        if scenario == "three_node":
            return self._run_three_node(params, seed)
        return self._run_two_node(params, seed)

    def _run_two_node(self, params: dict, seed: int) -> list[dict]:
        from two_node_werner import run_experiment  # noqa: PLC0415

        rows = []
        for dist_km in params["distances_km"]:
            t0 = time.perf_counter()
            result = run_experiment(
                distance_m=dist_km * 1000,
                attenuation=params["attenuation_db_per_km"] / 1000,
                num_attempts=params["num_attempts"],
                params=params,
                seed=seed,
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
                seed=seed,
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
