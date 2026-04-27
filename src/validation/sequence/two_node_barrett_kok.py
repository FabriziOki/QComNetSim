"""
Two-node Barrett-Kok entanglement generation experiment for SeQUeNCe.

Hardware parameters (fidelity, efficiency, coherence_time, BSM efficiency)
are read from the params dict passed by the validation runner, making this
script usable for both Erbium and NV-center hardware profiles without code changes.
"""

import time

from sequence.components.memory import Memory
from sequence.components.optical_channel import ClassicalChannel, QuantumChannel
from sequence.entanglement_management.generation import EntanglementGenerationA
from sequence.kernel.timeline import Timeline
from sequence.message import Message
from sequence.topology.node import BSMNode, Node


def _memory_params(params: dict) -> dict:
    return {
        "fidelity":    params.get("memory_fidelity", 0.95),
        "frequency":   params.get("generation_frequency_khz", 2.0) * 1e3,
        "efficiency":  params.get("memory_efficiency", 0.9),
        "coherence_s": params.get("coherence_time_ms", 100.0) * 1e-3,  # Memory expects seconds
    }


class SimpleManager:
    def __init__(self, owner, memo_name):
        self.owner = owner
        self.memo_name = memo_name
        self.raw_counter = 0
        self.ent_counter = 0
        self.fidelities = []

    def update(self, protocol, memory, state):
        if state == "RAW":
            self.raw_counter += 1
            memory.reset()
        else:
            self.ent_counter += 1
            self.fidelities.append(memory.fidelity)

    def create_protocol(self, middle: str, other: str):
        self.owner.protocols = [
            EntanglementGenerationA.create(
                self.owner,
                f"{self.owner.name}.eg",
                middle,
                other,
                self.owner.components[self.memo_name],
            )
        ]


class EntangleGenNode(Node):
    def __init__(self, name: str, tl: Timeline, mp: dict):
        super().__init__(name, tl)
        self.memo_name = f"{name}.memo"
        memory = Memory(
            self.memo_name, tl,
            fidelity=mp["fidelity"],
            frequency=mp["frequency"],
            efficiency=mp["efficiency"],
            coherence_time=mp["coherence_s"],
            wavelength=500,
        )
        memory.owner = self
        memory.add_receiver(self)
        self.add_component(memory)
        self.resource_manager = SimpleManager(self, self.memo_name)

    def init(self):
        self.get_components_by_type("Memory")[0].reset()

    def receive_message(self, src: str, msg: Message) -> None:
        self.protocols[0].received_message(src, msg)

    def get(self, photon, **kwargs):
        self.send_qubit(kwargs["dst"], photon)


def _pair_protocol(node1: Node, node2: Node):
    p1 = node1.protocols[0]
    p2 = node2.protocols[0]
    m1 = node1.get_components_by_type("Memory")[0].name
    m2 = node2.get_components_by_type("Memory")[0].name
    p1.set_others(p2.name, node2.name, [m2])
    p2.set_others(p1.name, node1.name, [m1])


def run_experiment(
    distance_m: float,
    attenuation: float = 0.0002,
    num_attempts: int = 50,
    params: dict | None = None,
    seed: int = 0,
) -> dict:
    """
    Run a two-node Barrett-Kok entanglement generation experiment.

    Args:
        distance_m:   per-link distance in metres.
        attenuation:  channel attenuation in dB/m.
        num_attempts: number of generation rounds.
        params:       hardware parameter dict from the validation config translator.
        seed:         RNG seed — each seed offset by 10 to avoid overlap across trials.

    Returns:
        Standard metrics dict compatible with the validation framework.
    """
    if params is None:
        params = {}

    mp = _memory_params(params)
    bsm_eff = params.get("bsm_efficiency", 0.9)

    tl = Timeline()

    node1 = EntangleGenNode("node1", tl, mp)
    node2 = EntangleGenNode("node2", tl, mp)
    bsm_node = BSMNode("bsm_node", tl, ["node1", "node2"])

    node1.set_seed(seed * 10 + 0)
    node2.set_seed(seed * 10 + 1)
    bsm_node.set_seed(seed * 10 + 2)

    bsm_node.get_components_by_type("SingleAtomBSM")[0].update_detectors_params(
        "efficiency", bsm_eff
    )

    qc1 = QuantumChannel("qc1", tl, attenuation=attenuation, distance=distance_m)
    qc2 = QuantumChannel("qc2", tl, attenuation=attenuation, distance=distance_m)
    qc1.set_ends(node1, bsm_node.name)
    qc2.set_ends(node2, bsm_node.name)

    # Classical channel delay: fiber propagation at ~2×10⁸ m/s.
    # Using physics-based delay (not a fixed constant) ensures the protocol
    # completes within the memory coherence window for all hardware profiles.
    cc_delay = max(1, int(distance_m / 2e8 * 1e12))  # ps, minimum 1 ps

    nodes = [node1, node2, bsm_node]
    for i, n1 in enumerate(nodes):
        for j, n2 in enumerate(nodes):
            if i != j:
                cc = ClassicalChannel(
                    f"cc_{n1.name}_{n2.name}", tl,
                    distance=distance_m,
                    delay=cc_delay,
                )
                cc.set_ends(n1, n2.name)

    node1.resource_manager.create_protocol("bsm_node", "node2")
    node2.resource_manager.create_protocol("bsm_node", "node1")
    _pair_protocol(node1, node2)

    tl.init()

    t0_wall = time.perf_counter()

    for _ in range(num_attempts):
        tl.time = tl.now() + 1e11

        # Reset memories (cancels stale expiration events) and clear quantum-channel
        # send_bins (prevents stale time-bin reservations from causing the schedule_qubit
        # assertion to fire when coherence_time is short, e.g. 10 ms NV-center).
        node1.get_components_by_type("Memory")[0].reset()
        node2.get_components_by_type("Memory")[0].reset()
        qc1.send_bins.clear()
        qc2.send_bins.clear()

        node1.resource_manager.create_protocol("bsm_node", "node2")
        node2.resource_manager.create_protocol("bsm_node", "node1")
        _pair_protocol(node1, node2)

        node1.protocols[0].start()
        node2.protocols[0].start()
        tl.run()

    wall_ms = (time.perf_counter() - t0_wall) * 1000

    successes = node1.resource_manager.ent_counter
    total = successes + node1.resource_manager.raw_counter
    fidelities = node1.resource_manager.fidelities
    sim_time_s = tl.now() / 1e12

    return {
        "distance_m":   distance_m,
        "distance_km":  distance_m / 1000,
        "successes":    successes,
        "success_rate": successes / total if total > 0 else 0.0,
        "avg_fidelity": sum(fidelities) / len(fidelities) if fidelities else 0.0,
        "fidelities":   fidelities,
        "throughput":   successes / sim_time_s if sim_time_s > 0 else 0.0,
        "memory_used":  successes * 2,
        "runtime_ms":   wall_ms,
    }


if __name__ == "__main__":
    for km in [1, 5, 10, 20, 50]:
        r = run_experiment(km * 1000, attenuation=0.0002, num_attempts=100)
        print(f"{km:2d} km  rate={r['success_rate']*100:.1f}%  F={r['avg_fidelity']:.4f}  tp={r['throughput']:.2f}")
