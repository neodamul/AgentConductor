#!/usr/bin/env python3
"""Assemble a local native release and its pinned dependency/license evidence."""
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
host = subprocess.check_output(["rustc", "-vV"], text=True).split("host: ")[1].splitlines()[0]
destination = root / "dist" / f"agentconductor-{host}"
if destination.exists():
    sys.exit(f"Refusing to overwrite {destination}; archive or remove the old package first")
metadata = json.loads(subprocess.check_output([
    "cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", host
], cwd=root))
ids = {node["id"] for node in metadata["resolve"]["nodes"]}
packages = sorted((p for p in metadata["packages"] if p["id"] in ids and p["source"]), key=lambda p:p["name"])
allowed = {"MIT", "MIT OR Apache-2.0", "Apache-2.0 OR MIT", "MIT/Apache-2.0", "Unlicense OR MIT", "Zlib", "(MIT OR Apache-2.0) AND Unicode-3.0"}
for package in packages:
    if package["license"] not in allowed:
        sys.exit(f"Unreviewed license: {package['name']} {package['license']}")
    source = Path(package["manifest_path"]).parent
    texts = [p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT"))]
    if not texts:
        sys.exit(f"Missing license text: {package['name']}")

destination.mkdir(parents=True)
binary = destination / "agentconductor"
shutil.copy2(root / "target/release/agentconductor", binary)
binary.chmod(0o755)
shutil.copy2(root / "README.md", destination / "README.md")
shutil.copytree(root / "docs/assets", destination / "docs/assets")
shutil.copy2(root / "docs/interface-gallery.md", destination / "docs/interface-gallery.md")
shutil.copy2(root / "Cargo.lock", destination / "Cargo.lock")
components = []
inventory = [f"# Rust dependency license inventory\n\nTarget: {host}. Lockfile-resolved runtime/build dependency closure: {len(packages)} packages.\n\nLicense declarations and bundled texts were checked against the reviewed permissive allowlist.\nThis inventory does not grant a license to the private root project or certify provider terms.\n\n| Package | Version | Declared SPDX expression |\n|---|---|---|"]
for package in packages:
    source = Path(package["manifest_path"]).parent
    target = destination / "licenses" / f"{package['name']}-{package['version']}"
    target.mkdir(parents=True)
    for path in source.iterdir():
        if path.is_file() and path.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")):
            shutil.copy2(path, target / path.name)
    if package["name"] == "libsqlite3-sys":
        sqlite_source = source / "sqlite3/sqlite3.c"
        with sqlite_source.open() as stream:
            start = stream.read(8192)
        notice = next((block for block in re.findall(r"/\*.*?\*/",start,re.S) if "author disclaims copyright" in block.lower()),None)
        if notice is None:
            sys.exit("Bundled SQLite copyright-disclaimer source notice not found")
        (target / "SQLITE-SOURCE-NOTICE.txt").write_text(notice+"\n")
    expression = package["license"].replace("MIT/Apache-2.0", "MIT OR Apache-2.0")
    components.append({"type":"library", "name":package["name"], "version":package["version"],
                       "purl":f"pkg:cargo/{package['name']}@{package['version']}",
                       "licenses":[{"expression":expression}]})
    inventory.append(f"| {package['name']} | {package['version']} | {package['license']} |")
(destination / "SBOM.cdx.json").write_text(json.dumps({"bomFormat":"CycloneDX","specVersion":"1.6","version":1,"components":components},indent=2)+"\n")
(root / "knowledge/wiki/rust-dependencies.md").write_text("\n".join(inventory)+"\n")
(destination / "THIRD-PARTY-NOTICES.md").write_text("""# Third-party notices

The licenses directory contains the pinned Rust dependency license texts, including
build dependencies, Unicode data notices, and the bundled SQLite source notice.
Dual MIT/Apache declarations may be used under their MIT option; Unicode-3.0 and
Zlib requirements remain applicable where declared. Review all included notices
when redistributing the executable. The private AgentConductor root package has
no public license grant. Its owner controls internal distribution.

Claude Code, Codex, OpenCode, Git, agentmemory, Graphify, CocoIndex, RTK, and Ponytail
are not bundled. Their licenses, authentication policies, and provider terms must
be reviewed separately if an organization distributes them. No iii/Elastic-licensed
runtime or optional indexing service is included in this executable.
""")
checksum = hashlib.sha256(binary.read_bytes()).hexdigest()
(destination / "SHA256SUMS").write_text(f"{checksum}  agentconductor\n")
manifest = {"name":"agentconductor", "version":"0.1.0", "target":host,
            "source_commit":subprocess.check_output(["git","rev-parse","HEAD"],cwd=root,text=True).strip(),
            "binary_sha256":checksum,"binary_bytes":binary.stat().st_size,
            "dependency_count":len(packages),"runtime":"one native executable; SQLite bundled",
            "external_requirements":["Git","selected authenticated agent CLIs"],
            "scope":"CLI MVP; UI images are mockups; native provider metadata queries tested separately from mocked workflow execution"}
(destination / "manifest.json").write_text(json.dumps(manifest,indent=2)+"\n")
print(json.dumps({"directory":str(destination),"sha256":checksum,"bytes":binary.stat().st_size,"dependencies":len(packages)}))
