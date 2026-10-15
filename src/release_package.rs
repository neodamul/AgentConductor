mod driver;
mod licenses;

pub use driver::build_archive;

use licenses::ReviewedDependency;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const THIRD_PARTY_NOTICE: &str = "# Third-party notices

The licenses directory includes the pinned dependency license texts and notices,
including build dependencies, Unicode data and the bundled SQLite disclaimer.
Preserve these texts when distributing. Apache-2.0 components and conjunctions
retain their Apache notices; dual declarations and alternatives retain their
available texts. ISC and BSL-1.0 licenses are permissive and their bundled
copyright and license notices must be preserved. AgentConductor and its AC
artwork use the root MIT license.

Git, selected agent CLIs, provider artwork, RTK, Ponytail, Graphify, CocoIndex,
agentmemory and its optional services are not bundled. Their licenses and account
terms require separate review if redistributed. Windows uses the installed system
WebView2 runtime; macOS uses system WebKit. No UI web server is required.
";

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

fn referenced_assets(root: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let readme = fs::read_to_string(root.join("README.md"))
        .map_err(|error| format!("read README: {error}"))?;
    let docs = root
        .join("docs")
        .canonicalize()
        .map_err(|error| format!("resolve docs: {error}"))?;
    let mut assets = BTreeMap::new();
    let mut rest = readme.as_str();
    while let Some(end) = rest.find("](") {
        rest = &rest[end + 2..];
        let Some(close) = rest.find(')') else { break };
        let raw = rest[..close].trim();
        let raw = raw
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
            .unwrap_or(raw);
        let relative = raw.split_ascii_whitespace().next().unwrap_or("");
        if relative.starts_with("docs/") {
            let source = root
                .join(relative)
                .canonicalize()
                .map_err(|_| format!("missing or invalid README asset: {relative}"))?;
            if !source.starts_with(&docs) || !source.is_file() {
                return Err(format!("missing or invalid README asset: {relative}"));
            }
            assets.insert(relative.to_owned(), source);
        }
        rest = &rest[close + 1..];
    }
    Ok(assets)
}

fn project_version(manifest: &Path) -> Result<String, String> {
    let text = fs::read_to_string(manifest).map_err(|error| error.to_string())?;
    let mut in_package = false;
    for line in text.lines() {
        if line.starts_with('[') {
            in_package = line.trim() == "[package]";
        } else if in_package && let Some(value) = line.trim().strip_prefix("version = ") {
            return Ok(value.trim().trim_matches('"').to_owned());
        }
    }
    Err("package version missing from Cargo.toml".into())
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("cargo metadata omitted {key}"))
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::copy(source, destination)
        .map(|_| ())
        .map_err(|error| format!("copy {}: {error}", source.display()))
}

fn write_bytes(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

fn write_json(root: &Path, name: &str, value: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_bytes(root, name, &bytes)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
