import heapq
import time
import csv
from dataclasses import dataclass
from typing import List

@dataclass
class Event:
    time: float
    event_type: str
    node_id: int
    
    def __lt__(self, other):
        return self.time < other.time

class EventScheduler:
    def __init__(self):
        self.queue = []
        
    def schedule(self, event: Event):
        heapq.heappush(self.queue, event)
        
    def next_event(self):
        if self.queue:
            return heapq.heappop(self.queue)
        return None
        
    def has_events(self):
        return len(self.queue) > 0

def benchmark_insert(size: int) -> float:
    """Benchmark just inserting events"""
    scheduler = EventScheduler()
    
    start = time.perf_counter()
    for i in range(size):
        event = Event(
            time=i * 0.001,
            event_type="EntanglementGeneration",
            node_id=i % 10
        )
        scheduler.schedule(event)
    end = time.perf_counter()
    
    return end - start

def benchmark_insert_remove(size: int) -> float:
    """Benchmark inserting and removing events"""
    scheduler = EventScheduler()
    
    start = time.perf_counter()
    
    # Insert
    for i in range(size):
        event = Event(
            time=i * 0.001,
            event_type="EntanglementGeneration",
            node_id=i % 10
        )
        scheduler.schedule(event)
    
    # Remove
    while scheduler.has_events():
        scheduler.next_event()
    
    end = time.perf_counter()
    
    return end - start

def run_benchmarks():
    sizes = [100, 1_000, 10_000, 100_000]
    results = []
    
    print("Python Event Scheduling Benchmark")
    print("=" * 50)
    
    for size in sizes:
        insert_times = []
        insert_remove_times = []
        
        for _ in range(5):
            insert_times.append(benchmark_insert(size))
            insert_remove_times.append(benchmark_insert_remove(size))
        
        avg_insert = sum(insert_times) / len(insert_times)
        avg_insert_remove = sum(insert_remove_times) / len(insert_remove_times)
        
        print(f"\nSize: {size:,}")
        print(f"  Insert:        {avg_insert*1000:.3f} ms")
        print(f"  Insert+Remove: {avg_insert_remove*1000:.3f} ms")
        
        # Store results
        results.append({
            "language": "Python",
            "benchmark": "Insert",
            "size": size,
            "time_ms": avg_insert * 1000
        })
        results.append({
            "language": "Python",
            "benchmark": "Insert+Remove",
            "size": size,
            "time_ms": avg_insert_remove * 1000
        })
    
    # Save to CSV
    save_to_csv(results)
    print("\nSaved Python results to benches/results/python_scheduling_results.csv")

def save_to_csv(results, filename="../benches/results/python_scheduling_results.csv"):
    """Append Python results to CSV"""
    import os
    file_exists = os.path.exists(filename)
    
    with open(filename, 'a', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=["language", "benchmark", "size", "time_ms"])
        
        if not file_exists:
            writer.writeheader()
        
        for row in results:
            writer.writerow(row)

if __name__ == "__main__":
    run_benchmarks()

