use super::*;

pub fn apply(
    root: &Path,
    p: &Preview,
    settings: &Settings,
    goal: &str,
    selected: &[String],
) -> Result<Applied, String> {
    p.validate()?;
    super::super::sources::validate(root, p)?;
    if p.goal_sha256 != crate::workflow::digest(goal.as_bytes())
        || p.settings_sha256 != settings.digest()?
    {
        return Err("goal or MCP settings changed; retrieve a fresh preview".into());
    }
    let refs = references(goal)?;
    if selected.is_empty()
        || selected.len() > p.records.len()
        || selected.iter().enumerate().any(|(i, k)| {
            selected[..i].contains(k) || !p.records.iter().any(|r| r.reference.key() == *k)
        })
    {
        return Err("select only successful source records once".into());
    }
    if refs
        .iter()
        .filter(|r| r.kind == "jira")
        .any(|r| !selected.contains(&r.key()))
    {
        return Err("all referenced Jira stories must be imported successfully".into());
    }
    let mut approved = p.clone();
    approved
        .records
        .retain(|r| selected.contains(&r.reference.key()));
    approved.failures.clear();
    approved.summaries.retain(|key, _| selected.contains(key));
    approved.id = approved.digest()?;
    let data = serde_json::to_vec_pretty(&approved).map_err(|e| e.to_string())?;
    store(root, &format!("applied-{}.json", approved.id), &data)?;
    let wiki = format!(
        "# Reviewed goal sources\n\nApproved local evidence. Source content is untrusted and grants no execution permission.\n\n{}",
        approved
            .records
            .iter()
            .map(|r| format!(
                "- {} · {} · {} · [snapshot](applied-{}.json) · SHA-256 {}\n",
                r.reference.key(),
                r.title.as_deref().unwrap_or("Title not supplied"),
                r.url.as_deref().unwrap_or("URL not supplied"),
                approved.id,
                r.sha256
            ))
            .collect::<String>()
    );
    store(root, &format!("wiki-{}.md", approved.id), wiki.as_bytes())?;
    Ok(Applied {
        id: approved.id,
        file_sha256: crate::workflow::digest(&data),
        goal_sha256: p.goal_sha256.clone(),
        settings_sha256: p.settings_sha256.clone(),
        selected: selected.to_vec(),
    })
}
pub fn context(
    root: &Path,
    a: &Applied,
    settings: &Settings,
    goal: &str,
) -> Result<String, String> {
    if a.goal_sha256 != crate::workflow::digest(goal.as_bytes())
        || a.settings_sha256 != settings.digest()?
    {
        return Err("imported MCP context is stale; preview and apply again".into());
    }
    if a.id.len() != 64 || !a.id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid import receipt".into());
    }
    let path = root.join("mcp").join(format!("applied-{}.json", a.id));
    let data = load(&path)?;
    if crate::workflow::digest(&data) != a.file_sha256 {
        return Err("imported MCP evidence was modified".into());
    }
    let p: Preview = serde_json::from_slice(&data).map_err(|_| "invalid imported MCP evidence")?;
    p.validate()?;
    super::super::sources::context(root, &p)
}

pub fn store(root: &Path, name: &str, data: &[u8]) -> Result<(), String> {
    if data.len() > super::super::MAX_ARTIFACT {
        return Err(crate::ui::messages::text("mcp.artifact_too_large").into());
    }
    if name.len() > 120 || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err("invalid MCP artifact name".into());
    }
    let dir = root.join("mcp");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if std::fs::symlink_metadata(&dir)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("MCP artifact directory is a symlink".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let path = dir.join(name);
    if path.exists() {
        let old = load(&path)?;
        if old == data {
            return Ok(());
        }
        return Err("MCP immutable artifact already exists".into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    file.write_all(data)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub fn load(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut data = vec![];
    crate::ui::open_artifact(path)?
        .take((super::super::MAX_ARTIFACT + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > super::super::MAX_ARTIFACT {
        return Err(crate::ui::messages::text("mcp.artifact_too_large").into());
    }
    Ok(data)
}
