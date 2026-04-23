"""
Reads a validation TOML config and emits a standardised parameter dict
that every SimulatorRunner receives identically.
"""

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:
    import tomli as tomllib  # pip install tomli


def load(path: str) -> dict:
    """
    Load and validate a validation config file.

    Expected TOML structure:

        [scenario]
        distances_km          = [1, 5, 10, 20, 50]
        attenuation_db_per_km = 0.2
        num_attempts          = 100
        seeds                 = [0, 1, 2, 3, 4]

        [hardware]
        memory_fidelity       = 0.95
        memory_efficiency     = 0.9
        coherence_time_ms     = 100.0
        bsm_efficiency        = 0.9
        generation_frequency_khz = 2.0

    Returns a flat dict consumed by all runners.
    """
    with open(path, "rb") as f:
        raw = tomllib.load(f)

    scenario = raw.get("scenario", {})
    hardware = raw.get("hardware", {})

    params = {
        # Scenario
        "scenario": scenario.get("type", "two_node"),
        "distances_km": scenario["distances_km"],
        "attenuation_db_per_km": scenario.get("attenuation_db_per_km", 0.2),
        "num_attempts": scenario.get("num_attempts", 100),
        "seeds": scenario.get("seeds", [0]),
        # Hardware
        "memory_fidelity": hardware.get("memory_fidelity", 0.95),
        "memory_efficiency": hardware.get("memory_efficiency", 0.9),
        "coherence_time_ms": hardware.get("coherence_time_ms", 100.0),
        "bsm_efficiency": hardware.get("bsm_efficiency", 0.9),
        "generation_frequency_khz": hardware.get("generation_frequency_khz", 2.0),
    }

    _validate(params)
    return params


def _validate(params: dict) -> None:
    assert params["distances_km"], "distances_km must not be empty"
    assert params["num_attempts"] > 0, "num_attempts must be positive"
    assert params["seeds"], "seeds must not be empty"
    assert 0 < params["memory_fidelity"] <= 1
    assert 0 < params["memory_efficiency"] <= 1
    assert 0 < params["bsm_efficiency"] <= 1
