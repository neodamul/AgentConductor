"""Offline HTML preserves manual tables and escapes non-document markup."""
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))

class ManualHtml(unittest.TestCase):
    def renderer(self):
        from render_manual import render
        return render
    def test_table_code_and_links_are_semantic_and_safe(self):
        text="| Command | Purpose |\n|---|---|\n| `status RUN` | **Inspect** [docs](https://example.com) |\n\n<script>bad()</script>\n\n[bad](javascript:alert)\n\n```rust\nlet x = \"<script>\";\n```\n"
        html=self.renderer()(text,ROOT/'docs')
        self.assertIn('<table>',html)
        pipes=self.renderer()(r"| Command | Purpose |"+"\n|---|---|\n"+r"| `normalize claude\|codex` | inspect |",ROOT/'docs')
        self.assertIn('<td><code>normalize claude|codex</code></td>',pipes)
        self.assertIn('<th>Command</th>',html)
        self.assertIn('<td><code>status RUN</code></td>',html)
        self.assertIn('<strong>Inspect</strong>',html)
        self.assertIn('href="https://example.com"',html)
        self.assertNotIn('<script>',html)
        self.assertNotIn('href="javascript:',html)
        self.assertIn('&lt;script&gt;',html)
    def test_images_are_offline_and_paths_cannot_escape_assets(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'assets').mkdir();(root/'assets/test.png').write_bytes(b'png')
            html=self.renderer()('![Screen](assets/test.png)',root)
            self.assertIn('data:image/png;base64,cG5n',html)
            with self.assertRaises(ValueError):self.renderer()('![Bad](../secret.png)',root)
    def test_generated_english_and_korean_files_match_the_sources(self):
        render=self.renderer()
        for name in ['user-manual','user-manual.ko']:
            source=ROOT/'docs'/(name+'.md')
            self.assertEqual((ROOT/'docs'/(name+'.html')).read_text(encoding='utf-8'),render(source.read_text(encoding='utf-8'),source.parent))

if __name__=='__main__':unittest.main()
