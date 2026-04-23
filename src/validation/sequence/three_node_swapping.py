"""
3-node linear chain with entanglement swapping in SeQUeNCe.

Topology:  Alice ── BSM_AB ── Repeater ── BSM_BC ── Bob

Protocol per round:
  1. Simultaneously attempt generation on Alice-Repeater and Repeater-Bob links.
  2. RepeaterManager detects both links ready and fires EntanglementSwappingA.
  3. Alice and Bob's EntanglementSwappingB receive the swap result and update
     their memories to the end-to-end fidelity.
  4. Outcome is recorded; all memories and protocols reset for next round.

link_distance_m is the per-hop distance; total path = 2 × link_distance_m.
This matches QComNetSim's linear-topology convention where distance_km is per-link.
"""

import time

from sequence.components.memory import Memory
from sequence.components.optical_channel import ClassicalChannel, QuantumChannel
from sequence.entanglement_management.generation import EntanglementGenerationA
from sequence.entanglement_management.swapping import (
    EntanglementSwappingA,
    EntanglementSwappingB,
)
from sequence.kernel.timeline import Timeline
from sequence.message import Message
from sequence.resource_management.memory_manager import MemoryInfo
from sequence.topology.node import BSMNode, Node


def _entangled(state) -> bool:
    return state == MemoryInfo.ENTANGLED or str(state) == "ENTANGLED"


# ---------------------------------------------------------------------------
# Node classes
# ---------------------------------------------------------------------------

class RoutingNode(Node):
    """Base node that routes received_message to the correct protocol.

    Two-pass routing:
      1. By msg.receiver name — covers NEGOTIATE, NEGOTIATE_ACK, swap messages.
      2. By p.middle == src — covers MEAS_RES, which SeQUeNCe sends with receiver=None.
         A repeater has two generation protocols (one per link/BSM); without this pass
         both MEAS_RES messages would fall to protocols[0] and the wrong link would be fed.
    """

    def receive_message(self, src: str, msg: Message) -> None:
        # Pass 1: explicit receiver field (NEGOTIATE, NEGOTIATE_ACK, swap messages)
        if getattr(msg, "receiver", None) is not None:
            for p in self.protocols:
                if p.name == msg.receiver:
                    p.received_message(src, msg)
                    return

        # Pass 2: route by source BSM (MEAS_RES has receiver=None in SeQUeNCe)
        for p in self.protocols:
            if getattr(p, "middle", None) == src:
                p.received_message(src, msg)
                return

        # Fallback
        if self.protocols:
            self.protocols[0].received_message(src, msg)

    def get(self, photon, **kwargs):
        self.send_qubit(kwargs["dst"], photon)


class EndNode(RoutingNode):
    def __init__(self, name: str, tl: Timeline, mp: dict):
        super().__init__(name, tl)
        self.memo_name = f"{name}.memo"
        memo = Memory(
            self.memo_name, tl,
            fidelity=mp["fidelity"],
            frequency=mp["frequency_hz"],
            efficiency=mp["efficiency"],
            coherence_time=mp["coherence_s"],
            wavelength=500,
        )
        memo.owner = self
        memo.add_receiver(self)
        self.add_component(memo)
        self.resource_manager = EndNodeManager(self, self.memo_name)

    def init(self):
        self.components[self.memo_name].reset()


class RepeaterNode(RoutingNode):
    def __init__(self, name: str, tl: Timeline, mp: dict):
        super().__init__(name, tl)
        self.left_memo_name = f"{name}.memo_left"
        self.right_memo_name = f"{name}.memo_right"
        for mname in [self.left_memo_name, self.right_memo_name]:
            memo = Memory(
                mname, tl,
                fidelity=mp["fidelity"],
                frequency=mp["frequency_hz"],
                efficiency=mp["efficiency"],
                coherence_time=mp["coherence_s"],
                wavelength=500,
            )
            memo.owner = self
            memo.add_receiver(self)
            self.add_component(memo)
        left = self.components[self.left_memo_name]
        right = self.components[self.right_memo_name]
        self.resource_manager = RepeaterManager(self, left, right)

    def init(self):
        for mname in [self.left_memo_name, self.right_memo_name]:
            self.components[mname].reset()


# ---------------------------------------------------------------------------
# Resource managers
# ---------------------------------------------------------------------------

class EndNodeManager:
    """
    Tracks end-to-end results at Alice and Bob.

    Generation successes are left alone (memory stays entangled for swapping).
    Generation failures reset the memory.
    Swap results (from EntanglementSwappingB) are recorded as E2E outcomes.
    """

    def __init__(self, owner, memo_name: str):
        self.owner = owner
        self.memo_name = memo_name
        self.e2e_fidelities: list[float] = []

    def reset_round(self):
        self.e2e_fidelities = []

    def update(self, protocol, memory, state):
        if isinstance(protocol, EntanglementSwappingB):
            if _entangled(state):
                self.e2e_fidelities.append(memory.fidelity)
        else:
            # Generation result — reset memory on failure so it's clean for next round
            if not _entangled(state):
                memory.reset()


