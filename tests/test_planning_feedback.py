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
    def test_snapshot_waits_for_a_running_action(self):
        # DD-91 amendment: prepare holds the settings lock while copying a folder; the 1 s poll must wait, not fail.
        poll = self.js.split("async function poll(")[1].split("\n")[0]
        self.assertIn("mutating?PLAN_START_LIMIT:undefined", poll)
    def test_progress_reports_coordinator_output(self):
        progress = self.js.split("function planProgress(")[1].split("\n")[0]
        for text in ["job.output", "startup.waiting_for_the_coordinator_s_first_output", "startup.last_coordinator_output_s_ago_kb", "stale"]:
            self.assertIn(text, progress)
    def test_cli_icon_reuses_room_opacity_and_respects_reduced_motion(self):
        self.assertIn("animation:agent-pulse", self.css)
        self.assertIn(".planning-agent-icon{animation:none}", self.css)
        icon = self.css.split(".planning-agent-icon{")[1].split("}")[0]
        self.assertNotIn("background", icon)
        self.assertNotIn("color", icon)
        self.assertNotIn("transform", icon)
    def test_progress_shows_while_the_folder_is_prepared(self):
        # The plan job exists only after prepare replies; the press itself shows progress meanwhile.
        progress = self.js.split("function planProgress(")[1].split("\n")[0]
        self.assertIn("if(planStarting&&!running)", progress)
        self.assertIn("startup.preparing_the_project_folder_for_planning", progress)
        self.assertIn("setInterval(planProgress,1000)", self.js)
        self.assertIn("save(true)", self.js)
if __name__ == "__main__": unittest.main()
