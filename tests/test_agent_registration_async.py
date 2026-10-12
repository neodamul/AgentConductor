import re
import subprocess
import unittest
from pathlib import Path

class RegistrationAsyncTests(unittest.TestCase):
    def test_dialog_opens_before_lookup_and_late_results_preserve_user_input(self):
        source=Path("src/ui/startup.js").read_text(encoding="utf-8")
        form=re.search(r"async function profileForm\(index\)\{[^\n]+\}",source).group(0)
        resolve=re.search(r"function resolveProfileExecutable\(id,executable\)\{[^\n]+\}",source).group(0)
        script=r"""
const assert=require('node:assert/strict'),controls=new Map();
function q(id){if(!controls.has(id))controls.set(id,{value:'',dataset:{},replaceChildren(){},showModal(){this.open=true;}});return controls.get(id);}
function el(){return {};}function fallbackModels(){}let agentIndex,providerChanged,release;
const draft={profiles:[{id:'claude',parser:'claude',executable:'claude',args:[]}]};
const request=()=>new Promise(resolve=>{release=resolve;});
"""+form+resolve+r"""
(async()=>{
const form=profileForm(0);assert.equal(q('agent-dialog').open,true);await form;
q('executable').value='operator-path';release({path:'discovered-path'});await new Promise(setImmediate);
assert.equal(q('executable').value,'operator-path');
await profileForm(0);release({path:'discovered-path'});await new Promise(setImmediate);
assert.equal(q('executable').value,'discovered-path');
})();
"""
        subprocess.run(["node","-e",script],check=True)

    def test_only_the_requested_agent_control_is_busy(self):
        source=Path("src/ui/startup.js").read_text(encoding="utf-8")
        helper=re.search(r"const metadataBusy=[^\n]+;",source).group(0)
        script="const a=require('node:assert/strict'),state={metadata_job:{state:'running',kind:'catalog:first'},metadata_queue:[{kind:'quota',agent:'second'}]};"+helper+"a.equal(metadataBusy('catalog','first'),true);a.equal(metadataBusy('catalog','second'),false);a.equal(metadataBusy('quota','first'),false);a.equal(metadataBusy('quota','second'),true);"
        subprocess.run(["node","-e",script],check=True)

if __name__=="__main__":unittest.main()
