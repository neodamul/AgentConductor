import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class BusyButtons(unittest.TestCase):
    def test_pressed_buttons_show_running_work(self):
        subprocess.run(["node", "tests/busy_button_runtime.cjs"], cwd=ROOT, check=True, capture_output=True)
        css = (ROOT / "src/ui/room.css").read_text(encoding="utf-8")
        self.assertIn("button.primary.is-busy:disabled{background:var(--accent)", css)
        startup = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        self.assertIn("AC_PRESS.hold(q('run'),running==='start')", startup)


if __name__ == "__main__":
    unittest.main()
