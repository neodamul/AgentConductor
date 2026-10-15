#!/usr/bin/env python3
"""CI: assemble, execute, then archive an already built native release binary."""
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path


def main():
    root=Path(__file__).resolve().parent.parent
    result=subprocess.check_output([sys.executable,root/"scripts/package_release.py"],cwd=root,text=True)
    package=Path(json.loads(result)["directory"])
    executable="agentconductor.exe" if os.name=="nt" else "agentconductor"
    runner=root/"target"/"release"/executable
    subprocess.check_output([runner,"smoke-package",package,root/".agentconductor/verification/packaged-workflow"],cwd=root,text=True)
    archiver=runner
    archive=Path(subprocess.check_output([archiver,"archive-package",package],cwd=root,text=True).strip())
    checksum=hashlib.sha256(archive.read_bytes()).hexdigest()
    (root/"dist/ARCHIVE-SHA256SUMS").write_bytes((checksum+"  "+archive.name+"\n").encode("utf-8"))
    print(json.dumps({"archive":str(archive),"sha256":checksum}))


if __name__=="__main__":
    main()
