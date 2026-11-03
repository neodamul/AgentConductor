use super::licenses::ReviewedDependency;
use super::{THIRD_PARTY_NOTICE, copy_file, required_string, sha256_file, write_bytes, write_json};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub(super) struct PackagePlan<'a> {
    pub(super) root: &'a Path,
    pub(super) destination: &'a Path,
    pub(super) binary: &'a Path,
    pub(super) name: &'a str,
    pub(super) target: &'a str,
    pub(super) commit: &'a str,
    pub(super) dependencies: &'a [ReviewedDependency<'a>],
    pub(super) assets: &'a BTreeMap<String, PathBuf>,
    pub(super) version: Option<&'a str>,
    pub(super) icon: &'a Path,
}

pub(super) fn write_package(plan: PackagePlan<'_>) -> Result<Value, String> {
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
    for file in ["LICENSE", "SOURCE.md"] {
        copy_file(
            &root.join("third_party/prompt-optimizer").join(file),
            &destination.join("licenses/prompt-optimizer").join(file),
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
        let expression = license
            .replace("MIT/Apache-2.0", "MIT OR Apache-2.0")
            .replace("Apache-2.0/MIT", "Apache-2.0 OR MIT");
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
        "external_requirements":["Git","selected authenticated agent CLIs"]
    });
    write_json(destination, "manifest.json", &manifest)?;
    Ok(manifest)
}
