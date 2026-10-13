from ui_messages import js_prelude, translated_message
import re
import subprocess
import unittest
from pathlib import Path

class QuotaSchedulerTests(unittest.TestCase):
    def test_inflight_metadata_does_not_skip_the_other_agent_or_block_editing(self):
        source = Path("src/ui/startup.js").read_text(encoding="utf-8")
        due = re.search(r"function quotaRefreshDue\(last,now,minutes=5\) \{[^\n]+\}", source).group(0)
        auto = re.search(r"function autoDiscover\(\)\{[^\n]+\}", source).group(0)
        busy = re.search(r"const busy=\(\)=>[^;]+;", source).group(0)
        script = r"""
const assert=require('node:assert/strict');let mutating=false,maintenance=false;
const state={metadata_job:{state:'running',kind:'quota:codex'},quota_job:{state:'running',kind:'quota:codex'},job:null,config:{profiles:[{id:'codex',parser:'codex'},{id:'claude',parser:'claude'}]}};
const quotaRefreshed=new Map([['codex',Date.now()]]),addedAgents=new Set();let calls=[];
const action=p=>calls.push(p),refreshQuota=agent=>action({kind:'quota',agent}),dirty=false,planDirty=false,draft={profiles:[]};
"""+busy+due+auto+r"""
assert.equal(busy(),false);autoDiscover();assert.equal(calls.length,0);assert.equal(quotaRefreshed.has('claude'),false);
state.quota_job.state='completed';state.metadata_job.state='completed';autoDiscover();assert.equal(calls.length,1);assert.equal(calls[0].agent,'claude');
maintenance=true;assert.equal(busy(),true);const count=calls.length;autoDiscover();assert.equal(calls.length,count);maintenance=false;
state.job={state:'running',kind:'plan'};assert.equal(busy(),true);
"""
        subprocess.run(["node","-e",js_prelude()+script],check=True)

    def test_rejected_refresh_does_not_advance_the_cooldown(self):
        source = Path("src/ui/startup.js").read_text(encoding="utf-8")
        refresh = re.search(r"async function refreshQuota\(agent\)\{[^\n]+\}", source).group(0)
        script = "const assert=require('node:assert/strict'),quotaRefreshed=new Map();let result=null;const action=async()=>result;"+refresh+"(async()=>{await refreshQuota('claude');assert.equal(quotaRefreshed.has('claude'),false);result={started:true};await refreshQuota('claude');assert.equal(quotaRefreshed.has('claude'),true);})();"
        subprocess.run(["node","-e",js_prelude()+script],check=True)

if __name__ == "__main__":
    unittest.main()
