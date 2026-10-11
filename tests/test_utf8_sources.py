"""R-70 / DD-76 / QAS-63: UTF-8 is explicit everywhere, so Windows (cp1252) behaves like macOS."""
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class Utf8SourceTests(unittest.TestCase):
    def test_python_text_io_names_utf8(self):
        offenders = []
        for path in sorted([*ROOT.glob("tests/*.py"), *ROOT.glob("scripts/*.py")]):
            for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
                text_io = re.search(r"\.(read_text|write_text)\(|(?<![\w.])open\(|\.open\(", line)
                binary = re.search(r"['\"][rwa]b['\"]|zipfile|ZipFile|tarfile", line)
                if text_io and not binary and "encoding" not in line and "re.search" not in line:
                    offenders.append(f"{path.name}:{number}: {line.strip()}")
        self.assertEqual(offenders, [])

    def test_interface_sources_are_utf8_and_declared(self):
        for path in (ROOT / "src" / "ui").glob("*.*"):
            if path.suffix in {".html", ".js", ".css"}:
                path.read_bytes().decode("utf-8")
        self.assertIn('<meta charset="utf-8">', (ROOT / "src/ui/room.html").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
