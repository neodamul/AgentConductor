mod driver;
mod files;
mod licenses;
mod notice;

#[cfg(test)]
#[path = "../tests/unit/release_package.rs"]
mod tests;

pub use driver::build_archive;

use files::{
    copy_file, project_version, referenced_assets, required_string, sha256_file, write_bytes,
    write_json,
};
use licenses::ReviewedDependency;
use notice::THIRD_PARTY_NOTICE;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn validate_company_suffix(file: &str, built: &str, cargo: &str) -> Result<(), String> {
    if file.trim().is_empty() && built == cargo {
        Ok(())
    } else {
        Err(
            "refusing to package a company-suffixed build; clear company-suffix.txt and rebuild"
                .into(),
        )
    }
}

pub fn assemble(
    root: &Path,
    destination: &Path,
    metadata: &Value,
    target: &str,
    commit: &str,
) -> Result<Value, String> {
    if !target.contains("apple") && !target.contains("windows") {
        return Err("unsupported release target".into());
    }
    if !commit
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || commit.len() != 40
    {
        return Err("exact lowercase source commit required".into());
    }
    match fs::symlink_metadata(destination) {
        Ok(_) => return Err(format!("refusing to overwrite {}", destination.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("inspect package destination: {error}")),
    }
    let root = root
        .canonicalize()
        .map_err(|error| format!("resolve repository: {error}"))?;
    let suffix = fs::read_to_string(root.join("company-suffix.txt"))
        .map_err(|error| format!("read company-suffix.txt: {error}"))?;
    validate_company_suffix(
        &suffix,
        &crate::update::current(),
        env!("CARGO_PKG_VERSION"),
    )?;
    let name = binary_name(target);
    let binary = root.join("target/release").join(name);
    if !binary.is_file() {
        return Err(format!(
            "native release binary missing: {}",
            binary.display()
        ));
    }
    let dependencies = licenses::review(&root, metadata)?;
    let icon = root.join("src/ui/assets/ac.icns");
    let version = if target.contains("apple") {
        if !icon.is_file() {
            return Err("AC launcher icon missing".into());
        }
        Some(project_version(&root.join("Cargo.toml"))?)
    } else {
        None
    };
    let assets = referenced_assets(&root)?;
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("create package parent: {error}"))?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let file_name = destination
        .file_name()
        .ok_or_else(|| "package destination has no file name".to_owned())?
        .to_string_lossy();
    let pending = parent.join(format!(
        ".{file_name}.{}.{}.pending",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&pending)
        .map_err(|error| format!("create package staging directory: {error}"))?;
    let result = write_package(PackagePlan {
        root: &root,
        destination: &pending,
        binary: &binary,
        name,
        target,
        commit,
        dependencies: &dependencies,
        assets: &assets,
        version: version.as_deref(),
        icon: &icon,
    });
    let manifest = match result {
        Ok(manifest) => manifest,
        Err(error) => {
            let _ = fs::remove_dir_all(&pending);
            return Err(error);
        }
    };
    if let Err(error) = fs::rename(&pending, destination) {
        let _ = fs::remove_dir_all(&pending);
        return Err(format!("publish package: {error}"));
    }
    Ok(manifest)
}

pub fn binary_name(target: &str) -> &'static str {
    if target.contains("windows") {
        "agentconductor.exe"
    } else {
        "agentconductor"
    }
}

struct PackagePlan<'a> {
    root: &'a Path,
    destination: &'a Path,
    binary: &'a Path,
    name: &'a str,
    target: &'a str,
    commit: &'a str,
    dependencies: &'a [ReviewedDependency<'a>],
    assets: &'a BTreeMap<String, PathBuf>,
    version: Option<&'a str>,
    icon: &'a Path,
}

