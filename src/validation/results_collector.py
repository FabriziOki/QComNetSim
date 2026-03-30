"""
Runs each SimulatorRunner over all seeds and aggregates per-(simulator, distance)
statistics: mean and standard deviation for every numeric metric.
"""

import statistics
from collections import defaultdict

_NUMERIC_METRICS = ["success_rate", "avg_fidelity", "throughput", "memory_used", "wall_clock_ms"]


def aggregate(raw_rows: list[dict]) -> list[dict]:
    """Aggregate pre-collected raw rows into mean ± std per (simulator, distance)."""
    return _aggregate(raw_rows)


def collect(runners: list[SimulatorRunner], params: dict) -> list[dict]:
    """Run all runners over all seeds and return aggregated rows."""
    raw: list[dict] = []
    for runner in runners:
        for seed in params["seeds"]:
            raw.extend(runner.run(params, seed))
    return _aggregate(raw)


def _aggregate(raw_rows: list[dict]) -> list[dict]:
    buckets: dict[tuple, list[dict]] = defaultdict(list)
    for row in raw_rows:
        key = (row["simulator"], row["distance_km"])
        buckets[key].append(row)

    aggregated = []
    for (simulator, distance_km), rows in sorted(buckets.items()):
        agg = {"simulator": simulator, "distance_km": distance_km}
        for metric in _NUMERIC_METRICS:
            values = [r[metric] for r in rows if r.get(metric) is not None]
            if values:
                agg[f"{metric}_mean"] = statistics.mean(values)
                agg[f"{metric}_std"] = statistics.stdev(values) if len(values) > 1 else 0.0
            else:
                agg[f"{metric}_mean"] = None
                agg[f"{metric}_std"] = None
        aggregated.append(agg)

    return aggregated
