#!/usr/bin/env python3
"""DD-49: print the CHANGELOG.md section for a version tag; fail when it is missing or empty."""
import sys
from pathlib import Path


def section(text, tag):
    lines = text.splitlines()
    heading = "## " + tag
    try:
        start = next(i for i, line in enumerate(lines) if line.strip() == heading)
    except StopIteration:
        raise SystemExit("CHANGELOG.md has no section for " + tag)
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    body = "\n".join(lines[start + 1:end]).strip()
    if not body:
        raise SystemExit("CHANGELOG.md section for " + tag + " is empty")
    return body + "\n"


def main():
    if len(sys.argv) not in (2, 3):
        raise SystemExit("Usage: release_notes.py <tag> [CHANGELOG.md]")
    path = Path(sys.argv[2]) if len(sys.argv) == 3 else Path(__file__).resolve().parents[1] / "CHANGELOG.md"
    sys.stdout.write(section(path.read_text(encoding="utf-8"), sys.argv[1]))


if __name__ == "__main__":
    main()
