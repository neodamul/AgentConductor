from ui_messages import js_prelude, translated_message
import re
import subprocess
import unittest
from pathlib import Path

class GitBaselineConsentTests(unittest.TestCase):
    def test_stop_has_no_write_and_approval_rechecks_the_baseline(self):
        source=Path("src/ui/startup.js").read_text(encoding="utf-8")
        function=re.search(r"async function ensureBaseline\(limit=PLAN_START_LIMIT\)\{[^\n]+\}",source).group(0)
        script=r"""
const PLAN_START_LIMIT=100;
const assert=require('node:assert/strict');let choice='cancel',calls=[],statuses=[],controls=new Map();
function q(id){if(!controls.has(id))controls.set(id,{showModal(){queueMicrotask(()=>{this.returnValue=choice;this.onclose();});}});return controls.get(id);}
const notice=()=>{},request=async()=>statuses.shift(),action=async packet=>{calls.push(packet);return {created:true};};
"""+function+r"""
(async()=>{
statuses=[{source_mode:'folder',needs_git:true,repository:'/fixture/project'}];assert.equal(await ensureBaseline(),true);assert.equal(calls.length,0);assert.equal(controls.size,0);
statuses=[{source_mode:'folder',needs_initial_commit:true}];assert.equal(await ensureBaseline(),true);assert.equal(calls.length,0);
statuses=[{clean:false,changes:['modified.txt'],os_files:[],total:1}];assert.equal(await ensureBaseline(),false);assert.equal(calls.length,0);
choice='confirm';statuses=[{clean:false,changes:['modified.txt'],os_files:[],total:1}];assert.equal(await ensureBaseline(),true);assert.equal(calls.length,1);assert.equal(calls[0].kind,'commit_baseline');
})();
"""
        subprocess.run(["node","-e",js_prelude()+script],check=True)

if __name__=="__main__":unittest.main()
