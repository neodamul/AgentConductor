//! Display limits never weaken verification of the complete candidate patch.
use super::*;
use sha2::{Digest, Sha256};
pub(super) fn read(
    path: &Path,
    limit: usize,
    expected: Option<&str>,
) -> Result<(String, bool), String> {
    let mut file = open_artifact(path)?;
    let mut digest = Sha256::new();
    let mut preview = Vec::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 32768];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
        let shown = count.min(limit.saturating_sub(preview.len()));
        preview.extend_from_slice(&buffer[..shown]);
        total = total.saturating_add(count);
    }
    let actual = format!("{:x}", digest.finalize());
    if expected.is_some_and(|hash| hash != actual) {
        return Err(messages::text("native.candidate_diff_changed_verification_required").into());
    }
    Ok((
        String::from_utf8_lossy(&preview).into_owned(),
        total > limit,
    ))
}
