use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

pub async fn build_archive() -> Result<Value, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let changes = checked(
        Command::new("git")
            .args(["diff", "HEAD", "--name-only"])
            .current_dir(&root),
        "check source revision",
    )?;
    if !changes.is_empty() {
        return Err("commit tracked source changes before packaging".into());
    }
    let toolchain = checked(
        Command::new("rustc").args(["-vV"]).current_dir(&root),
        "inspect Rust target",
    )?;
    let host = String::from_utf8_lossy(&toolchain)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc omitted host target".to_owned())?
        .to_owned();
    let metadata = checked(
        Command::new("cargo")
            .args([
                "metadata",
                "--locked",
                "--offline",
                "--format-version",
                "1",
                "--filter-platform",
            ])
            .arg(&host)
            .current_dir(&root),
        "read Cargo dependency metadata",
    )?;
    let metadata: Value = serde_json::from_slice(&metadata)
        .map_err(|error| format!("parse cargo metadata: {error}"))?;
    let commit = String::from_utf8_lossy(&checked(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&root),
        "read source commit",
    )?)
    .trim()
    .to_owned();
    let package = root.join("dist").join(format!(
        "agentconductor-{host}-{}",
        commit.get(..12).ok_or("source commit is too short")?
    ));
    super::assemble(&root, &package, &metadata, &host, &commit)?;
    let evidence = root.join(".agentconductor/verification/packaged-workflow");
    crate::release_smoke::run_smoke(&package, &evidence).await?;
    let archive = crate::release_archive::archive_package(&package)?;
    let checksum = sha256_file(&archive)?;
    std::fs::write(
        root.join("dist/ARCHIVE-SHA256SUMS"),
        format!(
            "{checksum}  {}\n",
            archive.file_name().unwrap_or_default().to_string_lossy()
        ),
    )
    .map_err(|error| format!("write archive checksum: {error}"))?;
    Ok(json!({"archive":archive.display().to_string(),"sha256":checksum}))
}

fn checked(command: &mut Command, label: &str) -> Result<Vec<u8>, String> {
    let output = command
        .output()
        .map_err(|error| format!("{label}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{label} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open archive: {error}"))?;
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
