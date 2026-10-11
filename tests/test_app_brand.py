"""Project-owned AC artwork is shared by native shells and embedded titlebars."""
import base64,struct,unittest
from pathlib import Path
class AppBrandTests(unittest.TestCase):
 def test_assets_have_valid_native_containers(self):
  root=Path("src/ui/assets")
  png=(root/"ac.png").read_bytes()
  self.assertEqual(png[:8],b"\x89PNG\r\n\x1a\n")
  self.assertEqual(struct.unpack(">II",png[16:24]),(1024,1024))
  ico=(root/"ac.ico").read_bytes();self.assertEqual(ico[:4],b"\0\0\1\0")
  icns=(root/"ac.icns").read_bytes();self.assertEqual(icns[:4],b"icns")
  self.assertEqual(struct.unpack(">I",icns[4:8])[0],len(icns))
  self.assertEqual((root/"ac-64.rgba").stat().st_size,64*64*4)
 def test_both_titlebars_use_the_same_asset_as_native_app(self):
  for page in ["startup.html","room.html"]:
   self.assertIn('class="brand-mark"',Path("src/ui",page).read_text())
   self.assertIn('data:image/png;base64,'+base64.b64encode(Path("src/ui/assets/ac-64.png").read_bytes()).decode(),Path("src/ui",page).read_text())
  self.assertIn('setApplicationIconImage:',Path("src/ui/native/cocoa/brand.rs").read_text())
  self.assertIn('with_window_icon',Path("src/ui/native/webview2.rs").read_text())
if __name__=="__main__":unittest.main()
