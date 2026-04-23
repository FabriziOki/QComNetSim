"""
Three-node Werner-state entanglement swapping experiment using SimQN.

Uses SimQN's WernerStateEntanglement model directly:
  1. Generate EPR_left  with fidelity F_init; apply channel decay over link_distance_m.
  2. Generate EPR_right with fidelity F_init; apply channel decay over link_distance_m.
  3. Repeater swaps:  EPR_final = EPR_left.swapping(EPR_right)
     => w_final = w_left * w_right  (Werner-state BSM rule)

The swap is performed num_attempts times.  Because the Werner-state fidelity
formula is deterministic (no stochastic photon loss), all attempts yield the
same fidelity.  success_rate is always 1.0.

Use for: validating the three-node Werner-state swapping formula independently
of the Barrett-Kok protocol implemented in SeQUeNCe / QComNetSim.
"""

import math
import time
import uuid

from qns.models.epr.werner import WernerStateEntanglement
from qns.utils.rnd import set_seed as qns_set_seed


def run_experiment(
    link_distance_m: float,
    attenuation: float = 0.0002,
    num_attempts: int = 50,
    params: dict | None = None,
    seed: int = 0,
) -> dict:
    """
    Run a three-node Werner-state swapping experiment using SimQN's fidelity model.

    Args:
        link_distance_m: per-hop distance in metres (total path = 2×).
        attenuation:     fiber loss in dB/m.
        num_attempts:    number of swap trials.
        params:          hardware parameter dict from the validation config translator.
        seed:            RNG seed (passed to SimQN's global RNG).

    Returns:
        Standard metrics dict.  success_rate is always 1.0; avg_fidelity
        is the post-swap Werner-state fidelity.
    """
    if params is None:
        params = {}

    qns_set_seed(seed)

    init_fidelity = params.get("memory_fidelity", 0.95)
    freq_hz = params.get("generation_frequency_khz", 2.0) * 1e3

    # Decoherence rate per metre: γ = att_dB_per_m * ln(10)/10
    decoherence_rate = attenuation * math.log(10) / 10

    fidelities: list[float] = []
    t0_wall = time.perf_counter()

    for _ in range(num_attempts):
        # Generate left-link EPR and apply channel fidelity decay
        epr_left = WernerStateEntanglement(fidelity=init_fidelity, name=uuid.uuid4().hex)
        epr_left.transfer_error_model(link_distance_m, decoherence_rate)

        # Generate right-link EPR and apply channel fidelity decay
        epr_right = WernerStateEntanglement(fidelity=init_fidelity, name=uuid.uuid4().hex)
        epr_right.transfer_error_model(link_distance_m, decoherence_rate)

        # Repeater performs entanglement swapping: w_final = w_left * w_right
        epr_final = epr_left.swapping(epr_right, name=uuid.uuid4().hex)
        fidelities.append(epr_final.fidelity)

    wall_ms = (time.perf_counter() - t0_wall) * 1000

    # Sim time: enough for num_attempts rounds at generation frequency
    sim_time_s = num_attempts / freq_hz

    return {
        "link_distance_m": link_distance_m,
        "distance_km":     link_distance_m / 1000,
        "successes":       num_attempts,
        "total_attempts":  num_attempts,
        "success_rate":    1.0,
        "avg_fidelity":    sum(fidelities) / len(fidelities) if fidelities else 0.0,
        "fidelities":      fidelities,
        "throughput":      num_attempts / sim_time_s if sim_time_s > 0 else 0.0,
        "memory_used":     num_attempts * 2,
        "runtime_ms":      wall_ms,
    }


if __name__ == "__main__":
    erbium = {"memory_fidelity": 0.95, "generation_frequency_khz": 2.0}
    print("=== SimQN Werner-state three-node swapping (Erbium) ===")
    for km in [1, 5, 10, 20, 50]:
        r = run_experiment(km * 1000, attenuation=0.0002, num_attempts=50, params=erbium)
        print(f"{km:2d} km  F={r['avg_fidelity']:.4f}")
