use super::safe_scope_path;
use std::path::Path;

#[test]
fn glob_paths_resolve_to_their_parent_directory_on_all_platforms() {
    assert_eq!(
        safe_scope_path("src/**/*.rs"),
        Some(Path::new("src").into())
    );
    assert_eq!(
        safe_scope_path(r"src\**\*.rs"),
        Some(Path::new("src").into())
    );
}

#[test]
fn scope_paths_cannot_escape_the_worktree() {
    assert_eq!(safe_scope_path("../secrets"), None);
    assert_eq!(safe_scope_path("src/../../secrets"), None);
    assert_eq!(safe_scope_path(r"C:\secrets\*.txt"), None);
    assert_eq!(safe_scope_path(r"\\server\share\*.txt"), None);
}
