use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(super) fn within(path: &str, scope: &str) -> bool {
    let (path, scope) = (
        path.to_lowercase(),
        scope.trim_end_matches('/').to_lowercase(),
    );
    path == scope
        || path
            .strip_prefix(&scope)
            .is_some_and(|tail| tail.starts_with('/'))
}

fn relative(path: &str) -> Result<PathBuf, String> {
    let relative = Path::new(path);
    if path.is_empty()
        || path.contains('\\')
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("unsafe quarantined recovery path".into());
    }
    Ok(relative.to_path_buf())
}

pub(super) fn safe_target(root: &Path, path: &str) -> Result<PathBuf, String> {
    let relative = relative(path)?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut target = root.clone();
    let components: Vec<_> = relative.components().collect();
    for (index, component) in components.iter().enumerate() {
        target.push(component);
        match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_symlink() && index + 1 < components.len() => {
                return Err("recovery refuses symbolic-link paths".into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    if !target.starts_with(root) {
        return Err("recovery path escaped its workspace".into());
    }
    Ok(target)
}

pub(super) fn quarantined(dir: &Path, path: &str) -> Result<PathBuf, String> {
    let root = dir.join("stray-files");
    match fs::symlink_metadata(&root) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("recovery refuses a symbolic-link quarantine directory".into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(root.join(relative(path)?));
        }
        Err(error) => return Err(error.to_string()),
        _ => {}
    }
    safe_target(&root, path)
}

pub(super) fn restore_exclusive(from: &Path, to: &Path) -> Result<(), String> {
    let mut source = fs::File::open(from).map_err(|e| e.to_string())?;
    let mut target = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(to)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "restore destination appeared after review; preview recovery again".into()
            } else {
                error.to_string()
            }
        })?;
    if let Err(error) = std::io::copy(&mut source, &mut target) {
        drop(target);
        let _ = fs::remove_file(to);
        return Err(error.to_string());
    }
    target.sync_all().map_err(|e| e.to_string())
}
