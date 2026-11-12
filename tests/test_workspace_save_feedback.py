"""#905/#906: run the assembled-product interaction regression in CI."""
from pathlib import Path
import subprocess
import unittest

class WorkspaceSaveFeedback(unittest.TestCase):
    def test_workspace_save_and_required_states(self):
        result = subprocess.run(["node", "tests/workspace_save_905.cjs"],
            cwd=Path(__file__).resolve().parents[1], capture_output=True,
            text=True, timeout=120)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
