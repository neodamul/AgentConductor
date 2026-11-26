use super::*;

pub fn scan(root: &Path) -> Result<BTreeMap<String, FileState>, String> {
    scan_with_ignored(root, |_| {})
}

pub(super) fn scan_with_ignored(
    root: &Path,
    mut ignored: impl FnMut(&str),
) -> Result<BTreeMap<String, FileState>, String> {
    let metadata = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if !ordinary(&metadata) || !metadata.is_dir() {
        return Err("original folder alias is unsupported".into());
    }
    fn walk(
        root: &Path,
        path: &Path,
        files: &mut BTreeMap<String, FileState>,
        total: &mut u64,
        ignored: &mut impl FnMut(&str),
    ) -> Result<(), String> {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "folder paths must be UTF-8")?;
            let path = entry.path();
            let relative = || {
                path.strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .map(|path| {
                        if cfg!(windows) {
                            path.replace('\\', "/")
                        } else {
                            path.into()
                        }
                    })
                    .ok_or_else(|| "folder paths must be UTF-8".to_string())
            };
            if excluded(&name) {
                ignored(&relative()?);
                continue;
            }
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if !ordinary(&metadata) {
                return Err(format!(
                    "folder aliases are unsupported: {}",
                    path.display()
                ));
            }
            // A workspace's goal folder (marked by slugs::create_tree) is not project source.
            if metadata.is_dir() && path.join(crate::slugs::MARKER).is_file() {
                ignored(&relative()?);
                continue;
            }
            if metadata.is_dir() {
                walk(root, &path, files, total, ignored)?;
            } else {
                let value = state(&path)?.ok_or("folder file vanished during snapshot")?;
                *total += value.bytes;
                if files.len() >= MAX_FILES || *total > MAX_TOTAL {
                    return Err(
                        "folder snapshot exceeds 10,000 files or 256 MiB; use a Git project".into(),
                    );
                }
                files.insert(relative()?, value);
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files, &mut 0, &mut ignored)?;
    Ok(files)
}
