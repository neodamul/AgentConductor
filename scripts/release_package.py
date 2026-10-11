"""Native package assembly; no provider binaries, artwork or internal plans are bundled."""
import hashlib
import json
import re
import shutil
from pathlib import Path

ALLOWED = {
    "MIT", "MIT OR Apache-2.0", "Apache-2.0 OR MIT", "MIT/Apache-2.0",
    "Unlicense OR MIT", "Zlib", "(MIT OR Apache-2.0) AND Unicode-3.0",
    "Apache-2.0", "Apache-2.0 AND MIT", "Unicode-3.0",
    "CC0-1.0 OR MIT-0 OR Apache-2.0", "MIT OR Apache-2.0 OR Zlib",
}
PREFIXES = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")


def binary_name(target):
    return "agentconductor.exe" if "windows" in target else "agentconductor"


REVIEWED_LICENSES = Path(__file__).resolve().parents[1] / "licenses/upstream"


def upstream_texts(package, source):
    provenance = REVIEWED_LICENSES / "sources.json"
    records = json.loads(provenance.read_text())
    matches = [record for record in records if all(record[key] == package[key] for key in ("name", "version", "license"))]
    if len(matches) != 1:
        raise ValueError("Missing reviewed license text: " + package["name"])
    record = matches[0]
    vcs_file = source / ".cargo_vcs_info.json"
    if not vcs_file.is_file() or json.loads(vcs_file.read_text()).get("git", {}).get("sha1") != record["vcs_commit"]:
        raise ValueError("License source revision mismatch: " + package["name"])
    text = (REVIEWED_LICENSES / record["file"]).resolve()
    if not text.is_relative_to(REVIEWED_LICENSES.resolve()) or not text.is_file():
        raise ValueError("Reviewed license file missing: " + package["name"])
    if hashlib.sha256(text.read_bytes()).hexdigest() != record["sha256"]:
        raise ValueError("Reviewed license digest mismatch: " + package["name"])
    return [text, provenance]


def reviewed_dependencies(metadata):
    ids = {node["id"] for node in metadata["resolve"]["nodes"]}
    packages = sorted((p for p in metadata["packages"] if p["id"] in ids and p["source"]), key=lambda p: (p["name"],p["version"]))
    reviewed = []
    for package in packages:
        if package["license"] not in ALLOWED:
            raise ValueError("Unreviewed license: " + package["name"] + " " + str(package["license"]))
        source = Path(package["manifest_path"]).parent
        texts = sorted(p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(PREFIXES))
        if not texts:
            texts = upstream_texts(package, source)
        sqlite_notice = None
        if package["name"] == "libsqlite3-sys":
            with (source / "sqlite3/sqlite3.c").open() as stream:
                start = stream.read(8192)
            sqlite_notice = next((block for block in re.findall(r"/\*.*?\*/",start,re.S) if "author disclaims copyright" in block.lower()),None)
            if sqlite_notice is None:
                raise ValueError("Bundled SQLite source disclaimer not found")
        reviewed.append((package,texts,sqlite_notice))
    return reviewed


def assemble(root, destination, metadata, target, commit):
    root, destination = Path(root), Path(destination)
    if destination.exists():
        raise FileExistsError("Refusing to overwrite " + str(destination))
    if not re.fullmatch(r"[0-9a-f]{40}",commit):
        raise ValueError("Exact source commit required")
    name = binary_name(target)
    binary = root / "target/release" / name
    if not binary.is_file():
        raise ValueError("Native release binary missing: " + str(binary))
    dependencies = reviewed_dependencies(metadata)
    mac_icon = root/"src/ui/assets/ac.icns"
    if "apple" in target:
        if not mac_icon.is_file(): raise ValueError("AC launcher icon missing")
        version = re.search(r'^version = "([^"]+)"', (root/"Cargo.toml").read_text(), re.M).group(1)
    readme = (root/"README.md").read_text()
    assets = []
    for relative in re.findall(r"!?\[[^]]*\]\(([^)]+)\)",readme):
        if relative.startswith("docs/"):
            source = (root/relative).resolve()
            if not source.is_relative_to((root/"docs").resolve()) or not source.is_file():
                raise ValueError("Missing or invalid README asset: " + relative)
            assets.append((source,relative))
    destination.mkdir(parents=True)
    shutil.copy2(binary,destination/name)
    if "windows" not in target:
        (destination/name).chmod(0o755)
    if "apple" in target:
        from macos_bundle import assemble as assemble_app
        assemble_app(destination/name,mac_icon,destination/"AgentConductor.app",version)
    for source,relative in assets:
        path = destination/relative
        path.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(source,path)
    shutil.copy2(root/"README.md",destination/"README.md")
    shutil.copy2(root/"LICENSE",destination/"LICENSE")
    shutil.copy2(root/"Cargo.lock",destination/"Cargo.lock")
    components=[]
    inventory=["# Dependency license inventory", "", "Target: " + target, "", "| Package | Version | SPDX expression |", "|---|---|---|"]
    for package,texts,sqlite_notice in dependencies:
        path=destination/"licenses"/(package["name"]+"-"+package["version"])
        path.mkdir(parents=True)
        for text in texts:
            shutil.copy2(text,path/text.name)
        if sqlite_notice:
            (path/"SQLITE-SOURCE-NOTICE.txt").write_bytes((sqlite_notice+"\n").encode("utf-8"))
        expression=package["license"].replace("MIT/Apache-2.0","MIT OR Apache-2.0")
        components.append({"type":"library","name":package["name"],"version":package["version"],"purl":"pkg:cargo/"+package["name"]+"@"+package["version"],"licenses":[{"expression":expression}]})
        inventory.append("| "+package["name"]+" | "+package["version"]+" | "+package["license"]+" |")
    (destination/"DEPENDENCY-LICENSES.md").write_bytes(("\n".join(inventory)+"\n").encode("utf-8"))
    (destination/"SBOM.cdx.json").write_bytes((json.dumps({"bomFormat":"CycloneDX","specVersion":"1.6","version":1,"components":components},indent=2)+"\n").encode("utf-8"))
    (destination/"THIRD-PARTY-NOTICES.md").write_bytes(("""# Third-party notices

The licenses directory includes the pinned dependency license texts and notices,
including build dependencies, Unicode data and the bundled SQLite disclaimer.
Preserve these texts when distributing. Apache-2.0 components and conjunctions
retain their Apache notices; dual declarations and alternatives retain their
available texts. AgentConductor and its AC artwork use the root MIT license.

Git, selected agent CLIs, provider artwork, RTK, Ponytail, Graphify, CocoIndex,
agentmemory and its optional services are not bundled. Their licenses and account
terms require separate review if redistributed. Windows uses the installed system
WebView2 runtime; macOS uses system WebKit. No UI web server is required.
""").encode("utf-8"))
    checksum=hashlib.sha256((destination/name).read_bytes()).hexdigest()
    (destination/"SHA256SUMS").write_bytes((checksum+"  "+name+"\n").encode("utf-8"))
    manifest={"name":"agentconductor","target":target,"source_commit":commit,"binary":name,
              "binary_sha256":checksum,"binary_bytes":binary.stat().st_size,
              "dependency_count":len(dependencies),"runtime":"one native executable; SQLite bundled",
              "external_requirements":["Git","selected authenticated agent CLIs",
                  "Microsoft Edge WebView2 Runtime" if "windows" in target else "system WebKit"]}
    (destination/"manifest.json").write_bytes((json.dumps(manifest,indent=2)+"\n").encode("utf-8"))
    return manifest
