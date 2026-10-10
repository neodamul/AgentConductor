"""ZIP packaging with registry epoch timestamps bounded to ZIP's supported range."""
import os
import zipfile
from pathlib import Path


def archive_package(package):
    package=Path(package).resolve()
    archive=package.with_suffix(".zip")
    pending=package.with_suffix(".zip.pending")
    if archive.exists() or pending.exists():
        raise FileExistsError("Refusing to overwrite release archive")
    with zipfile.ZipFile(pending,"x",compression=zipfile.ZIP_DEFLATED,strict_timestamps=False) as output:
        for path in sorted(package.rglob("*")):
            if path.is_symlink():
                raise ValueError("Package contains a symbolic link")
            if path.is_file():
                output.write(path,path.relative_to(package.parent))
    os.rename(pending,archive)
    return archive
