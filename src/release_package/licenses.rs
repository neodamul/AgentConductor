use serde_json::Value;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use super::{required_string, sha256_file};

const ALLOWED: &[&str] = &[
    "MIT",
    "ISC",
    "BSL-1.0",
    "MIT OR Apache-2.0",
    "Apache-2.0 OR MIT",
    "MIT/Apache-2.0",
    "Unlicense OR MIT",
    "Zlib",
    "(MIT OR Apache-2.0) AND Unicode-3.0",
    "Apache-2.0",
    "Apache-2.0 AND MIT",
    "Unicode-3.0",
    "CC0-1.0 OR MIT-0 OR Apache-2.0",
    "MIT OR Apache-2.0 OR Zlib",
];
const NOTICE_PREFIXES: &[&str] = &["LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT"];

pub(super) struct ReviewedDependency<'a> {
    pub(super) package: &'a Value,
    pub(super) texts: Vec<PathBuf>,
    pub(super) sqlite_notice: Option<String>,
}

pub(super) fn review<'a>(
    root: &Path,
    metadata: &'a Value,
) -> Result<Vec<ReviewedDependency<'a>>, String> {
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or_else(|| "cargo metadata omitted dependency resolution".to_owned())?;
    let ids: HashSet<&str> = nodes
        .iter()
        .filter_map(|node| node["id"].as_str())
        .collect();
    let mut packages: Vec<&Value> = metadata["packages"]
        .as_array()
        .ok_or_else(|| "cargo metadata omitted packages".to_owned())?
        .iter()
        .filter(|package| {
            package["source"].as_str().is_some()
                && package["id"].as_str().is_some_and(|id| ids.contains(id))
        })
        .collect();
    packages.sort_by_key(|package| {
        (
            package["name"].as_str().unwrap_or_default(),
            package["version"].as_str().unwrap_or_default(),
        )
    });
    let mut result = Vec::new();
    for package in packages {
        let name = required_string(package, "name")?;
        let version = required_string(package, "version")?;
        let license = package["license"]
            .as_str()
            .ok_or_else(|| format!("unreviewed license: {name} {}", package["license"]))?;
        if !ALLOWED.contains(&license) {
            return Err(format!("unreviewed license: {name} {license}"));
        }
        if [name, version]
            .iter()
            .any(|part| part.contains(['/', '\\']) || *part == "..")
        {
            return Err(format!("invalid dependency identity: {name} {version}"));
        }
        let source = PathBuf::from(required_string(package, "manifest_path")?)
            .parent()
            .ok_or_else(|| format!("dependency manifest has no parent: {name}"))?
            .to_owned();
        let mut texts = Vec::new();
        for entry in fs::read_dir(&source)
            .map_err(|error| format!("read license directory for {name}: {error}"))?
        {
            let path = entry
                .map_err(|error| format!("read license entry for {name}: {error}"))?
                .path();
            if path.is_file()
                && path.file_name().is_some_and(|file| {
                    let upper = file.to_string_lossy().to_ascii_uppercase();
                    NOTICE_PREFIXES
                        .iter()
                        .any(|prefix| upper.starts_with(prefix))
                })
            {
                texts.push(path);
            }
        }
        texts.sort();
        if texts.is_empty() {
            texts = upstream_texts(root, package, &source)?;
        }
        let sqlite_notice = if name == "libsqlite3-sys" {
            Some(sqlite_disclaimer(&source.join("sqlite3/sqlite3.c"))?)
        } else {
            None
        };
        result.push(ReviewedDependency {
            package,
            texts,
            sqlite_notice,
        });
    }
    Ok(result)
}

fn upstream_texts(root: &Path, package: &Value, source: &Path) -> Result<Vec<PathBuf>, String> {
    let directory = root.join("licenses/upstream");
    let provenance = directory.join("sources.json");
    let records: Value = serde_json::from_slice(
        &fs::read(&provenance)
            .map_err(|error| format!("read reviewed license sources: {error}"))?,
    )
    .map_err(|error| format!("parse reviewed license sources: {error}"))?;
    let matches: Vec<&Value> = records
        .as_array()
        .into_iter()
        .flatten()
        .filter(|record| {
            ["name", "version", "license"]
                .iter()
                .all(|key| record[*key] == package[*key])
        })
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "missing reviewed license text: {}",
            package["name"]
        ));
    }
    let record = matches[0];
    let vcs: Value = serde_json::from_slice(
        &fs::read(source.join(".cargo_vcs_info.json"))
            .map_err(|_| format!("license source revision mismatch: {}", package["name"]))?,
    )
    .map_err(|_| format!("invalid license source revision: {}", package["name"]))?;
    if vcs["git"]["sha1"] != record["vcs_commit"] {
        return Err(format!(
            "license source revision mismatch: {}",
            package["name"]
        ));
    }
    let relative = record["file"]
        .as_str()
        .ok_or_else(|| format!("invalid reviewed license path: {}", package["name"]))?;
    let base = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = directory
        .join(relative)
        .canonicalize()
        .map_err(|_| format!("reviewed license file missing: {}", package["name"]))?;
    if !path.starts_with(&base) || !path.is_file() {
        return Err(format!(
            "reviewed license file missing: {}",
            package["name"]
        ));
    }
    if sha256_file(&path)? != record["sha256"].as_str().unwrap_or_default() {
        return Err(format!(
            "reviewed license digest mismatch: {}",
            package["name"]
        ));
    }
    Ok(vec![path, provenance])
}

fn sqlite_disclaimer(path: &Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(8192).read_to_end(&mut bytes))
        .map_err(|error| format!("read bundled SQLite source: {error}"))?;
    let text = String::from_utf8_lossy(&bytes);
    let mut rest = text.as_ref();
    while let Some(start) = rest.find("/*") {
        let block = &rest[start + 2..];
        let Some(end) = block.find("*/") else { break };
        let candidate = &block[..end];
        if candidate
            .to_ascii_lowercase()
            .contains("author disclaims copyright")
        {
            return Ok(format!("/*{candidate}*/"));
        }
        rest = &block[end + 2..];
    }
    Err("bundled SQLite source disclaimer not found".into())
}
