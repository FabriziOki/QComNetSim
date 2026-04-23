"""
QComNetSim runner for the validation engine.

For each (distance, seed) pair this runner:
  1. Writes a temporary TOML config mapping validation.toml params to the
     QComNetSim config schema.
  2. Calls the release binary:  qcomnetsim --config <tmp> --output <out.csv>
  3. Parses the one-row CSV the binary writes and returns the standard
     SimulatorRunner row dict.

Seed handling: QComNetSim uses OS-seeded RNG (no seed control yet). Each
"seed" trial is therefore an independent run; variance is real, not synthetic.
This is noted in metric_aligner.py.
"""

import csv
import os
import subprocess
import tempfile
import time
from pathlib import Path

from .base import SimulatorRunner

_BINARY = Path(__file__).parents[3] / "target" / "release" / "qcomnetsim"

# Timeout per individual simulation run (seconds)
_TIMEOUT_S = 300


class QComNetSimRunner(SimulatorRunner):
    def name(self) -> str:
        return "QComNetSim"

    def run(self, params: dict, seed: int) -> list[dict]:
        if not _BINARY.exists():
            raise FileNotFoundError(
                f"QComNetSim release binary not found at {_BINARY}.\n"
                "Build it with:  cargo build --release"
            )

        rows = []
        for dist_km in params["distances_km"]:
            row = self._run_one(dist_km, params, seed)
            rows.append(row)
        return rows

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _run_one(self, dist_km: float, params: dict, seed: int) -> dict:
        cfg_text = self._build_toml(dist_km, params)

        cfg_fd, cfg_path = tempfile.mkstemp(suffix=".toml", prefix="qcs_val_")
        out_fd, out_path = tempfile.mkstemp(suffix=".csv",  prefix="qcs_out_")
        try:
            # Write config
            os.write(cfg_fd, cfg_text.encode())
            os.close(cfg_fd)
            os.close(out_fd)

            t0 = time.perf_counter()
            result = subprocess.run(
                [str(_BINARY), "--config", cfg_path, "--output", out_path],
                capture_output=True,
                timeout=_TIMEOUT_S,
            )
            wall_ms = (time.perf_counter() - t0) * 1000.0

            if result.returncode != 0:
                stderr = result.stderr.decode(errors="replace").strip()
                raise RuntimeError(
                    f"QComNetSim exited with code {result.returncode}: {stderr}"
                )

            return self._parse_csv(out_path, dist_km, wall_ms, seed)

        finally:
            for p in (cfg_path, out_path):
                try:
                    os.unlink(p)
                except FileNotFoundError:
                    pass

    def _build_toml(self, dist_km: float, params: dict) -> str:
        """
        Map the flat validation params dict to the QComNetSim TOML schema.

        Two-node mapping:
          nodes = 2, source = 0, dest = 1

        Three-node mapping:
          nodes = 3, source = 0, dest = 2
          distance_km is the per-link distance, matching SeQUeNCe's convention.

        Hardware fields:
          memory_efficiency        → hardware.memory_efficiency
          memory_fidelity          → hardware.initial_fidelity
          coherence_time_ms        → hardware.coherence_time_ms
          bsm_efficiency           → hardware.generation_bsm_efficiency / swap_bsm_efficiency
        """
        scenario = params.get("scenario", "two_node")
        nodes = 3 if scenario == "three_node" else 2
        dest  = nodes - 1
        profile = params.get("hardware_profile", "erbium")
        return f"""\
[topology]
type = "linear"
nodes = {nodes}
distance_km = {dist_km}
attenuation_db_per_km = {params["attenuation_db_per_km"]}
memory_slots = 200
source = 0
dest = {dest}

[hardware]
profile = "{profile}"
memory_efficiency = {params["memory_efficiency"]}
initial_fidelity = {params["memory_fidelity"]}
coherence_time_ms = {params["coherence_time_ms"]}
generation_bsm_efficiency = {params["bsm_efficiency"]}
swap_bsm_efficiency = {params["bsm_efficiency"]}

[simulation]
target_pairs = {params["num_attempts"]}
max_time_ms = 10000000.0
"""

    def _parse_csv(
        self, path: str, dist_km: float, wall_ms: float, seed: int
    ) -> dict:
        with open(path, newline="") as f:
            row = next(csv.DictReader(f))
        return {
            "simulator":    self.name(),
            "distance_km":  dist_km,
            "success_rate": float(row["success_rate"]),
            "avg_fidelity": float(row["avg_fidelity"]),
            "throughput":   float(row["throughput"]),
            "memory_used":  int(float(row["memory_used"])),
            "wall_clock_ms": wall_ms,
            "seed":         seed,
        }
