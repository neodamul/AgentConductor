import subprocess
import unittest


class IdlePollTimeoutTests(unittest.TestCase):
    def test_idle_poll_and_suspended_timeout(self):
        subprocess.run(
            ["node", "tests/idle_poll_timeout_runtime.cjs"],
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        )


if __name__ == "__main__":
    unittest.main()
