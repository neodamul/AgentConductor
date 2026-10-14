import subprocess, unittest
from pathlib import Path
class ApprovalFeedback(unittest.TestCase):
    def test_approval_state(self):
        subprocess.run(["node","tests/approval_feedback_runtime.cjs"],cwd=Path(__file__).resolve().parents[1],check=True,capture_output=True)
