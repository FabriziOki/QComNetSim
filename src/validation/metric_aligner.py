"""
Documents and enforces metric alignment across simulators.

Each simulator may compute the same physical quantity differently.
This module is the single place that records those mappings so they
can be cited in the paper.

Alignment table
---------------
Metric          QComNetSim                          SeQUeNCe
-----------     ----------------------------------  --------------------------------
success_rate    gen_successes / gen_attempts        ent_counter / (ent_counter + raw_counter)
avg_fidelity    mean(delivered_pair.fidelity)       mean(memory.fidelity) at success
throughput      pairs_delivered / (t_last_ms/1000)  ent_counter / (tl.now() / 1e12)
memory_used     pairs_delivered (pair count)        successes * 2 (qubit-memory count)
wall_clock_ms   Rust Instant elapsed (real time)    Python time.perf_counter elapsed

Notes
-----
- SeQUeNCe memory_used counts individual qubit-memories (2× the pair count).
  We normalise it to pairs so both simulators are comparable.
- Throughput: QComNetSim uses the last delivery timestamp as simulated time;
  SeQUeNCe uses tl.now() (picoseconds). Both are converted to pairs/second.
- QComNetSim does not currently support seeded RNG. Each seed trial is an
  independent run using OS-seeded randomness; variance reflects natural
  stochastic variation, not seed-controlled reproduction.
- wall_clock_ms measures real execution time and is used for P3 performance
  benchmarks, NOT for throughput calculation.
"""


def align(rows: list[dict]) -> list[dict]:
    """
    Apply normalisation corrections to raw rows before aggregation.

    Currently: halve SeQUeNCe memory_used so it counts pairs, not qubits.
    QComNetSim already outputs pairs directly — no correction needed.
    """
    for row in rows:
        if row.get("simulator") == "SeQUeNCe" and row.get("memory_used") is not None:
            row["memory_used"] = row["memory_used"] // 2
    return rows
