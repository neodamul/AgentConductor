import hashlib, json, unittest
from pathlib import Path
ROOT = Path("docs/design/approved-goal-ui")
class DesignManifest(unittest.TestCase):
    # GitHub #849: verify-and-capture.py refreshes these hashes after a passing capture.
    def test_manifest_matches_every_design_file(self):
        recorded = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))["artifacts"]
        actual = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"}
        stale = sorted(k for k in recorded.keys() | actual.keys() if recorded.get(k) != actual.get(k))
        self.assertEqual(stale, [], "run docs/design/approved-goal-ui/verify-and-capture.py")
if __name__ == "__main__": unittest.main()
