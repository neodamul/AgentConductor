"""The project is published under the MIT license (user decision 2026-10-11)."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class ProjectLicenseTests(unittest.TestCase):
    def test_mit_license_file_and_manifest_field(self):
        text = (ROOT / "LICENSE").read_text(encoding="utf-8")
        self.assertTrue(text.startswith("MIT License"))
        self.assertIn("Permission is hereby granted, free of charge", text)
        self.assertIn('license = "MIT"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn("](LICENSE)", (ROOT / "README.md").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
