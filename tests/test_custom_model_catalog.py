"""Offline checks for operator model IDs when a CLI has no discovery command."""
import json, pathlib, subprocess, unittest
ROOT = pathlib.Path(__file__).resolve().parents[1]
class CustomModels(unittest.TestCase):
    def test_model_ids_are_registered_without_a_cli_or_http_request(self):
        script = (ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        helper = next((line.strip() for line in script.splitlines() if line.strip().startswith("function manualModelCatalog(")), None)
        self.assertIsNotNone(helper, "missing manual model catalog parser")
        cases = """
const good=manualModelCatalog('company-small\\ncompany-large\\ncompany-small',42);
if(JSON.stringify(good.models)!=='["company-small","company-large"]'||good.queried_at!==42||!good.source.includes('Operator-configured'))throw Error('manual catalog provenance');
for(const bad of ['', 'bad model', '--unsafe', Array.from({length:1025},(_,i)=>'m'+i).join('\\n')]){let rejected=false;try{manualModelCatalog(bad,42)}catch{rejected=true}if(!rejected)throw Error('invalid catalog accepted');}
const observed={models:['observed'],source:'CLI output',queried_at:1,cli_version:'1'};
if(JSON.stringify(manualModelCatalog(JSON.stringify(observed),42))!==JSON.stringify(observed))throw Error('observed provenance changed');
"""
        subprocess.run(["node", "-e", helper + "\n" + cases], check=True, capture_output=True, text=True)
    def test_ui_discloses_discovery_limit_and_keeps_existing_models(self):
        html=(ROOT / "src/ui/startup.html").read_text(encoding="utf-8")
        script=(ROOT / "src/ui/startup.js").read_text(encoding="utf-8")
        self.assertIn('This CLI does not expose model discovery',html)
        self.assertIn("p.catalog?.models.join('\\n')",script)
        self.assertIn("manualModelCatalog(q('catalog-json').value",script)
if __name__ == '__main__': unittest.main()
