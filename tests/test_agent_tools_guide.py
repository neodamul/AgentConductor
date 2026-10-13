from ui_messages import js_prelude, translated_message
import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
# R-81 / DD-89 / QAS-77: Settings shows a static install guide for companion agent tools.
COMMANDS = [
    "npx -y @agentmemory/agentmemory@latest",
    "/plugin marketplace add rohitg00/agentmemory",
    "codex plugin add agentmemory@agentmemory",
    "/plugin install ponytail@ponytail",
    "codex plugin add ponytail@ponytail",
    "brew install rtk",
    "winget install rtk-ai.rtk",
    "rtk init -g --codex",
    "uv tool install graphifyy",
    "graphify install --platform codex",
    "uv tool install --upgrade 'cocoindex-code[full]'",
    "codex mcp add cocoindex-code -- ccc mcp",
]
class AgentToolsGuide(unittest.TestCase):
    def setUp(self):
        html = (ROOT / "src/ui/startup.html").read_text(encoding="utf-8")
        self.settings = html.split('<dialog id="start-settings"')[1].split("</dialog>")[0]
        self.guide = html.split('<dialog id="start-tools"')[1].split("</dialog>")[0]
    def test_settings_opens_the_guide(self):
        self.assertIn('id="start-tools-guide"', self.settings)
        js = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        self.assertIn("q('tools-guide').onclick", js)
    def test_guide_lists_exact_commands_as_untranslated_code(self):
        for command in COMMANDS:
            self.assertIn("<code>" + command + "</code>", self.guide)
    def test_guide_is_static_and_runs_nothing(self):
        for forbidden in ["<script", "<a ", "href=", "onclick", "request("]:
            self.assertNotIn(forbidden, self.guide)
    def test_guide_text_is_translated_and_manual_matches(self):
        table = (ROOT / "src/ui/messages/ko.json").read_text(encoding="utf-8")
        for key in ["Agent tools guide…", "Agent tools", "Close"]:
            self.assertTrue(translated_message(key), key)
        manual = (ROOT / "docs/user-manual.md").read_text(encoding="utf-8")
        for command in COMMANDS:
            self.assertIn(command, manual)
if __name__ == "__main__": unittest.main()
