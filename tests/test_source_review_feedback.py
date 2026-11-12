"""#916/#917: source review regressions through the assembled UI."""
from pathlib import Path
import subprocess
import unittest

class SourceReviewFeedback(unittest.TestCase):
    def test_guide_and_source_card_selection(self):
        result = subprocess.run(["node", "tests/source_review_916.cjs"],
            cwd=Path(__file__).resolve().parents[1], capture_output=True,
            text=True, timeout=180)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
