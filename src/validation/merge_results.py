#!/usr/bin/env python3
import pandas as pd

# Read both CSVs
qcom = pd.read_csv('data/qcomnetsim_results.csv')
seq = pd.read_csv('data/sequence_results.csv')

# Add simulator column
qcom['simulator'] = 'QComNetSim'
seq['simulator'] = 'SeQUeNCe'

# Combine
combined = pd.concat([qcom, seq], ignore_index=True)

# Reorder columns for clarity
combined = combined[['simulator', 'distance_km', 'success_rate', 
                     'throughput', 'memory_used', 'avg_fidelity']]

# Save
combined.to_csv('data/comparison.csv', index=False)
print("✓ Merged results saved to data/comparison.csv")
print(f"  QComNetSim: {len(qcom)} rows")
print(f"  SeQUeNCe: {len(seq)} rows")
print(f"  Total: {len(combined)} rows")
