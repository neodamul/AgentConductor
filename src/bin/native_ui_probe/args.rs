use std::path::PathBuf;

pub fn workspace_path(args: &[String]) -> Result<Option<PathBuf>, String> {
    match args {
        [] => Ok(None),
        [flag, path] if flag == "--workspace" && !path.trim().is_empty() => {
            Ok(Some(PathBuf::from(path)))
        }
        _ => Err(
            "usage: agentconductor-native-ui-probe [--workspace <dir>] | [--room <run-dir>]".into(),
        ),
    }
}