class RepeaterManager:
    """
    Tracks both link memories at the repeater.
    When both are entangled, creates and starts EntanglementSwappingA.
    """

    def __init__(self, owner, left_memo, right_memo):
        self.owner = owner
        self.left_memo = left_memo
        self.right_memo = right_memo
        self._alice = None
        self._bob = None
        self.left_ready = False
        self.right_ready = False
        self.swap_triggered = False

    def bind_endpoints(self, alice: EndNode, bob: EndNode):
        self._alice = alice
        self._bob = bob

    def reset_round(self):
        self.left_ready = False
        self.right_ready = False
        self.swap_triggered = False

    def update(self, protocol, memory, state):
        if isinstance(protocol, EntanglementSwappingA):
            # Repeater's memories were consumed — nothing to track
            return

        if _entangled(state):
            if memory.name == self.left_memo.name:
                self.left_ready = True
            elif memory.name == self.right_memo.name:
                self.right_ready = True

            if self.left_ready and self.right_ready and not self.swap_triggered:
                self.swap_triggered = True
                self._do_swap()
        else:
            memory.reset()
            if memory.name == self.left_memo.name:
                self.left_ready = False
            elif memory.name == self.right_memo.name:
                self.right_ready = False

    def _do_swap(self):
        """
        Both link memories are entangled. Create SwappingA/B protocols and start.

        EntanglementSwappingA.__init__ reads left_memo.entangled_memory['node_id']
        (== alice.name) and right_memo.entangled_memory['node_id'] (== bob.name)
        to set self.left_node / self.right_node, which set_others then validates.
        """
        alice = self._alice
        bob = self._bob
        repeater = self.owner

        alice_memo = alice.components[alice.memo_name]
        bob_memo = bob.components[bob.memo_name]

        swap_b_alice = EntanglementSwappingB(alice, f"{alice.name}.swap_b", alice_memo)
        swap_b_bob = EntanglementSwappingB(bob, f"{bob.name}.swap_b", bob_memo)
        swap_a = EntanglementSwappingA(
            repeater, f"{repeater.name}.swap_a",
            self.left_memo, self.right_memo,
            success_prob=0.99, degradation=0.95,
        )

        swap_a.set_others(swap_b_alice.name, alice.name, [alice_memo.name])
        swap_a.set_others(swap_b_bob.name, bob.name, [bob_memo.name])
        swap_b_alice.set_others(swap_a.name, repeater.name, [self.left_memo.name])
        swap_b_bob.set_others(swap_a.name, repeater.name, [self.right_memo.name])

        alice.protocols.append(swap_b_alice)
        bob.protocols.append(swap_b_bob)
        repeater.protocols.append(swap_a)

        swap_a.start()


# ---------------------------------------------------------------------------
# Experiment
# ---------------------------------------------------------------------------

def _memory_params(params: dict) -> dict:
    return {
        "fidelity":    params.get("memory_fidelity", 0.95),
        "frequency_hz": params.get("generation_frequency_khz", 2.0) * 1e3,
        "efficiency":  params.get("memory_efficiency", 0.9),
        "coherence_s": params.get("coherence_time_ms", 100.0) * 1e-3,  # Memory expects seconds
    }


def _make_gen(node, name, bsm_name, other_name, memo):
    p = EntanglementGenerationA.create(node, name, bsm_name, other_name, memo)
    node.protocols.append(p)
    return p


def _pair_gen(p1, node1, memo1, p2, node2, memo2):
    p1.set_others(p2.name, node2.name, [memo2.name])
    p2.set_others(p1.name, node1.name, [memo1.name])


