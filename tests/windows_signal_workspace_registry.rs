#[path = "support/signal_workspace_registry.rs"]
mod workspace_registry;

#[test]
fn resolves_the_exact_first_run_managed_workspace() {
    let home = std::env::temp_dir().join(format!(
        "ac-signal-registry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let expected = home
        .join(".agentconductor")
        .join("workspaces")
        .join("windows-uia-workspace");
    std::fs::create_dir_all(&expected).unwrap();
    let registry = home.join(".agentconductor").join("workspaces.json");
    std::fs::write(&registry, serde_json::to_vec(&[&expected]).unwrap()).unwrap();
    assert_eq!(
        workspace_registry::active_workspace(&home, &expected),
        expected
    );
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn waits_for_first_run_registry_persistence() {
    let home = std::env::temp_dir().join(format!(
        "ac-signal-registry-delay-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let expected = home
        .join(".agentconductor")
        .join("workspaces")
        .join("windows-uia-workspace");
    std::fs::create_dir_all(&expected).unwrap();
    std::fs::write(expected.join("startup.json"), b"{}").unwrap();
    let writer_home = home.clone();
    let writer_workspace = expected.clone();
    let writer = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let registry = writer_home.join(".agentconductor").join("workspaces.json");
        std::fs::write(registry, serde_json::to_vec(&[writer_workspace]).unwrap()).unwrap();
    });

    assert_eq!(
        workspace_registry::active_workspace(&home, &expected),
        expected
    );
    writer.join().unwrap();
    std::fs::remove_dir_all(home).unwrap();
}
