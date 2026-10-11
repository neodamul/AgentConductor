"""R-24: integrity validation must precede execution of a packaged binary."""
import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("smoke",Path(__file__).resolve().parents[1]/"scripts/smoke_release.py")
smoke=importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)

class SmokeTests(unittest.TestCase):
    def test_modified_binary_and_escaping_manifest_paths_are_refused(self):
        for name,checksum in [("agentconductor", "0"*64),("../agentconductor", hashlib.sha256(b"fixture").hexdigest())]:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);(root/"agentconductor").write_bytes(b"fixture")
                (root/"manifest.json").write_text(json.dumps({"binary":name,"binary_sha256":checksum,"source_commit":"a"*40}), encoding="utf-8")
                with self.assertRaises(ValueError): smoke.validated_binary(root)

    def test_checked_binary_is_the_exact_package_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);binary=root/"agentconductor";binary.write_bytes(b"fixture")
            manifest={"binary":"agentconductor","binary_sha256":hashlib.sha256(b"fixture").hexdigest(),"source_commit":"a"*40}
            (root/"manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
            (root/"SHA256SUMS").write_text(manifest["binary_sha256"]+"  agentconductor\n", encoding="utf-8")
            self.assertEqual(smoke.validated_binary(root),(binary.resolve(),manifest))

if __name__=="__main__": unittest.main()
