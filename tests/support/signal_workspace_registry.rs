use std::{
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

pub fn active_workspace(home: &Path, expected: &Path) -> PathBuf {
    let registry = home.join(".agentconductor").join("workspaces.json");
    let expected = expected
        .canonicalize()
        .unwrap_or_else(|error| panic!("expected first-run workspace is unavailable: {error}"));
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut observed = String::from("registry has not been written");
    while Instant::now() < deadline {
        match std::fs::read(&registry) {
            Ok(bytes) => match serde_json::from_slice::<Vec<PathBuf>>(&bytes) {
                Ok(paths) => {
                    observed = format!("{paths:?}");
                    if let Some(path) = paths.into_iter().find(|path| {
                        path.canonicalize()
                            .is_ok_and(|registered| registered == expected)
                    }) {
                        return path;
                    }
                }
                Err(error) => observed = format!("invalid registry: {error}"),
            },
            Err(error) => observed = format!("registry unavailable: {error}"),
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!(
        "first-run workspace was not registered before timeout; expected={expected:?}; observed={observed}"
    )
}
