from abc import ABC, abstractmethod


class SimulatorRunner(ABC):
    """
    Interface every simulator adapter must implement.

    run() returns one row per distance point, conforming to the shared
    output schema:

        simulator     : str   - runner name
        distance_km   : float
        success_rate  : float - [0, 1]
        avg_fidelity  : float - [0, 1]
        throughput    : float - entangled pairs / second
        memory_used   : int   - quantum memories consumed
        wall_clock_ms : float - real execution time for this distance point
        seed          : int
    """

    @abstractmethod
    def name(self) -> str:
        """Human-readable simulator label used in CSVs and plots."""
        ...

    @abstractmethod
    def run(self, params: dict, seed: int) -> list[dict]:
        """
        Run the simulation for every distance in params["distances_km"].

        Args:
            params: Standardised parameter dict produced by ConfigTranslator.
            seed:   RNG seed for this trial.

        Returns:
            List of row dicts (one per distance), each matching the schema
            above. Missing optional fields should be filled with None.
        """
        ...
