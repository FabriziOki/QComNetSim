#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import os

# Create plots directory
os.makedirs('data/plots', exist_ok=True)

# Read comparison data
df = pd.read_csv('data/comparison.csv')

# Separate by simulator
qcom = df[df['simulator'] == 'QComNetSim'].sort_values('distance_km')
seq = df[df['simulator'] == 'SeQUeNCe'].sort_values('distance_km')

# Style settings
plt.style.use('seaborn-v0_8-darkgrid')
colors = {'QComNetSim': '#2E86AB', 'SeQUeNCe': '#A23B72'}
markers = {'QComNetSim': 'o', 'SeQUeNCe': 's'}

# Plot 1: Success Rate
fig, ax = plt.subplots(figsize=(8, 5))
ax.plot(qcom['distance_km'], qcom['success_rate'],  # Remove * 100
        marker=markers['QComNetSim'], color=colors['QComNetSim'], 
        linewidth=2, markersize=8, label='QComNetSim')
ax.plot(seq['distance_km'], seq['success_rate'],  # Remove * 100
        marker=markers['SeQUeNCe'], color=colors['SeQUeNCe'], 
        linewidth=2, markersize=8, label='SeQUeNCe')
ax.set_xlabel('Distance (km)', fontsize=12, fontweight='bold')
ax.set_ylabel('Success Rate', fontsize=12, fontweight='bold')  # Remove (%)
ax.set_title('Entanglement Success Rate vs Distance', fontsize=14, fontweight='bold')
ax.set_ylim([0, 1])  # Add this line
ax.legend(fontsize=11)
ax.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('data/plots/success_rate.png', dpi=300, bbox_inches='tight')
print("✓ Saved: data/plots/success_rate.png")
plt.close()

# Plot 2: Throughput
fig, ax = plt.subplots(figsize=(8, 5))
ax.plot(qcom['distance_km'], qcom['throughput'], 
        marker=markers['QComNetSim'], color=colors['QComNetSim'], 
        linewidth=2, markersize=8, label='QComNetSim')
ax.plot(seq['distance_km'], seq['throughput'], 
        marker=markers['SeQUeNCe'], color=colors['SeQUeNCe'], 
        linewidth=2, markersize=8, label='SeQUeNCe')
ax.set_xlabel('Distance (km)', fontsize=12, fontweight='bold')
ax.set_ylabel('Throughput (pairs/sec)', fontsize=12, fontweight='bold')
ax.set_title('Entanglement Generation Throughput', fontsize=14, fontweight='bold')
ax.legend(fontsize=11)
ax.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('data/plots/throughput.png', dpi=300, bbox_inches='tight')
print("✓ Saved: data/plots/throughput.png")
plt.close()

# Plot 3: Fidelity
fig, ax = plt.subplots(figsize=(8, 5))
ax.plot(qcom['distance_km'], qcom['avg_fidelity'], 
        marker=markers['QComNetSim'], color=colors['QComNetSim'], 
        linewidth=2, markersize=8, label='QComNetSim')
ax.plot(seq['distance_km'], seq['avg_fidelity'], 
        marker=markers['SeQUeNCe'], color=colors['SeQUeNCe'], 
        linewidth=2, markersize=8, label='SeQUeNCe')
ax.set_xlabel('Distance (km)', fontsize=12, fontweight='bold')
ax.set_ylabel('Average Fidelity', fontsize=12, fontweight='bold')
ax.set_title('Entanglement Fidelity vs Distance', fontsize=14, fontweight='bold')
ax.set_ylim([0, 1])
ax.legend(fontsize=11)
ax.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('data/plots/fidelity.png', dpi=300, bbox_inches='tight')
print("✓ Saved: data/plots/fidelity.png")
plt.close()

# Plot 4: Quantum Memory Usage
fig, ax = plt.subplots(figsize=(8, 5))
ax.plot(qcom['distance_km'], qcom['memory_used'], 
        marker=markers['QComNetSim'], color=colors['QComNetSim'], 
        linewidth=2, markersize=8, label='QComNetSim')
ax.plot(seq['distance_km'], seq['memory_used'], 
        marker=markers['SeQUeNCe'], color=colors['SeQUeNCe'], 
        linewidth=2, markersize=8, label='SeQUeNCe')
ax.set_xlabel('Distance (km)', fontsize=12, fontweight='bold')
ax.set_ylabel('Quantum Memories Used', fontsize=12, fontweight='bold')
ax.set_title('Quantum Memory Consumption', fontsize=14, fontweight='bold')
ax.legend(fontsize=11)
ax.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('data/plots/quantum_memory.png', dpi=300, bbox_inches='tight')
print("✓ Saved: data/plots/quantum_memory.png")
plt.close()

print("\n✓ All plots generated successfully!")
