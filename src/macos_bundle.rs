use std::{fs, path::Path};

pub fn assemble(
    binary: &Path,
    icon: &Path,
    destination: &Path,
    version: &str,
) -> Result<(), String> {
    if destination.exists() {
        return Err(format!("refusing to overwrite {}", destination.display()));
    }
    if !binary.is_file() || !icon.is_file() {
        return Err("macOS bundle executable or icon is missing".into());
    }
    let contents = destination.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    let result = (|| {
        fs::create_dir_all(&macos)
            .map_err(|e| format!("create bundle executable directory: {e}"))?;
        fs::create_dir_all(&resources)
            .map_err(|e| format!("create bundle resources directory: {e}"))?;
        let executable = macos.join("agentconductor");
        fs::copy(binary, &executable).map_err(|e| format!("copy bundle executable: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("set bundle executable permissions: {e}"))?;
        }
        fs::copy(icon, resources.join("ac.icns")).map_err(|e| format!("copy bundle icon: {e}"))?;
        let version = escape_xml(version);
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>agentconductor</string>
<key>CFBundleIdentifier</key><string>dev.agentconductor.desktop</string>
<key>CFBundleName</key><string>AgentConductor</string>
<key>CFBundleDisplayName</key><string>AgentConductor</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleIconFile</key><string>ac.icns</string>
<key>CFBundleShortVersionString</key><string>{version}</string>
<key>CFBundleVersion</key><string>{version}</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
"#
        );
        fs::write(contents.join("Info.plist"), plist)
            .map_err(|e| format!("write bundle Info.plist: {e}"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
