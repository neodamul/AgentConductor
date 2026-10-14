from ui_messages import js_prelude, translated_message
import unittest
import subprocess
import shutil
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
class McpUi(unittest.TestCase):
    def test_operator_can_configure_discover_preview_and_apply(self):
        html = (ROOT / "src/ui/mcp.html").read_text(encoding="utf-8")
        js = (ROOT / "src/ui/mcp.js").read_text(encoding="utf-8")
        for control in ["mcp-json", "mcp-server", "mcp-discover", "mcp-preview", "mcp-apply", "source-reference"]:
            self.assertIn('id="start-' + control + '"', html)
        for action in ["mcp_save", "mcp_discover", "mcp_preview", "mcp_apply"]:
            self.assertIn(action, js)
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
        self.assertIn("source=el('pre'", js)
    def test_coordinator_selects_tools_and_server_operations_stay_connected(self):
        subprocess.run(["node", "tests/mcp_bindings_runtime.cjs"], cwd=ROOT, check=True, capture_output=True)
    @unittest.skipUnless(shutil.which("node"), "Node verifies the pure source suggestion helper")
    def test_shorthand_is_suggested_and_unrelated_numbers_are_ignored(self):
        script = (ROOT / "src/ui/mcp.js").read_text(encoding="utf-8").split("// DD-68:")[0]
        check = script + "\nconst a=sourceSuggestions('Implement EMM-52599. 참고 (53553565, 548443335)');if(a.length!==3)throw Error(JSON.stringify(a));if(sourceSuggestions('2026 build 500').length)throw Error('unrelated number');if(sourceSuggestions('jira:EMM-52599 confluence:53553565').length)throw Error('explicit duplicate');"
        subprocess.run(["node", "-e", js_prelude()+check], check=True)
if __name__ == "__main__": unittest.main()
