from sequence.topology.node import Node
from sequence.components.memory import Memory
from sequence.entanglement_management.generation import EntanglementGenerationA
from sequence.kernel.timeline import Timeline
from sequence.message import Message
from sequence.topology.node import BSMNode
from sequence.components.optical_channel import QuantumChannel, ClassicalChannel
import time

class SimpleManager:
    def __init__(self, owner, memo_name):
        self.owner = owner
        self.memo_name = memo_name
        self.raw_counter = 0
        self.ent_counter = 0
        self.fidelities = []  # Track fidelity over attempts

    def update(self, protocol, memory, state):
        if state == 'RAW':
            self.raw_counter += 1
            memory.reset()
        else:
            self.ent_counter += 1
            self.fidelities.append(memory.fidelity)  # Record fidelity

    def create_protocol(self, middle: str, other: str):
        self.owner.protocols = [
            EntanglementGenerationA.create(
                self.owner, 
                '%s.eg' % self.owner.name, 
                middle, 
                other,
                self.owner.components[self.memo_name]
            )
        ]

class EntangleGenNode(Node):
    def __init__(self, name: str, tl: Timeline):
        super().__init__(name, tl)
        memo_name = '%s.memo' % name
        
        # Realistic memory parameters
        memory = Memory(
            memo_name, 
            tl, 
            fidelity=0.95,        # Initial operation fidelity (realistic)
            frequency=2000,        # 2 kHz generation rate
            efficiency=0.9,        # 90% efficiency (realistic)
            coherence_time=1e11,   # 100ms coherence time (realistic)
            wavelength=500,
        )
        memory.owner = self
        memory.add_receiver(self)
        self.add_component(memory)
        self.resource_manager = SimpleManager(self, memo_name)

    def init(self):
        memory = self.get_components_by_type("Memory")[0]
        memory.reset()

    def receive_message(self, src: str, msg: "Message") -> None:
        self.protocols[0].received_message(src, msg)

    def get(self, photon, **kwargs):
        self.send_qubit(kwargs['dst'], photon)

def pair_protocol(node1: Node, node2: Node):
    p1 = node1.protocols[0]
    p2 = node2.protocols[0]
    node1_memo_name = node1.get_components_by_type("Memory")[0].name
    node2_memo_name = node2.get_components_by_type("Memory")[0].name
    p1.set_others(p2.name, node2.name, [node2_memo_name])
    p2.set_others(p1.name, node1.name, [node1_memo_name])

