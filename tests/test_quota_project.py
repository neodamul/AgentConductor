import re, subprocess, unittest
from pathlib import Path

class QuotaProjectTests(unittest.TestCase):
    def test_saved_project_change_retries_claude_without_waiting_for_the_old_cooldown(self):
        source=Path('src/ui/startup.js').read_text(encoding='utf-8')
        sync=re.search(r'function syncQuotaProject\(current\)\{[^\n]+\}',source).group(0)
        auto=re.search(r'function autoDiscover\(\)\{[^\n]+\}',source).group(0)
        enabled=re.search(r'function quotaEnabled\(profile\)\{[^\n]+\}',source).group(0)
        script="const assert=require('node:assert/strict');let state={config:{repository:'',profiles:[{id:'claude',parser:'claude'},{id:'codex',parser:'codex'}]},quota_refresh_due:{claude:false,codex:false}};const now=Date.now(),quotaRefreshed=new Map([['claude',now],['codex',now]]),addedAgents=new Set();let calls=[];const refreshQuota=id=>calls.push(id),busy=()=>false,dirty=false,planDirty=false,draft={profiles:[]},action=()=>{};"+enabled+sync+auto+"autoDiscover();assert.deepEqual(calls,[]);const next={config:{...state.config,repository:'C:/project'},quota_refresh_due:{claude:true,codex:false}};syncQuotaProject(next);state=next;assert.equal(quotaRefreshed.has('claude'),false);assert.equal(quotaRefreshed.get('codex'),now);autoDiscover();assert.deepEqual(calls,['claude']);quotaRefreshed.set('claude',now);syncQuotaProject(state);assert.equal(quotaRefreshed.get('claude'),now);const changed={config:{...state.config,repository:'C:/other'},quota_refresh_due:{claude:false,codex:false}};syncQuotaProject(changed);state=changed;state.quota_refresh_due.claude=true;autoDiscover();assert.deepEqual(calls,['claude','claude']);state.config.repository='';quotaRefreshed.delete('claude');autoDiscover();assert.equal(calls.length,2);"
        subprocess.run(['node','-e',script],check=True)

if __name__=='__main__': unittest.main()
