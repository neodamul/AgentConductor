import subprocess
import unittest
from pathlib import Path

class DiffReview(unittest.TestCase):
    def test_shortened_preview_approval(self):
        subprocess.run(["node", "tests/diff_review_runtime.cjs"], cwd=Path(__file__).resolve().parents[1], check=True, capture_output=True)
