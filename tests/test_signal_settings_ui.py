from ui_messages import js_prelude, translated_message
import unittest
import re
import subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
# R-92 / DD-100 / QAS-88: Settings selects the signal transport and shows the last effective route.
class SignalSettings(unittest.TestCase):
    def test_settings_dialog_selects_transport_and_port(self):
        html = (ROOT / "src/ui/startup.html").read_text(encoding="utf-8")
        settings = html.split('<dialog id="start-settings"')[1].split("</dialog>")[0]
        for control in ['id="start-signal-transport"', 'value="local"', 'value="agentmemory"', 'id="start-signal-port"', 'id="start-signal-last"']:
            self.assertIn(control, settings)
    def test_settings_load_asynchronously_and_save_the_preference(self):
        js = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        self.assertIn("request({kind:'signal_transport'})", js)
        self.assertIn("request({kind:'set_signal_transport',preference:", js)
        self.assertIn("startup.last_signal", js)
    def test_port_visibility_follows_transport_without_losing_value(self):
        js = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        match = re.search(r"function syncSignalSettings\(\)\{[^\n]+\}", js)
        self.assertIsNotNone(match)
        fixture = "const controls={'signal-transport':{value:'local'},'signal-server':{hidden:false},'signal-port':{value:'3111'}};function q(id){return controls[id];}"
        checks = "syncSignalSettings();const a=require('node:assert/strict');a.equal(q('signal-server').hidden,true);q('signal-transport').value='agentmemory';syncSignalSettings();a.equal(q('signal-server').hidden,false);q('signal-transport').value='local';syncSignalSettings();a.equal(q('signal-server').hidden,true);a.equal(q('signal-port').value,'3111');"
        subprocess.run(['node','-e',fixture+match.group(0)+checks],check=True)
        self.assertIn("q('signal-transport').onchange=syncSignalSettings", js)
    def test_workspace_actions_live_in_the_menu_not_in_settings(self):
        html = (ROOT / "src/ui/startup.html").read_text(encoding="utf-8")
        settings = html.split('<dialog id="start-settings"')[1].split("</dialog>")[0]
        for button in ('start-mcp-settings', 'start-cache-button'):
            self.assertIn(button, settings)
        for button in ('start-rules-open', 'start-messenger-open', 'start-export-settings', 'start-import-settings', 'start-tools-guide', 'start-manual-open'):
            self.assertNotIn(button, settings)
        js = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        # GitHub #751: the Workspace name opens no chooser; the application menu carries the recents and the actions.
        self.assertIn("window.acAppMenu.items({el,rows,create,open", js)
        self.assertIn("q('workspace-menu').onclick=()=>menuActions.workspaceSettings()", js)
        self.assertIn("tools:()=>q('tools').showModal()", js)
    def test_signal_settings_are_translated(self):
        table = (ROOT / "src/ui/messages/ko.json").read_text(encoding="utf-8")
        for key in ["Signal transport", "Local Rust", "agentmemory preferred", "agentmemory port", "No signal sent yet.", "Last signal: "]:
            self.assertTrue(translated_message(key), key)
if __name__ == "__main__": unittest.main()
