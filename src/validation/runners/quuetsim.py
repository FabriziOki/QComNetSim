from .base import SimulatorRunner


class QuNetSimRunner(SimulatorRunner):
    """Stub — not yet implemented."""

    def name(self) -> str:
        return "QuNetSim"

    def run(self, params: dict, seed: int) -> list[dict]:
        raise NotImplementedError(
            "QuNetSim runner is not implemented yet. "
            "See TODO.md for the integration plan."
        )