fn write_package(plan: PackagePlan<'_>) -> Result<Value, String> {
    let PackagePlan {
        root,
        destination,
        binary,
        name,
        target,
        commit,
        dependencies,
        assets,
        version,
        icon,
    } = plan;
    copy_file(binary, &destination.join(name))?;
    if !target.contains("windows") {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(destination.join(name), fs::Permissions::from_mode(0o755))
                .map_err(|error| format!("set executable permission: {error}"))?;
        }
    }
    if let Some(version) = version {
        crate::macos_bundle::assemble(
            &destination.join(name),
            icon,
            &destination.join("AgentConductor.app"),
            version,
        )?;
    }
    for (relative, source) in assets {
        copy_file(source, &destination.join(relative))?;
    }
    for file in ["README.md", "LICENSE", "Cargo.lock"] {
        copy_file(&root.join(file), &destination.join(file))?;
    }
    // GitHub #180: the embedded skill's Apache-2.0 text and origin travel with the executable.
    // GitHub #406: the embedded Lobe Icons marks travel with their MIT text and pinned revision.
    for (name, file) in [
        ("prompt-optimizer", "LICENSE"),
        ("prompt-optimizer", "SOURCE.md"),
        ("lobe-icons", "LICENSE"),
        ("lobe-icons", "SOURCE.md"),
    ] {
        copy_file(
            &root.join("third_party").join(name).join(file),
            &destination.join("licenses").join(name).join(file),
        )?;
    }

    let mut components = Vec::new();
    let mut inventory = vec![
        "# Dependency license inventory".to_owned(),
        String::new(),
        format!("Target: {target}"),
        String::new(),
        "| Package | Version | SPDX expression |".to_owned(),
        "|---|---|---|".to_owned(),
    ];
    for dependency in dependencies {
        let package = dependency.package;
        let package_name = required_string(package, "name")?;
        let package_version = required_string(package, "version")?;
        let directory = destination
            .join("licenses")
            .join(format!("{package_name}-{package_version}"));
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        for source in &dependency.texts {
            let file = source
                .file_name()
                .ok_or_else(|| "license file has no name".to_owned())?;
            copy_file(source, &directory.join(file))?;
        }
        if let Some(notice) = &dependency.sqlite_notice {
            fs::write(
                directory.join("SQLITE-SOURCE-NOTICE.txt"),
                format!("{notice}\n"),
            )
            .map_err(|error| error.to_string())?;
        }
        let license = required_string(package, "license")?;
        let expression = license.replace("MIT/Apache-2.0", "MIT OR Apache-2.0");
        components.push(json!({
            "type":"library",
            "name":package_name,
            "version":package_version,
            "purl":format!("pkg:cargo/{package_name}@{package_version}"),
            "licenses":[{"expression":expression}]
        }));
        inventory.push(format!(
            "| {package_name} | {package_version} | {license} |"
        ));
    }
    write_bytes(
        destination,
        "DEPENDENCY-LICENSES.md",
        format!("{}\n", inventory.join("\n")).as_bytes(),
    )?;
    write_json(
        destination,
        "SBOM.cdx.json",
        &json!({"bomFormat":"CycloneDX","specVersion":"1.6","version":1,"components":components}),
    )?;
    write_bytes(
        destination,
        "THIRD-PARTY-NOTICES.md",
        THIRD_PARTY_NOTICE.as_bytes(),
    )?;
    let digest = sha256_file(&destination.join(name))?;
    write_bytes(
        destination,
        "SHA256SUMS",
        format!("{digest}  {name}\n").as_bytes(),
    )?;
    let binary_bytes = fs::metadata(binary)
        .map_err(|error| error.to_string())?
        .len();
    let manifest = json!({
        "name":"agentconductor",
        "target":target,
        "source_commit":commit,
        "binary":name,
        "binary_sha256":digest,
        "binary_bytes":binary_bytes,
        "dependency_count":dependencies.len(),
        "runtime":"one native executable; SQLite bundled",
        "external_requirements":["Git","selected authenticated agent CLIs",if target.contains("windows") { "Microsoft Edge WebView2 Runtime" } else { "system WebKit" }]
    });
    write_json(destination, "manifest.json", &manifest)?;
    Ok(manifest)
}
