mod archive;
mod driver;
mod licenses;

use archive::{PackagePlan, write_package};
pub use driver::build_archive;

use serde_json::Value;
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
terms require separate review if redistributed. The desktop UI uses Rust eframe;
no WebView runtime or local UI server is required.

The bundled prompt-optimizer skill and references are from
https://github.com/getsentry/skills and licensed under Apache-2.0, Copyright
2025 Functional Software, Inc. dba Sentry. Its license and source commit are in
licenses/prompt-optimizer.
";

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
    let binary = root.join("target/release").join(binary_name(target));
    assemble_with_binary(root, destination, metadata, target, commit, &binary)
}

pub fn assemble_with_binary(
    root: &Path,
    destination: &Path,
    metadata: &Value,
    target: &str,
    commit: &str,
    binary: &Path,
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
        binary,
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

fn referenced_assets(root: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let docs = root
        .join("docs")
        .canonicalize()
        .map_err(|error| format!("resolve docs: {error}"))?;
    let mut assets = BTreeMap::new();
    for (document, prefix, base, label) in [
        ("README.md", "docs/", root, "README"),
        ("docs/user-manual.md", "assets/", &docs, "manual"),
        ("docs/user-manual.ko.md", "assets/", &docs, "manual"),
    ] {
        let path = root.join(document);
        let text =
            fs::read_to_string(&path).map_err(|error| format!("read {document}: {error}"))?;
        let mut rest = text.as_str();
        while let Some(end) = rest.find("](") {
            rest = &rest[end + 2..];
            let Some(close) = rest.find(')') else { break };
            let raw = rest[..close].trim();
            let raw = raw
                .strip_prefix('<')
                .and_then(|value| value.strip_suffix('>'))
                .unwrap_or(raw);
            let relative = raw.split_ascii_whitespace().next().unwrap_or("");
            if relative.starts_with(prefix) {
                let source = base
                    .join(relative)
                    .canonicalize()
                    .map_err(|_| format!("missing or invalid {label} asset: {relative}"))?;
                if !source.starts_with(&docs) || !source.is_file() {
                    return Err(format!("missing or invalid {label} asset: {relative}"));
                }
                let package_path = if prefix == "assets/" {
                    format!("docs/{relative}")
                } else {
                    relative.to_owned()
                };
                assets.insert(package_path, source);
            }
            rest = &rest[close + 1..];
        }
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
