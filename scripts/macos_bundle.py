"""Optional Finder launcher around the same standalone Rust binary."""
import plistlib
import shutil
from pathlib import Path

def assemble(binary, icon, destination, version):
    destination=Path(destination)
    if destination.exists(): raise FileExistsError(destination)
    contents=destination/"Contents"
    (contents/"MacOS").mkdir(parents=True)
    (contents/"Resources").mkdir()
    shutil.copy2(binary,contents/"MacOS/agentconductor")
    (contents/"MacOS/agentconductor").chmod(0o755)
    shutil.copy2(icon,contents/"Resources/ac.icns")
    info={"CFBundleExecutable":"agentconductor","CFBundleIdentifier":"dev.agentconductor.desktop",
          "CFBundleName":"AgentConductor","CFBundleDisplayName":"AgentConductor",
          "CFBundlePackageType":"APPL","CFBundleIconFile":"ac.icns",
          "CFBundleShortVersionString":version,"CFBundleVersion":version,"NSHighResolutionCapable":True}
    (contents/"Info.plist").write_bytes(plistlib.dumps(info))
    return destination