def run_experiment(distance_m, attenuation=0.0002, num_attempts=50):
    """Run entanglement generation experiment with given parameters"""
    
    print(f"\n{'='*70}")
    print(f"Distance: {distance_m}m ({distance_m/1000:.1f}km)")
    print(f"Attenuation: {attenuation} dB/m")
    print(f"Attempts: {num_attempts}")
    print(f"{'='*70}")
    
    start_time = time.time()
    
    tl = Timeline()
    
    node1 = EntangleGenNode('node1', tl)
    node2 = EntangleGenNode('node2', tl)
    bsm_node = BSMNode('bsm_node', tl, ['node1', 'node2'])
    
    node1.set_seed(0)
    node2.set_seed(1)
    bsm_node.set_seed(2)
    
    # Set realistic BSM detector efficiency
    bsm = bsm_node.get_components_by_type("SingleAtomBSM")[0]
    bsm.update_detectors_params('efficiency', 0.9)  # 90% realistic
    
    # CRITICAL: Set realistic attenuation and distance
    qc1 = QuantumChannel('qc1', tl, attenuation=attenuation, distance=distance_m)
    qc2 = QuantumChannel('qc2', tl, attenuation=attenuation, distance=distance_m)
    qc1.set_ends(node1, bsm_node.name)
    qc2.set_ends(node2, bsm_node.name)
    
    nodes = [node1, node2, bsm_node]
    
    # Classical channels for coordination
    for i in range(3):
        for j in range(3):
            if i != j:
                cc = ClassicalChannel(
                    'cc_%s_%s' % (nodes[i].name, nodes[j].name), 
                    tl, 
                    distance=distance_m,
                    delay=1e8  # 100ms classical delay
                )
                cc.set_ends(nodes[i], nodes[j].name)
    
    node1.resource_manager.create_protocol('bsm_node', 'node2')
    node2.resource_manager.create_protocol('bsm_node', 'node1')
    pair_protocol(node1, node2)
    
    memory = node1.get_components_by_type("Memory")[0]
    
    tl.init()
    
    # Run multiple attempts
    for i in range(num_attempts):
        tl.time = tl.now() + 1e11  # Wait 100ms between attempts
        
        node1.resource_manager.create_protocol('bsm_node', 'node2')
        node2.resource_manager.create_protocol('bsm_node', 'node1')
        pair_protocol(node1, node2)
        
        node1.protocols[0].start()
        node2.protocols[0].start()
        tl.run()
    
    wall_time = time.time() - start_time
    
    # Calculate statistics
    successes = node1.resource_manager.ent_counter
    failures = node1.resource_manager.raw_counter
    total = successes + failures
    success_rate = successes / total if total > 0 else 0
    
    fidelities = node1.resource_manager.fidelities
    # Calculate throughput (pairs per second)
    sim_time_seconds = tl.now() / 1e12  # Convert ps to seconds
    throughput = successes / sim_time_seconds if sim_time_seconds > 0 else 0
    
    # Memory usage (quantum memories used, not RAM)
    memory_usage_node1 = sum(1 for m in node1.get_components_by_type("Memory") if hasattr(m, 'entangled_memory'))
    memory_usage_node2 = sum(1 for m in node2.get_components_by_type("Memory") if hasattr(m, 'entangled_memory'))
    total_memory_used = successes * 2
    avg_fidelity = sum(fidelities) / len(fidelities) if fidelities else 0
    
    # Calculate expected loss
    expected_loss_db = attenuation * distance_m
    expected_transmission = 10 ** (-expected_loss_db / 10)
    
    print(f"\n--- Results ---")
    print(f"Successful entanglements: {successes}/{total}")
    print(f"Success rate: {success_rate*100:.1f}%")
    print(f"Average fidelity: {avg_fidelity:.4f}")
    print(f"Throughput: {throughput:.4f} pairs/sec")
    print(f"Quantum memories used: {total_memory_used}")
    if fidelities:
        print(f"Fidelity range: {min(fidelities):.4f} - {max(fidelities):.4f}")
    print(f"Expected channel transmission: {expected_transmission*100:.2f}%")
    print(f"Wall-clock runtime: {wall_time*1000:.2f}ms")
    
    return {
        'distance_m': distance_m,
        'distance_km': distance_m / 1000,
        'attenuation': attenuation,
        'successes': successes,
        'failures': failures,
        'success_rate': success_rate,
        'avg_fidelity': avg_fidelity,
        'fidelities': fidelities,
        'runtime_ms': wall_time * 1000,
        'expected_transmission': expected_transmission,
        'throughput': throughput,
        'memory_used': total_memory_used
    }

if __name__ == "__main__":
    import csv
    
    # Test different distances
    distances = [1000, 5000, 10000, 20000, 50000]  # 1km to 50km
    
    results = []
    for dist in distances:
        result = run_experiment(dist, attenuation=0.0002, num_attempts=100)
        results.append(result)
    
    # Save to CSV
    with open('data/sequence_results.csv', 'w', newline='') as f:
        fieldnames = ['distance_km', 'success_rate', 'avg_fidelity', 
                     'throughput','memory_used']
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        
        for r in results:
            writer.writerow({
                'distance_km': r['distance_km'],
                'success_rate': r['success_rate'],
                'avg_fidelity': r['avg_fidelity'],
                'throughput': r['throughput'],
                'memory_used': r['memory_used'],
            })
    
    print("\n" + "="*70)
    print("All results saved to sequence_distance_test.csv")
    print("="*70)
