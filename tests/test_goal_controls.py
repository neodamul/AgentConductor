import shutil
import subprocess
import unittest
from pathlib import Path
class GoalControls(unittest.TestCase):
    def test_goal_context_requests_and_failure_preservation(self):
        node=shutil.which('node')
        if not node:self.skipTest('Node runtime unavailable')
        root=Path(__file__).resolve().parents[1]
        subprocess.run([node,'tests/goal_controls_runtime.cjs'],cwd=root,check=True,capture_output=True,text=True,timeout=10)
