import json
import re
import subprocess
import unittest
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class MessageMarkup(HTMLParser):
    def __init__(self):
        super().__init__()
        self.stack = []
        self.unmarked = []
    def handle_starttag(self, tag, pairs):
        attrs = dict(pairs)
        if tag not in {"input", "img", "br", "hr", "meta", "link"}:
            self.stack.append(tag)
        for attr in ("title", "aria-label", "placeholder"):
            value = attrs.get(attr, "")
            if re.search(r"[A-Za-z가-힣]", value) and "data-i18n-" + attr not in attrs:
                self.unmarked.append(value)
        if tag == "option" and any(name.startswith("data-i18n-") for name in attrs):
            self.unmarked.append("option label must bind its text")
    def handle_endtag(self, tag):
        if self.stack:
            self.stack.pop()
    def handle_data(self, value):
        value = value.strip()
        if not value or any(t in {"code", "textarea", "script", "style", "title"} for t in self.stack):
            return
        invariant = {"AgentConductor", "AC", "English", "한국어", "Claude Code", "Codex", "OpenCode", "Jira", "Confluence"}
        if value not in invariant and not value.startswith("v{{APP_VERSION}}") and re.search(r"[A-Za-z가-힣]", value):
            self.unmarked.append(value)

class MessageCatalogs(unittest.TestCase):
    def load_catalog(self, language):
        def unique(pairs):
            result = {}
            for key, value in pairs:
                self.assertNotIn(key, result, "duplicate ID: " + key)
                result[key] = value
            return result
        return json.loads((ROOT / f"src/ui/messages/{language}.json").read_text(encoding="utf-8"), object_pairs_hook=unique)
    def test_catalog_ids_and_placeholders_match(self):
        en, ko = self.load_catalog("en"), self.load_catalog("ko")
        self.assertEqual(set(en), set(ko))
        for key, english in en.items():
            self.assertRegex(key, r"^[a-z][a-z0-9_]*\.[a-z0-9_.]+$")
            self.assertIsInstance(ko[key], str)
            self.assertTrue(ko[key], key)
            placeholders = lambda text: set(re.findall(r"\{([a-zA-Z][a-zA-Z0-9_]*)\}", text))
            self.assertEqual(placeholders(english), placeholders(ko[key]), key)
    def test_every_explicit_reference_has_a_catalog_entry(self):
        en = self.load_catalog("en")
        pattern = r'''(?:acMessage|messages::text|messages::render)\(\s*["']([^"']+)["']|data-i18n(?:-title|-placeholder|-aria-label)?=["']([^"']+)["']'''
        for file in [ROOT / "src/ui.rs", *(ROOT / "src/ui").rglob("*")]:
            if file.suffix not in {".rs", ".js", ".html"}:
                continue
            for match in re.finditer(pattern, file.read_text(encoding="utf-8")):
                key = match.group(1) or match.group(2)
                self.assertIn(key, en, str(file) + ": " + key)
    def test_static_ui_text_has_explicit_message_bindings(self):
        for name in ["startup.html", "room.html", "mcp.html"]:
            parser = MessageMarkup()
            parser.feed((ROOT / "src/ui" / name).read_text(encoding="utf-8"))
            self.assertEqual(parser.unmarked, [], name)
    def test_runtime_behavior(self):
        subprocess.run(["node", "tests/i18n_runtime.cjs"], cwd=ROOT, check=True, capture_output=True)
    def test_old_source_text_matching_is_removed(self):
        source = (ROOT / "src/ui/i18n.js").read_text(encoding="utf-8")
        self.assertNotIn("acSwap", source)
        self.assertNotIn("acInvert", source)
        self.assertFalse((ROOT / "src/ui/i18n_ko.js").exists())

if __name__ == "__main__":
    unittest.main()
