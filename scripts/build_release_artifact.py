#!/usr/bin/env python3
"""CI: assemble, execute, then archive an already built native release binary."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from smoke_release import run
from release_archive import archive_package


def main():
    root=Path(__file__).resolve().parent.parent
    result=subprocess.check_output([sys.executable,root/"scripts/package_release.py"],cwd=root,text=True)
    package=Path(json.loads(result)["directory"])
    run(package,root/".agentconductor/verification/packaged-workflow")
    archive=archive_package(package)
    checksum=hashlib.sha256(archive.read_bytes()).hexdigest()
    (root/"dist/ARCHIVE-SHA256SUMS").write_text(checksum+"  "+archive.name+"\n")
    print(json.dumps({"archive":str(archive),"sha256":checksum}))


if __name__=="__main__":
    main()
