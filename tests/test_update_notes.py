"""R-133 / DD-142: read the actual release body without executing remote markup."""
import re
import subprocess
import unittest
from pathlib import Path
from ui_messages import js_prelude

class UpdateNotes(unittest.TestCase):
    def test_remote_body_is_literal_scrollable_text_and_clears_when_absent(self):
        js = Path('src/ui/startup.js').read_text(encoding='utf-8')
        render = js.split('function renderUpdate(){')[1].split('  async function updateRequest')[0]
        fixture = r"""
const controls=new Map();function q(id){if(!controls.has(id))controls.set(id,{textContent:'',hidden:false,set innerHTML(v){throw Error('remote HTML must not execute');}});return controls.get(id);}
let update={phase:'idle',current:'0.1.18',latest:'0.1.19',newer:true,installable:true,page:'https://github.com/example/repo/releases/tag/v0.1.19',notes:'- First change\n<script>danger()</script>\n- Final change'};
"""
        checks = r"""
renderUpdate();const a=require('node:assert/strict');a.equal(q('update-body').hidden,false);a.equal(q('update-body').textContent,update.notes);
update={...update,notes:''};renderUpdate();a.equal(q('update-body').hidden,true);a.equal(q('update-body').textContent,'');
"""
        subprocess.run(['node','-e',js_prelude()+fixture+'function renderUpdate(){'+render+checks],check=True)
        html=Path('src/ui/startup.html').read_text(encoding='utf-8')
        self.assertIn('id="start-update-body"',html)
        self.assertLess(html.index('id="start-update-notes"'),html.index('id="start-update-body"'))
        css=Path('src/ui/startup.css').read_text(encoding='utf-8')
        rule=re.search(r'#start-update-body\{([^}]+)\}',css).group(1)
        self.assertIn('overflow:auto',rule)
        self.assertIn('max-height:',rule)
        self.assertIn('white-space:pre-wrap',rule)

if __name__=='__main__': unittest.main()
