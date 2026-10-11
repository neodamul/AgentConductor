import json
import re
import subprocess
import unittest
from pathlib import Path

class QuotaBadgeTests(unittest.TestCase):
    def test_capacity_badges_share_conservative_display_behavior(self):
        for file in ["room.js", "startup.js"]:
            source = (Path("src/ui") / file).read_text()
            match = re.search(r"function quotaBadge\(quota\) \{[^\n]+\}", source)
            self.assertIsNotNone(match, file)
            script = match.group(0) + "\n" + r"""
const assert=require('node:assert/strict');
assert.equal(quotaBadge({windows:[{remaining_percent:73,label:'five_hour'}]}).text,'73%');
assert.equal(quotaBadge({windows:[{remaining_percent:73},{remaining_percent:36}]}).text,'36%');
assert.equal(quotaBadge({status:'stale',windows:[{remaining_percent:73}]}).text,'Stale');
assert.equal(quotaBadge({windows:[{remaining_percent:null}]}).text,'—');
assert.equal(quotaBadge(undefined).text,'—');
assert.ok(quotaBadge({source:'provider',observed_at:1000,windows:[{remaining_percent:73,label:'five_hour'}]}).title.includes('provider'));
"""
            subprocess.run(["node", "-e", script], check=True)
            self.assertIn("quota-badge", source)
            self.assertIn("name-row", source)
            due = re.search(r"function quotaRefreshDue\(last,now\) \{[^\n]+\}", source)
            self.assertIsNotNone(due, file)
            subprocess.run(["node", "-e", due.group(0)+"\nconst a=require('node:assert/strict');a.equal(quotaRefreshDue(undefined,0),true);a.equal(quotaRefreshDue(0,119999),false);a.equal(quotaRefreshDue(0,120000),true);a.equal(quotaRefreshDue(10,0),true);"], check=True)
            self.assertIn("quota-refresh", source)
            self.assertIn("aria-label", source)

if __name__ == '__main__':
    unittest.main()
