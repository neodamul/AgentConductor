import re, subprocess, unittest
from pathlib import Path
class WorkspaceActions(unittest.TestCase):
    def test_create_waits_asynchronously_for_metadata(self):
        js=Path('src/ui/startup.js').read_text(encoding="utf-8")
        f=re.search(r"async function waitForWorkspaceIdle\(\)\{[^\n]+\}",js)
        self.assertIsNotNone(f)
        subprocess.run(['node','-e', f.group(0)+"\nlet count=0;const PLAN_START_LIMIT=300000;async function request(){count++;return {job:null,metadata_job:{state:count<3?'running':'completed'}};}\nwaitForWorkspaceIdle().then(()=>require('node:assert/strict').equal(count,3));"],check=True)
        self.assertIn("requestSubmit(q('new-create'))",js)
        self.assertIn("kind:'delete_workspace'",js)
    def test_settings_registers_generic_mcp_servers(self):
        html=Path('src/ui/startup.html').read_text(encoding="utf-8")+Path('src/ui/mcp.html').read_text(encoding="utf-8")
        ko=Path('src/ui/i18n_ko.js').read_text(encoding="utf-8")
        self.assertIn('MCP servers…',html)
        self.assertIn('Register MCP servers',html)
        self.assertNotIn('MCP 문서 연결',ko)