def run_experiment(
    link_distance_m: float,
    attenuation: float = 0.0002,
    num_attempts: int = 50,
    params: dict | None = None,
    seed: int = 0,
) -> dict:
    """
    Run a 3-node linear-chain experiment with entanglement swapping.

    Args:
        link_distance_m: per-hop distance in metres (total path = 2×).
        attenuation: channel loss in dB/m.
        num_attempts: number of rounds to simulate.
        params: hardware parameter dict from the validation config translator.

    Returns:
        Standard metrics dict compatible with the validation framework.
    """
    if params is None:
        params = {}

    mp = _memory_params(params)
    bsm_eff = params.get("bsm_efficiency", 0.9)

    tl = Timeline()

    alice = EndNode("alice", tl, mp)
    repeater = RepeaterNode("repeater", tl, mp)
    bob = EndNode("bob", tl, mp)

    alice.set_seed(seed * 10 + 0)
    repeater.set_seed(seed * 10 + 1)
    bob.set_seed(seed * 10 + 2)

    bsm_ab = BSMNode("bsm_ab", tl, ["alice", "repeater"])
    bsm_bc = BSMNode("bsm_bc", tl, ["repeater", "bob"])
    bsm_ab.set_seed(seed * 10 + 3)
    bsm_bc.set_seed(seed * 10 + 4)

    for bsm in (bsm_ab, bsm_bc):
        bsm.get_components_by_type("SingleAtomBSM")[0].update_detectors_params(
            "efficiency", bsm_eff
        )

    # Quantum channels (one per node-to-BSM link)
    for name, sender, bsm_name in [
        ("qc_a_ab",  alice,    bsm_ab.name),
        ("qc_r_ab",  repeater, bsm_ab.name),
        ("qc_r_bc",  repeater, bsm_bc.name),
        ("qc_b_bc",  bob,      bsm_bc.name),
    ]:
        qc = QuantumChannel(name, tl, attenuation=attenuation, distance=link_distance_m)
        qc.set_ends(sender, bsm_name)

    # Classical channels — full mesh.
    # Delay is physics-based (fiber at ~2×10⁸ m/s) so the protocol completes
    # within the memory coherence window for all hardware profiles.
    cc_delay = max(1, int(link_distance_m / 2e8 * 1e12))  # per-hop delay in ps

    all_nodes = [alice, repeater, bob, bsm_ab, bsm_bc]
    for i, n1 in enumerate(all_nodes):
        for j, n2 in enumerate(all_nodes):
            if i != j:
                cc = ClassicalChannel(
                    f"cc_{n1.name}_{n2.name}", tl,
                    distance=link_distance_m * 2,
                    delay=cc_delay,
                )
                cc.set_ends(n1, n2.name)

    repeater.resource_manager.bind_endpoints(alice, bob)
    tl.init()

    e2e_successes = 0
    e2e_fidelities: list[float] = []

    t0_wall = time.perf_counter()

    for _ in range(num_attempts):
        # Reset memories
        alice.components[alice.memo_name].reset()
        for mname in [repeater.left_memo_name, repeater.right_memo_name]:
            repeater.components[mname].reset()
        bob.components[bob.memo_name].reset()

        # Reset manager state and per-round fidelity list
        repeater.resource_manager.reset_round()
        alice.resource_manager.reset_round()
        bob.resource_manager.reset_round()

        # Clear protocols from previous round
        alice.protocols.clear()
        repeater.protocols.clear()
        bob.protocols.clear()

        # Retrieve memories
        alice_memo = alice.components[alice.memo_name]
        left_memo = repeater.components[repeater.left_memo_name]
        right_memo = repeater.components[repeater.right_memo_name]
        bob_memo = bob.components[bob.memo_name]

        # Create generation protocols for both links simultaneously
        p_alice    = _make_gen(alice,    "alice.eg",        "bsm_ab", "repeater", alice_memo)
        p_rep_left = _make_gen(repeater, "repeater.eg_left","bsm_ab", "alice",    left_memo)
        p_rep_rght = _make_gen(repeater, "repeater.eg_right","bsm_bc","bob",      right_memo)
        p_bob      = _make_gen(bob,      "bob.eg",          "bsm_bc", "repeater", bob_memo)

        _pair_gen(p_alice,    alice,    alice_memo, p_rep_left, repeater, left_memo)
        _pair_gen(p_rep_rght, repeater, right_memo, p_bob,     bob,      bob_memo)

        # Clear stale time-bin reservations from quantum channels before each round.
        # Without this, send_bins accumulates across rounds and schedule_qubit may
        # return a different bin than requested, triggering SeQUeNCe's emit-time assert.
        for node in (alice, repeater, bob):
            for qc in node.qchannels.values():
                qc.send_bins.clear()

        # Advance simulated clock (100 ms gap between rounds for decoherence effects)
        tl.time = tl.now() + 1e11

        p_alice.start()
        p_rep_left.start()
        p_rep_rght.start()
        p_bob.start()

        tl.run()

        # Record if this round produced an end-to-end pair
        if alice.resource_manager.e2e_fidelities:
            e2e_successes += 1
            e2e_fidelities.append(alice.resource_manager.e2e_fidelities[-1])

    wall_ms = (time.perf_counter() - t0_wall) * 1000
    sim_time_s = tl.now() / 1e12

    success_rate = e2e_successes / num_attempts if num_attempts > 0 else 0.0
    avg_fidelity = sum(e2e_fidelities) / len(e2e_fidelities) if e2e_fidelities else 0.0
    throughput   = e2e_successes / sim_time_s if sim_time_s > 0 else 0.0

    return {
        "link_distance_m": link_distance_m,
        "distance_km":     link_distance_m / 1000,
        "successes":       e2e_successes,
        "total_attempts":  num_attempts,
        "success_rate":    success_rate,
        "avg_fidelity":    avg_fidelity,
        "fidelities":      e2e_fidelities,
        "throughput":      throughput,
        "memory_used":     e2e_successes * 2,
        "runtime_ms":      wall_ms,
    }


if __name__ == "__main__":
    result = run_experiment(
        link_distance_m=1000,
        attenuation=0.0002,
        num_attempts=50,
    )
    print(f"Success rate : {result['success_rate']*100:.1f}%")
    print(f"Avg fidelity : {result['avg_fidelity']:.4f}")
    print(f"Throughput   : {result['throughput']:.2f} pairs/s")
    print(f"Wall clock   : {result['runtime_ms']:.1f} ms")
