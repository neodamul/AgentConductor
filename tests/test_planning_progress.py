import subprocess, unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
class PlanningProgress(unittest.TestCase):
    def test_retry_and_cli_activity(self):
        subprocess.run(["node", "tests/planning_progress_runtime.cjs"], cwd=ROOT, check=True, capture_output=True)
