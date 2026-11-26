import shutil
import subprocess
import unittest
from pathlib import Path


class GoalReferenceList(unittest.TestCase):
    def run_node(self, script):
        node = shutil.which("node")
        if not node:
            self.skipTest("Node runtime unavailable")
        root = Path(__file__).resolve().parents[1]
        result = subprocess.run(
            [node, "tests/" + script], cwd=root, capture_output=True, text=True, timeout=60
        )
        if result.returncode == 77:
            self.skipTest("Chrome runtime unavailable")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_goal_reference_list(self):
        self.run_node("goal_reference_list_runtime.cjs")

    def test_goal_reference_panel(self):
        self.run_node("goal_reference_panel_runtime.cjs")
