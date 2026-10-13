from ui_messages import js_prelude, translated_message
import re, subprocess, unittest
from pathlib import Path
class ProfileSaveFeedback(unittest.TestCase):
    def test_valid_save_closes_and_failures_remain_visible(self):
        source=Path('src/ui/startup.js').read_text(encoding='utf-8')
        match=re.search(r"async function applyAgentProfile\(\)\{[^\n]+\}",source)
        self.assertIsNotNone(match)
        fixture="""const assert=require('node:assert/strict');const fields=new Map();function q(id){if(!fields.has(id))fields.set(id,{value:'',dataset:{},disabled:false,open:true,close(){this.closed=true;this.open=false;}});return fields.get(id);}const copy=x=>JSON.parse(JSON.stringify(x));let mutating=false,agentIndex=0,providerChanged=false,error='',calls=0,fail=false,captured;
const draft={profiles:[{id:'claude',parser:'claude',executable:'claude'},{id:'codex',parser:'codex',executable:'codex'}]};function setTimeout(){return 0;}function clearTimeout(){}async function saveProfiles(list){calls++;captured=list;if(fail)throw Error('cyclic fallback policy');return true;}
for(const [id,value]of Object.entries({'args':'[]','quota-minutes':'5','agent-id':'claude','provider':'claude','executable':'claude','input':'stdin','sessions':'3','permission':'unverified','fallback-agent':'codex','fallback-model':''}))q(id).value=value;
"""
        checks="""(async()=>{await applyAgentProfile();assert.equal(captured[0].fallback.agent,'codex');assert.equal(q('agent-dialog').closed,true);assert.equal(q('agent-apply').disabled,false);q('agent-dialog').closed=false;q('agent-dialog').open=true;fail=true;await applyAgentProfile();assert.equal(q('agent-error').textContent,'cyclic fallback policy');assert.equal(q('agent-dialog').closed,false);mutating=true;error='';const before=calls;await applyAgentProfile();assert.equal(calls,before);assert.ok(q('agent-error').textContent.includes('finishing'));mutating=false;fail=false;q('args').value='invalid';await applyAgentProfile();assert.equal(q('agent-dialog').closed,false);assert.equal(q('agent-apply').disabled,false);})().catch(e=>{console.error(e);process.exit(1);});"""
        subprocess.run(['node','-e',js_prelude()+fixture+re.search(r"function notice\(text,error=false\)\{[^\n]+\}",source).group(0)+'\n'+match.group(0)+'\n'+checks],check=True)
        html=Path('src/ui/startup.html').read_text(encoding='utf-8')
        self.assertIn('id="start-agent-error"',html)
