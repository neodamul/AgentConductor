import subprocess
import unittest
from pathlib import Path

class AvailabilityTimer(unittest.TestCase):
    def test_deadline_countdown_and_reassignment(self):
        subprocess.run(["node", "tests/availability_timer_runtime.cjs"], cwd=Path(__file__).resolve().parents[1], check=True, capture_output=True)
