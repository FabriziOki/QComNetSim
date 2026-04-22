"""
Consistent publication-quality comparison plots.

Design decisions
----------------
- Simulator → color/marker is fixed by NAME, not by iteration order, so the
  same simulator always looks the same regardless of which others are present.
- Error representation: filled 1σ band (fill_between) where variance exists,
  cleaner for publication than sparse cap-bars.
- Log scale on throughput and wall_clock_ms (differ by ~1000× between
  simulators — linear scale hides the QComNetSim curve at the bottom).
- x-ticks at actual data points only (1, 5, 10, 20, 50 km).
- Global rcParams applied via rc_context so the rest of the process is
  unaffected.
"""

from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
import pandas as pd

_PLOTS_DIR = Path(__file__).parents[3] / "data" / "plots"

# ── Per-simulator style (extend as new simulators are added) ──────────────────
_STYLE: dict[str, dict] = {
    "QComNetSim": {"color": "#2E86AB", "marker": "o"},
    "SeQUeNCe":   {"color": "#A23B72", "marker": "s"},
    "QuNetSim":   {"color": "#F18F01", "marker": "^"},
    "SimQN":      {"color": "#C73E1D", "marker": "D"},
}
_FALLBACK = [
    {"color": "#555555", "marker": "P"},
    {"color": "#888888", "marker": "X"},
]

# ── Metrics to plot ───────────────────────────────────────────────────────────
# (csv_prefix, y-axis label, unit string, use log scale)
_METRICS = [
    ("success_rate",  "Success Rate",     "ratio [0–1]", False),
    ("avg_fidelity",  "Average Fidelity", "ratio [0–1]", False),
    ("throughput",    "Throughput",       "pairs / s",   True),
    ("memory_used",   "Memory Used",      "pairs",       False),
    ("wall_clock_ms", "Wall-clock Time",  "ms",          True),
]

# ── Shared rcParams ───────────────────────────────────────────────────────────
_RC = {
    "font.family":       "sans-serif",
    "font.size":         11,
    "axes.titlesize":    12,
    "axes.labelsize":    11,
    "xtick.labelsize":   10,
    "ytick.labelsize":   10,
    "legend.fontsize":   10,
    "legend.framealpha": 0.85,
    "axes.linewidth":    1.2,
    "lines.linewidth":   2.0,
    "lines.markersize":  7,
    "figure.figsize":    (7.0, 4.5),
    "figure.dpi":        150,
    "savefig.dpi":       300,
}


# ── Public entry point ────────────────────────────────────────────────────────

def generate_plots(rows: list[dict], out_dir: Path = _PLOTS_DIR) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    df = pd.DataFrame(rows).sort_values(["simulator", "distance_km"])
    simulators = sorted(df["simulator"].unique())

    with plt.rc_context(_RC):
        for metric, label, unit, log_scale in _METRICS:
            _plot_metric(df, simulators, metric, label, unit, log_scale, out_dir)

    print(f"  Plots saved to {out_dir}/")


# ── Internal helpers ──────────────────────────────────────────────────────────

def _style(sim: str, fallback_idx: int) -> dict:
    return _STYLE.get(sim, _FALLBACK[fallback_idx % len(_FALLBACK)])


def _plot_metric(
    df: pd.DataFrame,
    simulators: list[str],
    metric: str,
    label: str,
    unit: str,
    log_scale: bool,
    out_dir: Path,
) -> None:
    mean_col = f"{metric}_mean"
    std_col  = f"{metric}_std"
    if mean_col not in df.columns:
        return

    fig, ax = plt.subplots()

    for idx, sim in enumerate(simulators):
        sub  = df[df["simulator"] == sim].sort_values("distance_km")
        if sub.empty:
            continue

        x    = sub["distance_km"].to_numpy(dtype=float)
        mean = sub[mean_col].to_numpy(dtype=float)
        std  = (
            sub[std_col].to_numpy(dtype=float)
            if std_col in sub.columns
            else np.zeros_like(mean)
        )
        sty  = _style(sim, idx)

        ax.plot(x, mean,
                color=sty["color"], marker=sty["marker"],
                label=sim, zorder=3)

        # Only draw the band where variance is non-zero
        if std.any():
            ax.fill_between(x, mean - std, mean + std,
                            color=sty["color"], alpha=0.15, zorder=2)

    # ── Axes labels & title ───────────────────────────────────────────────────
    ax.set_xlabel("Distance (km)")
    ax.set_ylabel(f"{label}  ({unit})")
    ax.set_title(f"{label} vs. Distance")

    # ── x-ticks at data points only ───────────────────────────────────────────
    x_ticks = sorted(df["distance_km"].unique())
    ax.set_xticks(x_ticks)
    ax.xaxis.set_major_formatter(
        ticker.FuncFormatter(lambda v, _: f"{int(v)}")
    )

    # ── Log scale ─────────────────────────────────────────────────────────────
    if log_scale and (df[mean_col] > 0).any():
        ax.set_yscale("log")
        ax.yaxis.set_major_formatter(
            ticker.LogFormatterSciNotation(labelOnlyBase=False)
        )

    # ── Grid (horizontal only) ────────────────────────────────────────────────
    ax.yaxis.grid(True, linestyle="--", alpha=0.25)
    ax.xaxis.grid(False)
    ax.set_axisbelow(True)

    ax.legend(loc="upper right")
    fig.tight_layout()

    out_path = out_dir / f"{metric}.png"
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved: {out_path}")
