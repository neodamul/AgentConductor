from ui_messages import js_prelude, translated_message
import unittest
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
    def test_signal_settings_are_translated(self):
        table = (ROOT / "src/ui/messages/ko.json").read_text(encoding="utf-8")
        for key in ["Signal transport", "Local Rust", "agentmemory preferred", "agentmemory port", "No signal sent yet.", "Last signal: "]:
            self.assertTrue(translated_message(key), key)
if __name__ == "__main__": unittest.main()
