"""R-24 / DD-27: platform package contents match the exact binary and license closure."""
import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("release_package", Path(__file__).resolve().parents[1] / "scripts/release_package.py")
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)

class PackageTests(unittest.TestCase):
    def fixture(self, root, target):
        (root / "README.md").write_text("# Actual product\n![UI](docs/assets/live.jpg)\n")
        (root / "docs/assets").mkdir(parents=True)
        (root / "docs/assets/live.jpg").write_bytes(b"live fixture")
        (root / "docs/assets/mock.jpg").write_bytes(b"unreferenced mock")
        (root / "Cargo.lock").write_text("locked fixture")
        binary = root / "target/release" / package.binary_name(target)
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"exact native binary")
        source = root / "dependency"
        source.mkdir()
        (source / "LICENSE-MIT").write_text("MIT fixture license")
        metadata = {"resolve":{"nodes":[{"id":"dependency-id"}]},"packages":[{"id":"dependency-id","source":"registry+fixture","name":"dep","version":"1.0.0","license":"MIT","manifest_path":str(source / "Cargo.toml")} ]}
        return metadata

    def test_native_packages_include_exact_binary_notices_and_only_referenced_assets(self):
        for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"]:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                metadata = self.fixture(root,target)
                destination = root / "dist/package"
                package.assemble(root,destination,metadata,target,"a" * 40)
                name = package.binary_name(target)
                self.assertEqual((destination/name).read_bytes(),b"exact native binary")
                manifest = json.loads((destination/"manifest.json").read_text())
                self.assertEqual(manifest["source_commit"],"a" * 40)
                self.assertNotIn("mockup",json.dumps(manifest).lower())
                self.assertEqual(manifest["binary_sha256"],hashlib.sha256(b"exact native binary").hexdigest())
                self.assertTrue((destination/"licenses/dep-1.0.0/LICENSE-MIT").is_file())
                self.assertTrue((destination/"SBOM.cdx.json").is_file())
                self.assertTrue((destination/"THIRD-PARTY-NOTICES.md").is_file())
                self.assertTrue((destination/"docs/assets/live.jpg").is_file())
                self.assertFalse((destination/"docs/assets/mock.jpg").exists())
                self.assertEqual((destination/"SHA256SUMS").read_text(),manifest["binary_sha256"]+"  "+name+"\n")
                self.assertFalse((root/"knowledge").exists())
                with self.assertRaises(FileExistsError):
                    package.assemble(root,destination,metadata,target,"a" * 40)

    def test_unreviewed_or_missing_licenses_refuse_before_writing_package(self):
        for mode in ["unreviewed","missing"]:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);metadata=self.fixture(root,"aarch64-apple-darwin")
                if mode=="unreviewed": metadata["packages"][0]["license"]="AGPL-3.0-only"
                else: (root/"dependency/LICENSE-MIT").unlink()
                destination=root/"dist/package"
                with self.assertRaises(ValueError):
                    package.assemble(root,destination,metadata,"aarch64-apple-darwin","a"*40)
                self.assertFalse(destination.exists())

if __name__ == "__main__":
    unittest.main()
