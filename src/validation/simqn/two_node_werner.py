"""
Two-node Werner-state entanglement fidelity experiment using SimQN.

Physical model:  WernerStateEntanglement.transfer_error_model applies
  w_out = w_in * exp(-decoherence_rate * length)
to every EPR pair in transit.  Unlike Barrett-Kok (SeQUeNCe / QComNetSim),
every pair arrives — success_rate is always 1.0.  Channel attenuation
manifests as fidelity decay rather than photon loss.

Use this script to validate the Werner-state fidelity formula across three
independent implementations.  Do NOT use it for success-rate or throughput
comparison: the protocol semantics differ fundamentally from Barrett-Kok.
"""

import math
import time
import uuid

from qns.entity.node.app import Application
from qns.entity.node.node import QNode
from qns.entity.qchannel.qchannel import QuantumChannel, RecvQubitPacket
from qns.models.epr.werner import WernerStateEntanglement
from qns.simulator.event import func_to_event
from qns.simulator.simulator import Simulator
from qns.utils.rnd import set_seed as qns_set_seed


class _SenderApp(Application):
    def __init__(self, dst: QNode, interval_s: float, init_fidelity: float, count: int):
        super().__init__()
        self.dst = dst
        self.interval_s = interval_s
        self.init_fidelity = init_fidelity
        self.count = count
        self._sent = 0
        self._qc = None

    def install(self, node: QNode, simulator: Simulator):
        super().install(node, simulator)
        self._qc = node.get_qchannel(self.dst)
        simulator.add_event(func_to_event(simulator.ts, self._fire, by=self))

    def _fire(self):
        if self._sent >= self.count:
            return
        epr = WernerStateEntanglement(fidelity=self.init_fidelity, name=uuid.uuid4().hex)
        self._qc.send(epr, self.dst)
        self._sent += 1
        next_t = self._simulator.tc + self._simulator.time(sec=self.interval_s)
        self._simulator.add_event(func_to_event(next_t, self._fire, by=self))


class _ReceiverApp(Application):
    def __init__(self):
        super().__init__()
        self.fidelities: list[float] = []
        self.add_handler(self._on_recv, [RecvQubitPacket])

    def _on_recv(self, node: QNode, event: RecvQubitPacket):
        self.fidelities.append(event.qubit.fidelity)


def run_experiment(
    distance_m: float,
    attenuation: float = 0.0002,
    num_attempts: int = 50,
    params: dict | None = None,
    seed: int = 0,
) -> dict:
    """
    Run a two-node Werner-state fidelity experiment using SimQN.

    Args:
        distance_m:   channel length in metres.
        attenuation:  fiber loss in dB/m.
        num_attempts: number of EPR pairs to generate.
        params:       hardware parameter dict from the validation config translator.
        seed:         RNG seed (passed to SimQN's global RNG).

    Returns:
        Standard metrics dict.  success_rate is always 1.0; fidelity decays
        with distance following the Werner-state formula.
    """
    if params is None:
        params = {}

    qns_set_seed(seed)

    init_fidelity = params.get("memory_fidelity", 0.95)
    freq_hz = params.get("generation_frequency_khz", 2.0) * 1e3
    interval_s = 1.0 / freq_hz

    # Decoherence rate for the Werner-state transfer model (per metre):
    #   P_photon_survive = 10^(-att_dB/10) = exp(-γ * L)  =>  γ = att_dB_per_m * ln(10)/10
    decoherence_rate = attenuation * math.log(10) / 10

    # Fiber propagation delay (2×10^8 m/s)
    delay_s = distance_m / 2e8

    # Simulation end: all pairs sent + one extra propagation window
    end_s = num_attempts * interval_s + delay_s * 2 + 1.0

    sim = Simulator(0, end_s, accuracy=1_000_000)

    alice = QNode("alice")
    bob = QNode("bob")

    qc = QuantumChannel(
        name="qc",
        length=distance_m,
        decoherence_rate=decoherence_rate,
        delay=delay_s,
    )
    alice.add_qchannel(qc)
    bob.add_qchannel(qc)

    sender = _SenderApp(bob, interval_s, init_fidelity, num_attempts)
    receiver = _ReceiverApp()
    alice.add_apps(sender)
    bob.add_apps(receiver)

    alice.install(sim)
    bob.install(sim)

    t0_wall = time.perf_counter()
    sim.run()
    wall_ms = (time.perf_counter() - t0_wall) * 1000

    fidelities = receiver.fidelities
    successes = len(fidelities)

    return {
        "distance_m":   distance_m,
        "distance_km":  distance_m / 1000,
        "successes":    successes,
        "success_rate": 1.0,
        "avg_fidelity": sum(fidelities) / len(fidelities) if fidelities else 0.0,
        "fidelities":   fidelities,
        "throughput":   successes / end_s if end_s > 0 else 0.0,
        "memory_used":  successes * 2,
        "runtime_ms":   wall_ms,
    }


if __name__ == "__main__":
    erbium = {"memory_fidelity": 0.95, "generation_frequency_khz": 2.0}
    print("=== SimQN Werner-state two-node (Erbium) ===")
    for km in [1, 5, 10, 20, 50]:
        r = run_experiment(km * 1000, attenuation=0.0002, num_attempts=100, params=erbium)
        print(f"{km:2d} km  F={r['avg_fidelity']:.4f}  successes={r['successes']}")
