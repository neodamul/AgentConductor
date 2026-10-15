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
