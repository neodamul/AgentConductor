use agentconductor::manual::render;
use std::{fs, path::PathBuf};

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    if args == ["manual", "render"] {
        return Some(render_manuals());
    }
    if args.len() == 3 && args[0] == "release" && args[1] == "notes" {
        return Some(release_notes(&args[2]));
    }
    if args.len() == 5 && args[0] == "bundle-macos" {
        return Some(agentconductor::macos_bundle::assemble(
            &PathBuf::from(&args[1]),
            &PathBuf::from(&args[2]),
            &PathBuf::from(&args[3]),
            &args[4],
        ));
    }
    if args.len() == 2 && args[0] == "archive-package" {
        return Some(archive_package(&args[1]));
    }
    if args.len() == 2 && args[0] == "validate-package" {
        return Some(validate_package(&args[1]));
    }
    if args.len() == 3 && args[0] == "smoke-package" {
        return Some(smoke_package(&args[1], &args[2]));
    }
    None
}

fn archive_package(package: &str) -> Result<(), String> {
    let path = agentconductor::release_archive::archive_package(&PathBuf::from(package))?;
    println!("{}", path.display());
    Ok(())
}

fn validate_package(package: &str) -> Result<(), String> {
    let validated = agentconductor::release_smoke::validate_package(&PathBuf::from(package))?;
    println!(
        "{}",
        serde_json::json!({"binary":validated.binary,"manifest":validated.manifest})
    );
    Ok(())
}

fn smoke_package(package: &str, evidence: &str) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let result = runtime.block_on(agentconductor::release_smoke::run_smoke(
        &PathBuf::from(package),
        &PathBuf::from(evidence),
    ))?;
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn release_notes(tag: &str) -> Result<(), String> {
    let changelog = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.md");
    let text = fs::read_to_string(changelog).map_err(|e| format!("read changelog: {e}"))?;
    let section = agentconductor::release_notes::section(&text, tag)?;
    std::io::Write::write_all(&mut std::io::stdout().lock(), section.as_bytes())
        .map_err(|e| format!("write release notes: {e}"))
}

fn render_manuals() -> Result<(), String> {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    for name in ["user-manual", "user-manual.ko"] {
        let source = docs.join(format!("{name}.md"));
        let target = docs.join(format!("{name}.html"));
        let markdown = fs::read_to_string(source).map_err(|e| e.to_string())?;
        fs::write(&target, render(&markdown, &docs)?).map_err(|e| e.to_string())?;
        println!("{}", target.display());
    }
    Ok(())
}
