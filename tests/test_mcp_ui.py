from ui_messages import translated_message
import unittest
import subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
class McpUi(unittest.TestCase):
    def test_operator_can_configure_discover_preview_and_apply(self):
        html = (ROOT / "src/ui/mcp.html").read_text(encoding="utf-8")
        js = (ROOT / "src/ui/mcp.js").read_text(encoding="utf-8")
        for control in ["mcp-servers", "mcp-add", "mcp-kind", "mcp-name", "mcp-url", "mcp-executable", "mcp-arguments", "mcp-secrets", "mcp-advanced", "mcp-edit-save", "mcp-preview", "mcp-apply", "source-reference"]:
            self.assertIn('id="start-' + control + '"', html)
        for action in ["mcp_save", "mcp_discover", "mcp_preview", "mcp_apply"]:
            self.assertIn(action, js)
        self.assertIn('id="start-mcp-list-view"', html)
        self.assertIn('id="start-mcp-edit-view"', html)
        for obsolete in ["mcp-jira", "mcp-confluence", "mcp-bindings-save", "mcp-tools"]:
            self.assertNotIn('id="start-' + obsolete + '"', html)
        self.assertNotIn("function bindingFor", js)
        self.assertIn("await discoverServer(name)", js)
        self.assertLess(html.index('id="start-mcp-advanced"'), html.index('id="start-mcp-json"'))
        self.assertNotIn("innerHTML", js)
        self.assertNotIn("setInterval", js)
        self.assertIn("AC_I18N.bind", js)
    def test_connection_settings_live_in_settings_menu(self):
        main = (ROOT / "src/ui/startup.html").read_text(encoding="utf-8")
        settings = main.split('<dialog id="start-settings"')[1].split('</dialog>')[0]
        self.assertIn('id="start-mcp-settings"', settings)
        source = (ROOT / "src/ui/mcp.html").read_text(encoding="utf-8").split('</div>')[0]
        self.assertNotIn('start-mcp-settings', source)
    def test_mcp_controls_are_translated_without_translating_evidence(self):
        table = (ROOT / "src/ui/messages/ko.json").read_text(encoding="utf-8")
        for key in ["MCP servers…", "Preview imports", "Apply selected sources", "Saved server", "Discover tools"]:
            self.assertTrue(translated_message(key), key)
        js = (ROOT / "src/ui/mcp.js").read_text(encoding="utf-8")
        # #903: source text and summaries are shown as plain text, never through the translation table.
        self.assertIn("mk('pre','source-body',record.body)", js)
        self.assertIn("mk('div','source-summary',summary.text)", js)
    def test_coordinator_selects_tools_and_server_operations_stay_connected(self):
        subprocess.run(["node", "tests/mcp_bindings_runtime.cjs"], cwd=ROOT, check=True, capture_output=True)
if __name__ == "__main__": unittest.main()
