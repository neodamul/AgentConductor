pub fn section(changelog: &str, tag: &str) -> Result<String, String> {
    let lines: Vec<_> = changelog.lines().collect();
    let heading = format!("## {tag}");
    let start = lines
        .iter()
        .position(|line| line.trim() == heading)
        .ok_or_else(|| format!("CHANGELOG.md has no section for {tag}"))?;
    let end = lines[start + 1..]
        .iter()
        .position(|line| line.starts_with("## "))
        .map_or(lines.len(), |offset| start + 1 + offset);
    let body = lines[start + 1..end].join("\n");
    if body.trim().is_empty() {
        return Err(format!("CHANGELOG.md section for {tag} is empty"));
    }
    Ok(format!("{}\n", body.trim()))
}
