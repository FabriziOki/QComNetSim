from .base import SimulatorRunner


class SimQNRunner(SimulatorRunner):
    """Stub — not yet implemented."""

    def name(self) -> str:
        return "SimQN"

    def run(self, params: dict, seed: int) -> list[dict]:
        raise NotImplementedError(
            "SimQN runner is not implemented yet. "
            "See TODO.md for the integration plan."
        )
