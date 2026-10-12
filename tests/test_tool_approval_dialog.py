import unittest
from pathlib import Path

UI = Path("src/ui")


class ToolApprovalDialogTests(unittest.TestCase):
    def test_live_request_opens_a_decision_window_with_the_activity_log(self):
        html = (UI / "room.html").read_text(encoding="utf-8")
        for control in ["tool-dialog", "tool-dialog-status", "tool-dialog-log", "tool-dialog-allow", "tool-dialog-deny", "tool-dialog-later"]:
            self.assertIn('id="' + control + '"', html)
        js = (UI / "room.js").read_text(encoding="utf-8")
        self.assertIn("syncToolDialog();", js.split("function renderTools()")[1].split("\nfunction ")[0])
        self.assertIn("dialog.showModal()", js)
        # The log mirrors the task timeline on every poll, whatever was decided.
        self.assertIn("mirrorToolLog(t.id,timeline)", js)
        self.assertIn("No longer waiting: ", js)
        self.assertIn("kind:'tool_approve'", js.replace("decideTool('tool_approve')", "kind:'tool_approve'"))
        # Deciding later is remembered per request and the tools link reopens it.
        self.assertIn("deferredTools.add(toolDialog.id)", js)
        self.assertIn("deferredTools.clear()", js)

    def test_feedback_to_an_idle_run_resumes_it_after_the_revision(self):
        js = (UI / "room.js").read_text(encoding="utf-8")
        send = js.split("$('send-feedback').onclick=")[1].split("\n")[0]
        self.assertIn("resumeAfterRevise=true", send)
        self.assertIn("job.kind==='revise'&&resumeAfterRevise", js)
        self.assertIn("act({kind:'resume'})", js)

    def test_dialog_text_is_translated(self):
        table = (UI / "i18n_ko.js").read_text(encoding="utf-8")
        for key in ["TOOL APPROVAL", "Decide later", "Allow once", "Waiting for your decision. Time left: ", "No longer waiting: "]:
            self.assertIn('"' + key + '":', table)


if __name__ == "__main__":
    unittest.main()
