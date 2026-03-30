from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd

_PLOTS_DIR = Path(__file__).parents[4] / "data" / "plots"
_COLORS = ["#2E86AB", "#A23B72", "#F18F01", "#C73E1D"]
_MARKERS = ["o", "s", "^", "D"]


def generate_plots(rows: list[dict], out_dir: Path = _PLOTS_DIR) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    df = pd.DataFrame(rows).sort_values(["simulator", "distance_km"])
    simulators = df["simulator"].unique()

    _plot_metric(df, simulators, "success_rate", "Success Rate", "[0–1]", out_dir)
    _plot_metric(df, simulators, "avg_fidelity", "Average Fidelity", "[0–1]", out_dir)
    _plot_metric(df, simulators, "throughput", "Throughput", "pairs/sec", out_dir)
    _plot_metric(df, simulators, "wall_clock_ms", "Wall-clock Time", "ms", out_dir)

    print(f"  Plots saved to {out_dir}/")


def _plot_metric(
    df: pd.DataFrame,
    simulators,
    metric: str,
    label: str,
    unit: str,
    out_dir: Path,
) -> None:
    mean_col = f"{metric}_mean"
    std_col = f"{metric}_std"
    if mean_col not in df.columns:
        return

    fig, ax = plt.subplots(figsize=(8, 5))
    for i, sim in enumerate(simulators):
        sub = df[df["simulator"] == sim]
        ax.errorbar(
            sub["distance_km"],
            sub[mean_col],
            yerr=sub[std_col] if std_col in df.columns else None,
            label=sim,
            color=_COLORS[i % len(_COLORS)],
            marker=_MARKERS[i % len(_MARKERS)],
            linewidth=2,
            markersize=8,
            capsize=4,
        )

    ax.set_xlabel("Distance (km)", fontsize=12, fontweight="bold")
    ax.set_ylabel(f"{label} ({unit})", fontsize=12, fontweight="bold")
    ax.set_title(f"{label} vs Distance", fontsize=14, fontweight="bold")
    ax.legend(fontsize=11)
    ax.grid(True, alpha=0.3)
    plt.tight_layout()
    out_path = out_dir / f"{metric}.png"
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"  Saved: {out_path}")
