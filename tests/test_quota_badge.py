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
assert.equal(quotaBadge({plan_type:'enterprise',windows:[{label:'primary',window_minutes:300,remaining_percent:73},{label:'secondary',window_minutes:10080,remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({plan_type:'enterprise',windows:[{label:'five_hour',remaining_percent:73},{label:'seven_day',remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({plan_type:'plus',windows:[{label:'primary',window_minutes:300,remaining_percent:73},{label:'secondary',window_minutes:10080,remaining_percent:36}]}).text,'73%/36%');
assert.equal(quotaBadge({windows:[{label:'primary',window_minutes:1440,remaining_percent:42},{label:'secondary',window_minutes:10080,remaining_percent:null}]}).text,'42%/—');
assert.equal(quotaBadge({windows:[{label:'seven_day',remaining_percent:56},{label:'five_hour',remaining_percent:93}]}).text,'93%/56%');
assert.equal(quotaBadge({windows:[{label:'primary',window_minutes:120,remaining_percent:80}]}).text,'80%');
assert.ok(quotaBadge({plan_type:'enterprise',windows:[{label:'primary',remaining_percent:73},{label:'secondary',remaining_percent:36}]}).title.includes('Current: 73%'));
assert.ok(quotaBadge({windows:[{label:'primary',window_minutes:1440,remaining_percent:42}]}).title.includes('Daily: 42%'));
assert.ok(quotaBadge({source:'provider',observed_at:1000,windows:[{remaining_percent:73,label:'five_hour'}]}).title.includes('provider'));
"""
            subprocess.run(["node", "-e", script], check=True)
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
