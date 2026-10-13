import subprocess
import unittest
from pathlib import Path

class StartupTabs(unittest.TestCase):
    def test_tab_interaction(self):
        subprocess.run(["node", "tests/startup_tabs_runtime.cjs"], cwd=Path(__file__).resolve().parents[1], check=True, capture_output=True)
