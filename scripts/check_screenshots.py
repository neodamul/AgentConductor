#!/usr/bin/env python3
"""GitHub #290 (R-200 / DD-212): the captured screens are as new as the interface.

The last commit that changed the interface markup or styles must be an ancestor of (or be) the
last commit that changed a captured screen or the confirmation file, so a release after a visible
change needs recaptured screens, or a commit that confirms they still match.
Usage: check_screenshots.py [repository]   (needs full history; the release workflow fetches it)
Exit codes: 0 fresh, 1 stale or missing, 2 cannot be judged.
"""
import subprocess
import sys

INTERFACE = ["src/ui/*.html", "src/ui/*.css"]
CAPTURES = ["docs/assets/ui-*-current.png", "docs/assets/screenshots-current.txt"]


def git(repo, *args, check=True):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True, check=check)


def last(repo, paths):
    return git(repo, "log", "-1", "--format=%H", "--", *paths).stdout.strip()


def main(repo="."):
    if git(repo, "rev-parse", "--is-shallow-repository").stdout.strip() == "true":
        print("The screenshot check needs the full history; fetch it (actions/checkout fetch-depth: 0).", file=sys.stderr)
        return 2
    interface, captured = last(repo, INTERFACE), last(repo, CAPTURES)
    if not interface:
        return 0
    if not captured:
        print("No captured screen exists under docs/assets/ui-*-current.png.", file=sys.stderr)
        return 1
    if git(repo, "merge-base", "--is-ancestor", interface, captured, check=False).returncode == 0:
        return 0
    changed = git(repo, "show", "--name-only", "--format=", interface).stdout.split()
    changed = [name for name in changed if name.startswith("src/ui/") and name.endswith((".html", ".css"))]
    print(
        f"The interface changed in {interface[:8]} ({', '.join(changed)}) after the screens were last captured or confirmed in {captured[:8]}.\n"
        "Recapture docs/assets/ui-*-current.png (see the release steps), or, if the captured screens did not change,\n"
        "commit docs/assets/screenshots-current.txt with the version you checked them against.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:2]))
