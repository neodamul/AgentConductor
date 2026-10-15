use agentconductor::manual::render;
use std::{fs, path::PathBuf};

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    (args == ["manual", "render"]).then(render_manuals)
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
