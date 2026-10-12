import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
# R-83 / DD-91 / QAS-79: Generate plan never reports a false timeout and shows real activity.
class PlanningFeedback(unittest.TestCase):
    def setUp(self):
        self.js = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        self.css = (ROOT / "src/ui/startup.css").read_text(encoding="utf-8")
    def test_planning_requests_allow_a_bounded_folder_copy(self):
        self.assertIn("async function action(packet,limit)", self.js)
        self.assertIn("request({...packet,revision:state.revision},limit)", self.js)
        self.assertIn("action({kind:'prepare'},PLAN_START_LIMIT)", self.js)
    def test_progress_reports_coordinator_output(self):
        progress = self.js.split("function planProgress(")[1].split("\n")[0]
        for text in ["job.output", "Waiting for the coordinator's first output.", "Last coordinator output ", "stale"]:
            self.assertIn(text, progress)
    def test_reduced_motion_still_blinks_without_movement(self):
        reduced = self.css.split("@media(prefers-reduced-motion:reduce){.plan-progress")[1].split("}}")[0]
        self.assertIn("plan-blink", reduced)
        self.assertIn("@keyframes plan-blink{", self.css)
        self.assertNotIn("transform", self.css.split("@keyframes plan-blink{")[1].split("}}")[0])
if __name__ == "__main__": unittest.main()
