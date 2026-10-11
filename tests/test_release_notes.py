"""R-41 / DD-49 / QAS-33: each release publishes its own changelog section."""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "release_notes.py"


def notes(tag, changelog=None):
    args = [sys.executable, "-I", str(SCRIPT), tag]
    if changelog is not None:
        args.append(str(changelog))
    return subprocess.run(args, capture_output=True, text=True)


class ReleaseNotesTests(unittest.TestCase):
    def test_section_for_the_tag_is_printed_exactly(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "CHANGELOG.md"
            path.write_bytes(b"# Changelog\n\n## v1.1.0\n\n### Added\n\n- New thing\n\n## v1.0.0\n\n### Fixed\n\n- Old bug\n")
            result = notes("v1.1.0", path)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout, "### Added\n\n- New thing\n")
            self.assertNotIn("Old bug", result.stdout)

    def test_missing_or_empty_sections_refuse_publication(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "CHANGELOG.md"
            path.write_bytes(b"# Changelog\n\n## v2.0.0\n\n## v1.0.0\n\n- Something\n")
            self.assertNotEqual(notes("v3.0.0", path).returncode, 0)
            self.assertNotEqual(notes("v2.0.0", path).returncode, 0)

    def test_repository_changelog_covers_every_published_tag(self):
        for tag in ["v0.1.0-preview.1", "v0.1.0-preview.2", "v0.1.0-preview.3",
                    "v0.1.0-preview.4", "v0.1.0-preview.5", "v0.1.0-preview.6"]:
            result = notes(tag)
            self.assertEqual(result.returncode, 0, tag + ": " + result.stderr)
            self.assertTrue(result.stdout.strip())

    def test_publish_job_uses_the_changelog_section(self):
        release = (ROOT / ".github" / "workflows" / "release.yml").read_text()
        publish = release[release.index("  publish:"):]
        self.assertIn("scripts/release_notes.py", publish)
        self.assertIn("--notes-file", publish)
        self.assertNotIn("--notes \"", publish)


if __name__ == "__main__":
    unittest.main()
