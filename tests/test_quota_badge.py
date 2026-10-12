import json
import re
import subprocess
import unittest
from pathlib import Path

class QuotaBadgeTests(unittest.TestCase):
    def test_capacity_badges_share_conservative_display_behavior(self):
        for file in ["room.js", "startup.js"]:
            source = (Path("src/ui") / file).read_text(encoding="utf-8")
            match = re.search(r"function quotaBadge\(quota\) \{[^\n]+\}", source)
            self.assertIsNotNone(match, file)
            script = match.group(0) + "\n" + r"""
const assert=require('node:assert/strict');
assert.equal(quotaBadge({windows:[{remaining_percent:73,label:'five_hour'}]}).text,'73%');
assert.equal(quotaBadge({windows:[{remaining_percent:73},{remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({status:'stale',windows:[{remaining_percent:73}]}).text,'Stale');
assert.equal(quotaBadge({windows:[{remaining_percent:null}]}).text,'—');
assert.equal(quotaBadge(undefined).text,'—');
assert.equal(quotaBadge({plan_type:'enterprise',budgets:[{scope:'individual',remaining:'640',allocated:'1000',remaining_percent:64}],windows:[{remaining_percent:30}]}).text,'64%/100%');
assert.equal(quotaBadge({plan_type:'enterprise',budgets:[{scope:'credits',remaining:'730',allocated:null}]}).text,'—');
assert.equal(quotaBadge({budgets:[{scope:'credits',unlimited:true}]}).text,'∞');
assert.equal(quotaBadge({plan_type:'enterprise',budgets:[{scope:'credits',remaining:null,allocated:null}]}).text,'—');
assert.equal(quotaBadge({status:'stale',plan_type:'enterprise',budgets:[{scope:'individual',remaining:'640',allocated:'1000'}]}).text,'Stale');
assert.equal(quotaBadge({plan_type:'enterprise',budgets:[{scope:'individual',remaining:'640',allocated:'1000'}]}).text,'64%/100%');
assert.equal(quotaBadge({plan_type:'enterprise',budgets:[{scope:'individual',remaining:'0',allocated:'0'}]}).text,'—');
assert.ok(quotaBadge({plan_type:'enterprise',budgets:[{scope:'individual',remaining:'640',allocated:'1000'}]}).title.includes('Individual allocation'));

assert.equal(quotaBadge({plan_type:'enterprise',windows:[{label:'primary',window_minutes:300,remaining_percent:73},{label:'secondary',window_minutes:10080,remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({plan_type:'enterprise',windows:[{label:'five_hour',remaining_percent:73},{label:'seven_day',remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({plan_type:'plus',windows:[{label:'primary',window_minutes:300,remaining_percent:73},{label:'secondary',window_minutes:10080,remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({windows:[{label:'primary',window_minutes:1440,remaining_percent:42},{label:'secondary',window_minutes:10080,remaining_percent:null}]}).text,'42%/—');
assert.equal(quotaBadge({windows:[{label:'seven_day',remaining_percent:56},{label:'five_hour',remaining_percent:93}]}).text,'93%/56%');
assert.equal(quotaBadge({windows:[{label:'primary',window_minutes:120,remaining_percent:80}]}).text,'80%');
assert.ok(quotaBadge({plan_type:'enterprise',windows:[{label:'primary',remaining_percent:73},{label:'secondary',remaining_percent:36}]}).title.includes('primary: 73%'));
assert.ok(quotaBadge({windows:[{label:'primary',window_minutes:1440,remaining_percent:42}]}).title.includes('Daily: 42%'));
assert.ok(quotaBadge({source:'provider',observed_at:1000,windows:[{remaining_percent:73,label:'five_hour'}]}).title.includes('provider'));
"""
            subprocess.run(["node", "-e", script], check=True)
            if file == "room.js":
                render = source.split("function renderQuota(agent) {")[1].split("\nfunction selectTask")[0]
                fixture = """const controls=new Map();function $(id){if(!controls.has(id))controls.set(id,{value:'codex',replaceChildren(...rows){this.rows=rows;}});return controls.get(id);}function node(tag,text){return {text,children:[],append(child){this.children.push(child);}};}function setOptions(){}function agentName(){return 'Codex';}function busy(){return false;}function pretty(x){return x;}const quotaAgent='codex',snapshot={report:{plan:{agents:[{id:'codex',parser:'codex'}]},state:'idle'},quotas:{codex:{status:'available',budgets:[{scope:'individual',remaining:'640',allocated:'1000'}],windows:[]}}};"""
                fixture += "function renderQuota(agent) {" + render + "\nrenderQuota();const a=require('node:assert/strict');a.equal($('quota-windows').rows[0].children[0].text,'640/1000 · Individual allocation');snapshot.quotas.codex.status='stale';renderQuota();a.equal($('quota-windows').rows[0].children[0].text,'Stale · Individual allocation');"
                subprocess.run(["node", "-e", fixture], check=True)
            self.assertIn("quota-badge", source)
            self.assertIn("name-row", source)
            due = re.search(r"function quotaRefreshDue\(last,now,minutes=5\) \{[^\n]+\}", source)
            self.assertIsNotNone(due, file)
            subprocess.run(["node", "-e", due.group(0)+"\nconst a=require('node:assert/strict');a.equal(quotaRefreshDue(undefined,0),true);a.equal(quotaRefreshDue(0,299999),false);a.equal(quotaRefreshDue(0,300000),true);a.equal(quotaRefreshDue(10,0),true);a.equal(quotaRefreshDue(0,60000,1),true);a.equal(quotaRefreshDue(0,300000,10),false);a.equal(quotaRefreshDue(0,600000,10),true);a.equal(quotaRefreshDue(0,60000,0),false);"], check=True)
            self.assertIn("quota-refresh", source)
            self.assertIn("quota-controls", source)
            self.assertIn("aria-label", source)
            if file == "startup.js":
                self.assertIn("!state.job.kind.startsWith('quota:')", source)
                self.assertIn("if(packet.kind!=='quota')notice", source)
            else:
                self.assertIn("if(action.kind!=='quota')notice", source)
                self.assertIn("job.kind!=='quota'", source)

if __name__ == '__main__':
    unittest.main()
