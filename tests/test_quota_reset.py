import re
import subprocess
import unittest
from pathlib import Path


class QuotaResetTests(unittest.TestCase):
    """R-69 / DD-75 / QAS-62: exhausted quota shows when it returns."""

    def test_exhausted_window_shows_reset_time(self):
        for file in ["room.js", "startup.js"]:
            source = (Path("src/ui") / file).read_text(encoding="utf-8")
            match = re.search(r"function quotaBadge\(quota\) \{[^\n]+\}", source)
            self.assertIsNotNone(match, file)
            script = match.group(0) + "\n" + r"""
const assert=require('node:assert/strict');
const today=new Date();today.setHours(23,59,0,0);const t=Math.floor(today/1000);
const later=new Date(today);later.setDate(later.getDate()+2);later.setHours(9,5,0,0);const l=Math.floor(later/1000);
assert.equal(quotaBadge({windows:[{label:'five_hour',remaining_percent:0,resets_at:t},{label:'seven_day',remaining_percent:40,resets_at:l}]}).text,'0%/40% · 23:59');
assert.equal(quotaBadge({windows:[{label:'five_hour',remaining_percent:0,resets_at:l}]}).text,`0% · ${later.getMonth()+1}/${later.getDate()} 09:05`);
assert.equal(quotaBadge({windows:[{label:'five_hour',remaining_percent:0},{label:'seven_day',remaining_percent:0,resets_at:t}]}).text,'0%/0% · 23:59');
assert.equal(quotaBadge({windows:[{label:'five_hour',remaining_percent:0}]}).text,'0% · reset unknown');
assert.equal(quotaBadge({windows:[{label:'five_hour',remaining_percent:12,resets_at:t}]}).text,'12%');
assert.ok(quotaBadge({windows:[{label:'five_hour',remaining_percent:12,resets_at:t}]}).title.includes('resets 23:59'));
// Two exhausted windows: usable again only after the later reset.
assert.equal(quotaBadge({windows:[{label:'five_hour',remaining_percent:0,resets_at:t},{label:'seven_day',remaining_percent:0,resets_at:l}]}).text,`0%/0% · ${later.getMonth()+1}/${later.getDate()} 09:05`);
assert.equal(quotaBadge({status:'stale',windows:[{remaining_percent:0,resets_at:t}]}).text,'Stale');
"""
            subprocess.run(["node", "-e", script], check=True)


if __name__ == "__main__":
    unittest.main()
