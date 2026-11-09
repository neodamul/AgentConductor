//! GitHub #614: a merge or cherry-pick conflict marker left in a tracked file broke main twice (spec.cjs).

#[test]
fn no_tracked_file_contains_a_conflict_marker() {
    let listing = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .filter(|o| o.status.success());
    // shortcut: a source archive without .git skips the check; CI always has it.
    let Some(listing) = listing else {
        return;
    };
    let (open, close) = ("<".repeat(7) + " ", ">".repeat(7) + " ");
    let mut found = Vec::new();
    for name in String::from_utf8_lossy(&listing.stdout)
        .split('\0')
        .filter(|n| !n.is_empty())
    {
        let Ok(text) =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        else {
            continue; // binary or deleted
        };
        for (number, line) in text.lines().enumerate() {
            if line.starts_with(&open) || line.starts_with(&close) {
                found.push(format!("{name}:{}", number + 1));
            }
        }
    }
    assert!(found.is_empty(), "conflict markers in: {found:?}");
}
