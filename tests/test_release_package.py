"""R-24 / DD-27: platform package contents match the exact binary and license closure."""
import hashlib
import importlib.util
import json
import tempfile
import sys
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"scripts"))

spec = importlib.util.spec_from_file_location("release_package", Path(__file__).resolve().parents[1] / "scripts/release_package.py")
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)

class PackageTests(unittest.TestCase):
    def fixture(self, root, target):
        (root / "README.md").write_text("# Actual product\n![UI](docs/assets/live.jpg)\n", encoding="utf-8")
        (root / "docs/assets").mkdir(parents=True)
        (root / "docs/assets/live.jpg").write_bytes(b"live fixture")
        (root / "docs/assets/mock.jpg").write_bytes(b"unreferenced mock")
        (root / "Cargo.lock").write_text("locked fixture", encoding="utf-8")
        (root / "Cargo.toml").write_text('[package]\nversion = "0.1.0"\n', encoding="utf-8")
        icon=root/"src/ui/assets/ac.icns";icon.parent.mkdir(parents=True)
        icon.write_bytes((Path(__file__).resolve().parents[1]/"src/ui/assets/ac.icns").read_bytes())
        (root / "LICENSE").write_text("MIT License\n\nCopyright (c) fixture\n", encoding="utf-8")
        binary = root / "target/release" / package.binary_name(target)
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"exact native binary")
        source = root / "dependency"
        source.mkdir()
        (source / "LICENSE-MIT").write_text("MIT fixture license", encoding="utf-8")
        metadata = {"resolve":{"nodes":[{"id":"dependency-id"}]},"packages":[{"id":"dependency-id","source":"registry+fixture","name":"dep","version":"1.0.0","license":"MIT","manifest_path":str(source / "Cargo.toml")} ]}
        return metadata

    def test_native_packages_include_exact_binary_notices_and_only_referenced_assets(self):
        for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"]:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                metadata = self.fixture(root,target)
                destination = root / "dist/package"
                package.assemble(root,destination,metadata,target,"a" * 40)
                if "apple" in target:
                    import plistlib
                    app=destination/"AgentConductor.app/Contents"
                    info=plistlib.loads((app/"Info.plist").read_bytes())
                    self.assertEqual(info["CFBundleIconFile"],"ac.icns")
                    self.assertEqual(info["CFBundleShortVersionString"],"0.1.0")
                    self.assertEqual((app/"MacOS/agentconductor").read_bytes(),b"exact native binary")
                name = package.binary_name(target)
                self.assertEqual((destination/name).read_bytes(),b"exact native binary")
                manifest = json.loads((destination/"manifest.json").read_text(encoding="utf-8"))
                self.assertEqual(manifest["source_commit"],"a" * 40)
                self.assertNotIn("mockup",json.dumps(manifest).lower())
                # Published text must verify with `shasum -c` on any platform (no CRLF).
                for generated in ["SHA256SUMS","manifest.json","SBOM.cdx.json","DEPENDENCY-LICENSES.md","THIRD-PARTY-NOTICES.md"]:
                    self.assertNotIn(b"\r\n",(destination/generated).read_bytes(),generated)
                prerequisite="Microsoft Edge WebView2 Runtime" if "windows" in target else "system WebKit"
                self.assertIn(prerequisite,manifest["external_requirements"])

                self.assertEqual(manifest["binary_sha256"],hashlib.sha256(b"exact native binary").hexdigest())
                self.assertTrue((destination/"licenses/dep-1.0.0/LICENSE-MIT").is_file())
                self.assertTrue((destination/"SBOM.cdx.json").is_file())
                self.assertTrue((destination/"THIRD-PARTY-NOTICES.md").is_file())
                self.assertTrue((destination/"docs/assets/live.jpg").is_file())
                # The product's own MIT license ships with every package.
                self.assertEqual((destination/"LICENSE").read_text(encoding="utf-8"),"MIT License\n\nCopyright (c) fixture\n")
                self.assertFalse((destination/"docs/assets/mock.jpg").exists())
                self.assertEqual((destination/"SHA256SUMS").read_text(encoding="utf-8"),manifest["binary_sha256"]+"  "+name+"\n")
                self.assertFalse((root/"knowledge").exists())
                with self.assertRaises(FileExistsError):
                    package.assemble(root,destination,metadata,target,"a" * 40)

    def test_omitted_crate_text_requires_exact_reviewed_upstream_provenance(self):
        for mode in ["valid", "wrong-revision", "wrong-version"]:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);metadata=self.fixture(root,"x86_64-pc-windows-msvc")
                entry=metadata["packages"][0]
                entry.update(name="webview2-com",version="0.39.1")
                source=root/"dependency"
                (source/"LICENSE-MIT").unlink()
                revision="edc2caf886175ccaebe86078c9cfe1ae2a187328"
                if mode=="wrong-revision": revision="b"*40
                if mode=="wrong-version": entry["version"]="0.39.2"
                (source/".cargo_vcs_info.json").write_text(json.dumps({"git":{"sha1":revision}}), encoding="utf-8")
                destination=root/"dist/package"
                if mode=="valid":
                    package.assemble(root,destination,metadata,"x86_64-pc-windows-msvc","a"*40)
                    text=(destination/"licenses/webview2-com-0.39.1/LICENSE").read_text(encoding="utf-8")
                    self.assertIn("Copyright (c) 2021 Bill Avery",text)
                else:
                    with self.assertRaises(ValueError):
                        package.assemble(root,destination,metadata,"x86_64-pc-windows-msvc","a"*40)
                    self.assertFalse(destination.exists())

    def test_autocrlf_checkout_preserves_official_license_bytes(self):
        import subprocess
        import shutil
        project=Path(__file__).resolve().parents[1]
        relative="licenses/upstream/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE"
        original=(project/relative).read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);license_path=root/relative
            license_path.parent.mkdir(parents=True);license_path.write_bytes(original)
            if (project/".gitattributes").exists():
                shutil.copy2(project/".gitattributes",root/".gitattributes")
            def git(*args):
                subprocess.run(["git",*args],cwd=root,check=True,capture_output=True)
            git("init","--quiet")
            git("-c","core.autocrlf=false","add",".")
            license_path.unlink()
            git("-c","core.autocrlf=true","checkout","--",relative)
            self.assertEqual(hashlib.sha256(license_path.read_bytes()).hexdigest(),hashlib.sha256(original).hexdigest())

    def test_modified_upstream_license_refuses_before_package_output(self):
        from unittest.mock import patch
        import shutil
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);metadata=self.fixture(root,"x86_64-pc-windows-msvc")
            metadata["packages"][0].update(name="webview2-com",version="0.39.1")
            source=root/"dependency";(source/"LICENSE-MIT").unlink()
            revision="edc2caf886175ccaebe86078c9cfe1ae2a187328"
            (source/".cargo_vcs_info.json").write_text(json.dumps({"git":{"sha1":revision}}), encoding="utf-8")
            retained=root/"retained";shutil.copytree(package.REVIEWED_LICENSES,retained)
            (retained/revision/"LICENSE").write_text("altered", encoding="utf-8")
            destination=root/"dist/package"
            with patch.object(package,"REVIEWED_LICENSES",retained):
                with self.assertRaisesRegex(ValueError,"digest mismatch"):
                    package.assemble(root,destination,metadata,"x86_64-pc-windows-msvc","a"*40)
            self.assertFalse(destination.exists())

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


class ArchiveChecksumTests(unittest.TestCase):
    def test_archive_checksum_file_is_written_with_lf_line_endings(self):
        source=(Path(__file__).resolve().parents[1]/"scripts/build_release_artifact.py").read_text(encoding="utf-8")
        # Text mode would translate to CRLF on Windows; checksum files are written as bytes.
        self.assertNotIn("write_text(",source)
