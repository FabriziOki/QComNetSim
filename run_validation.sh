#!/usr/bin/env bash
# Run the full validation pipeline.
# Usage: ./run_validation.sh [validation.toml] [--simulators qcomnetsim,sequence]

set -euo pipefail

CONFIG="${1:-validation.toml}"
shift 2>/dev/null || true

python src/validation/orchestrator.py "$CONFIG" "$@"
