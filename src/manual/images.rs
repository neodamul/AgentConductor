use super::{base64, quoted};
use std::path::Path;

pub(super) fn embed(label: &str, url: &str, directory: &Path) -> Result<String, String> {
    let root = directory.canonicalize().map_err(|e| e.to_string())?;
    let assets = root
        .join("assets")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let path = root.join(url).canonicalize().map_err(|e| e.to_string())?;
    if !path.starts_with(&assets) {
        return Err(format!(
            "manual image must be a local PNG or JPEG asset: {url}"
        ));
    }
    let mime = match path.extension().and_then(|s| s.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("png") => "image/png",
        Some(ext) if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") => {
            "image/jpeg"
        }
        _ => {
            return Err(format!(
                "manual image must be a local PNG or JPEG asset: {url}"
            ));
        }
    };
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(format!(
        "<img alt=\"{}\" src=\"data:{mime};base64,{}\">",
        quoted(label),
        base64(&data)
    ))
}
