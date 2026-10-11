"""R-24 / DD-27: tag-triggered publication repeats verification before any Release."""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"


class ReleaseWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.release = (WORKFLOWS / "release.yml").read_text(encoding="utf-8")
        self.validation = (WORKFLOWS / "platform-validation.yml").read_text(encoding="utf-8")

    def test_release_is_triggered_only_by_version_tags(self):
        self.assertRegex(self.release, r"on:\s*\n\s*push:\s*\n\s*tags:\s*\[\"v\*\"\]")
        self.assertNotIn("branches:", self.release)

    def test_publication_reuses_the_full_platform_validation_and_package_smoke(self):
        self.assertIn("workflow_call:", self.validation)
        self.assertIn("uses: ./.github/workflows/platform-validation.yml", self.release)
        publish = self.release[self.release.index("  publish:"):]
        self.assertIn("needs: verify", publish)

    def test_only_the_publish_job_can_write_and_it_attaches_verified_archives(self):
        self.assertRegex(self.release, r"(?m)^permissions:\s*\n\s*contents: read")
        publish = self.release[self.release.index("  publish:"):]
        self.assertIn("contents: write", publish)
        self.assertIn("gh release create", publish)
        self.assertNotIn("--prerelease", publish)
        self.assertIn("SHA256SUMS", publish)
        self.assertIn("*.zip", publish)

    def test_third_party_actions_are_pinned_to_commits(self):
        for workflow in (self.release, self.validation):
            for action in re.findall(r"uses: ([^\s]+)", workflow):
                if action.startswith("./"):
                    continue
                self.assertRegex(action, r"@[0-9a-f]{40}$", action)


if __name__ == "__main__":
    unittest.main()
