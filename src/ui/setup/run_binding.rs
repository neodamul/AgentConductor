//! A planning copy and a direct folder execution share a reviewed source binding.
use super::*;
pub(super) fn matches(
    planned: &Path,
    selected: &Path,
    base: &str,
    actual: &Path,
) -> Result<bool, String> {
    if actual == planned {
        return Ok(true);
    }
    let Some(source) = crate::folder::load(planned)? else {
        return Ok(false);
    };
    if source.base != base
        || source.original != actual
        || source.original != selected.canonicalize().map_err(|e| e.to_string())?
    {
        return Ok(false);
    }
    // Workers are expected to change source files. Check identity, not baseline content.
    Ok(Repository::open(actual)?.is_external())
}
