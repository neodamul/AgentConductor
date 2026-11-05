//! The shared secret of the local memory server, read from its bounded files.
use std::io::Read;

pub(super) fn bounded_file(path: &std::path::Path, max: u64) -> Result<Option<String>, String> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err("cannot read existing agentmemory authentication configuration".into());
        }
    };
    let mut value = String::new();
    file.take(max + 1)
        .read_to_string(&mut value)
        .map_err(|_| "invalid agentmemory authentication configuration")?;
    if value.len() as u64 > max {
        return Err("agentmemory authentication configuration exceeds limit".into());
    }
    Ok(Some(value))
}
pub(super) fn load_secret(home: &std::path::Path) -> Result<Option<String>, String> {
    if let Some(content) = bounded_file(&home.join(".env"), 64 * 1024)? {
        for line in content.lines().rev() {
            let line = line.trim().strip_prefix("export ").unwrap_or(line.trim());
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() != "AGENTMEMORY_SECRET" {
                continue;
            }
            let value = value.trim();
            let value = if value.starts_with(['\"', '\'']) {
                let quote = value.chars().next().unwrap();
                value[1..]
                    .split_once(quote)
                    .ok_or("invalid agentmemory secret quoting")?
                    .0
            } else {
                value.split(" #").next().unwrap_or_default().trim()
            };
            if !value.is_empty() && !value.starts_with("${") {
                return Ok(Some(value.into()));
            }
            break;
        }
    }
    Ok(bounded_file(&home.join("secret"), 4096)?
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty()))
}
