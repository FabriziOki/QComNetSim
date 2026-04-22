"""
Validation orchestrator — entry point for the validation engine.

Usage:
    python src/validation/orchestrator.py validation.toml [--simulators qcomnetsim,sequence]

Steps:
    1. Load and validate config (ConfigTranslator)
    2. Instantiate selected SimulatorRunners
    3. Run all runners over all seeds (ResultsCollector)
    4. Align metrics across simulators (MetricAligner)
    5. Write comparison CSV and plots (Reporter)
"""

import argparse
import sys
from pathlib import Path

# Allow running as a script from the project root.
# parents[1] = src/  — where the `validation` package lives.
sys.path.insert(0, str(Path(__file__).parents[1]))

from validation import config_translator, metric_aligner, results_collector
from validation.reporter import csv_writer, plot_generator
from validation.runners import QComNetSimRunner, SeQUeNCeRunner, QuNetSimRunner, SimQNRunner

_AVAILABLE_RUNNERS = {
    "qcomnetsim": QComNetSimRunner,
    "sequence": SeQUeNCeRunner,
    "quuetsim": QuNetSimRunner,
    "simqn": SimQNRunner,
}

_DEFAULT_RUNNERS = ["qcomnetsim", "sequence"]


def main() -> None:
    parser = argparse.ArgumentParser(description="QComNetSim Validation Engine")
    parser.add_argument("config", help="Path to validation TOML config")
    parser.add_argument(
        "--simulators",
        default=",".join(_DEFAULT_RUNNERS),
        help=f"Comma-separated list of simulators to run. Available: {', '.join(_AVAILABLE_RUNNERS)}",
    )
    args = parser.parse_args()

    print("=== QComNetSim Validation Engine ===\n")

    # 1. Load config
    print(f"Loading config: {args.config}")
    params = config_translator.load(args.config)
    print(f"  Distances : {params['distances_km']} km")
    print(f"  Attenuation: {params['attenuation_db_per_km']} dB/km")
    print(f"  Seeds     : {params['seeds']}")
    print(f"  Attempts  : {params['num_attempts']}\n")

    # 2. Instantiate runners
    selected = [s.strip() for s in args.simulators.split(",")]
    runners = []
    for key in selected:
        if key not in _AVAILABLE_RUNNERS:
            print(f"  [WARN] Unknown simulator '{key}', skipping.")
            continue
        runners.append(_AVAILABLE_RUNNERS[key]())
    print(f"Running simulators: {[r.name() for r in runners]}\n")

    # 3. Collect raw results
    print("Running simulations...")
    raw_rows: list[dict] = []
    for runner in runners:
        for seed in params["seeds"]:
            print(f"  [{runner.name()}] seed={seed} ...", flush=True)
            try:
                rows = runner.run(params, seed)
                raw_rows.extend(rows)
            except NotImplementedError as e:
                print(f"  [SKIP] {runner.name()} is not yet implemented: {e}")
                break  # skip all seeds for this runner
            except Exception as e:
                print(f"  [ERROR] {runner.name()} seed={seed} failed: {e}")

    # 4. Align metrics
    aligned = metric_aligner.align(raw_rows)

    # 5. Aggregate (mean ± std per simulator × distance)
    aggregated = results_collector.aggregate(aligned)

    # 6. Write outputs
    if not aggregated:
        print("\nNo results collected — nothing to write.")
        sys.exit(1)

    print("\nWriting outputs...")
    csv_writer.write_csv(aggregated)
    plot_generator.generate_plots(aggregated)

    print("\nDone.")


if __name__ == "__main__":
    main()
