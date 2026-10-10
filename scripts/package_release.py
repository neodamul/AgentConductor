#!/usr/bin/env python3
"""Package the already built native release for this host from a clean source revision."""
import json
import subprocess
from pathlib import Path
from release_package import assemble


def main():
    root=Path(__file__).resolve().parent.parent
    if subprocess.check_output(["git","diff","HEAD","--name-only"],cwd=root).strip():
        raise SystemExit("Commit tracked source changes before packaging")
    host=subprocess.check_output(["rustc","-vV"],text=True).split("host: ")[1].splitlines()[0]
    metadata=json.loads(subprocess.check_output(["cargo","metadata","--locked","--offline","--format-version","1","--filter-platform",host],cwd=root))
    commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=root,text=True).strip()
    destination=root/"dist"/("agentconductor-"+host+"-"+commit[:12])
    manifest=assemble(root,destination,metadata,host,commit)
    print(json.dumps({"directory":str(destination),**manifest}))


if __name__ == "__main__":
    main()
