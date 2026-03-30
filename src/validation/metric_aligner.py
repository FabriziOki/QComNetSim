"""
Documents and enforces metric alignment across simulators.

Each simulator may compute the same physical quantity differently.
This module is the single place that records those mappings so they
can be cited in the paper.

Alignment table
---------------
Metric          QComNetSim                  SeQUeNCe
-----------     --------------------------  --------------------------------
success_rate    successes / attempts        ent_counter / (ent_counter + raw_counter)
avg_fidelity    mean(pair.fidelity)         mean(memory.fidelity) at success
throughput      successes / sim_time_s      ent_counter / (tl.now() / 1e12)
memory_used     successes (pairs stored)    successes * 2 (one memory per node)

Notes
-----
- SeQUeNCe memory_used counts individual qubit-memories (2× the pair count).
  We normalise it to pairs so both simulators are comparable.
- Throughput denominators differ: QComNetSim uses a fixed simulation window;
  SeQUeNCe uses accumulated simulation time in picoseconds.
  Both are converted to pairs/second before storage.
"""


def align(rows: list[dict]) -> list[dict]:
    """
    Apply normalisation corrections to raw rows before aggregation.

    Currently: halve SeQUeNCe memory_used so it counts pairs, not qubits.
    """
    for row in rows:
        if row.get("simulator") == "SeQUeNCe" and row.get("memory_used") is not None:
            row["memory_used"] = row["memory_used"] // 2
    return rows
