#[path = "support/signal_workspace_registry.rs"]
mod workspace_registry;

struct TempDirectory(std::path::PathBuf);

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_home(label: &str) -> TempDirectory {
    let path = std::env::temp_dir().join(format!(
        "ac-signal-registry-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    TempDirectory(path)
}

#[test]
fn resolves_the_exact_first_run_managed_workspace() {
    let home = temp_home("first-run");
    let expected = home
        .0
        .join(".agentconductor")
        .join("workspaces")
        .join("windows-uia-workspace");
    std::fs::create_dir_all(&expected).unwrap();
    let registry = home.0.join(".agentconductor").join("workspaces.json");
    std::fs::write(&registry, serde_json::to_vec(&[&expected]).unwrap()).unwrap();
    assert_eq!(
        workspace_registry::active_workspace(&home.0, &expected),
        expected
    );
}

#[test]
fn waits_for_first_run_registry_persistence() {
    let home = temp_home("delayed");
    let expected = home
        .0
        .join(".agentconductor")
        .join("workspaces")
        .join("windows-uia-workspace");
    std::fs::create_dir_all(&expected).unwrap();
    let writer_home = home.0.clone();
    let writer_workspace = expected.clone();
    let writer = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let registry = writer_home.join(".agentconductor").join("workspaces.json");
        std::fs::write(registry, serde_json::to_vec(&[writer_workspace]).unwrap()).unwrap();
    });

    assert_eq!(
        workspace_registry::active_workspace(&home.0, &expected),
        expected
    );
    writer.join().unwrap();
}
