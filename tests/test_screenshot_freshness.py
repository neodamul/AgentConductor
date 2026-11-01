"""GitHub #290 (QAS-198): the captured screens are as new as the interface, or a later commit confirms them."""
import pathlib
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_screenshots.py"
GIT = ["git", "-c", "user.name=t", "-c", "user.email=t@example.invalid", "-c", "commit.gpgsign=false"]


def git(repo, *args):
    subprocess.run([*GIT, "-C", str(repo), *args], check=True, capture_output=True)


def commit(repo, path, text):
    file = pathlib.Path(repo) / path
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_text(text)
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", f"change {path}")


def check(repo):
    return subprocess.run(["python3", str(SCRIPT), str(repo)], capture_output=True, text=True)


class Screenshots(unittest.TestCase):
    def repo(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        git(directory.name, "init", "-q", "-b", "main")
        return directory.name

    def test_screens_captured_after_the_last_interface_change_pass(self):
        repo = self.repo()
        commit(repo, "src/ui/startup.css", "a{}")
        commit(repo, "docs/assets/ui-workspace-current.png", "png")
        self.assertEqual(check(repo).returncode, 0, check(repo).stderr)

    def test_an_interface_change_after_the_screens_fails_and_names_the_way_out(self):
        repo = self.repo()
        commit(repo, "docs/assets/ui-workspace-current.png", "png")
        commit(repo, "src/ui/room.html", "<p>")
        result = check(repo)
        self.assertEqual(result.returncode, 1)
        self.assertIn("src/ui/room.html", result.stderr)
        self.assertIn("docs/assets/screenshots-current.txt", result.stderr)

    def test_confirming_the_screens_in_a_later_commit_passes(self):
        repo = self.repo()
        commit(repo, "docs/assets/ui-workspace-current.png", "png")
        commit(repo, "src/ui/room.css", "a{}")
        commit(repo, "docs/assets/screenshots-current.txt", "The captured screens match the interface of 0.1.52.\n")
        self.assertEqual(check(repo).returncode, 0, check(repo).stderr)

    def test_script_and_message_changes_are_not_interface_changes(self):
        repo = self.repo()
        commit(repo, "docs/assets/ui-workspace-current.png", "png")
        commit(repo, "src/ui/room.js", "x")
        commit(repo, "src/ui/messages/en.json", "{}")
        self.assertEqual(check(repo).returncode, 0, check(repo).stderr)

    def test_a_shallow_checkout_cannot_be_judged(self):
        repo = self.repo()
        commit(repo, "docs/assets/ui-workspace-current.png", "png")
        commit(repo, "src/ui/room.css", "a{}")
        clone = tempfile.TemporaryDirectory()
        self.addCleanup(clone.cleanup)
        subprocess.run(["git", "clone", "-q", "--depth", "1", f"file://{repo}", clone.name], check=True, capture_output=True)
        result = check(clone.name)
        self.assertEqual(result.returncode, 2)
        self.assertIn("full history", result.stderr)

    def test_no_captured_screen_at_all_fails(self):
        repo = self.repo()
        commit(repo, "src/ui/room.css", "a{}")
        self.assertEqual(check(repo).returncode, 1)


if __name__ == "__main__":
    unittest.main()
