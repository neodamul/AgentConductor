#[path = "../src/bin/native_ui_probe/args.rs"]
mod args;

#[test]
fn workspace_capture_accepts_an_explicit_isolated_directory() {
    let path = std::path::PathBuf::from("/private/tmp/ac-capture/workspace");
    let parsed = args::workspace_path(&["--workspace".into(), path.display().to_string()]).unwrap();
    assert_eq!(parsed.as_deref(), Some(path.as_path()));
}

#[test]
fn workspace_capture_defaults_to_the_app_workspace() {
    assert_eq!(args::workspace_path(&[]).unwrap(), None);
}

#[test]
fn workspace_capture_rejects_missing_or_unexpected_arguments() {
    assert!(args::workspace_path(&["--workspace".into()]).is_err());
    assert!(args::workspace_path(&["--other".into()]).is_err());
}
