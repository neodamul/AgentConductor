#[cfg(windows)]
#[path = "support/env.rs"]
mod env;

#[test]
fn windows_package_suite_includes_deleted_workspace_reopen_journey() {
    let workflow = include_str!("../.github/workflows/platform-validation.yml");
    assert!(workflow.contains("--test windows_workspace_reopen"));
    assert!(
        include_str!("windows_workspace_reopen.rs")
            .contains("exact_windows_package_reopens_after_default_workspace_deletion")
    );
}

#[cfg(windows)]
mod package {
    use super::env;
    use agentconductor::{
        leases::Lease,
        ui::{Room, workspaces},
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        process::{Child, Command, Stdio},
        thread,
        time::{Duration, Instant},
    };

    struct TestDirectory(PathBuf);
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct ChildProcess(Child);
    impl Drop for ChildProcess {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn call(room: &Room, value: serde_json::Value) -> serde_json::Value {
        room.dispatch(value.to_string().as_bytes()).unwrap()
    }

    fn save_name(room: &Room) {
        let revision = call(room, serde_json::json!({"kind":"snapshot"}))["revision"].clone();
        call(
            room,
            serde_json::json!({
                "kind":"save",
                "revision":revision,
                "config":{"profiles":[],"repository":"","goal":"","coordinator":"","workspace_mode":"shared","name":"Deleted default"}
            }),
        );
    }

    #[test]
    #[ignore = "Runs only against the exact packaged binary."]
    fn exact_windows_package_reopens_after_default_workspace_deletion() {
        let root = std::env::temp_dir().join(format!("ac-workspace-reopen-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let _cleanup = TestDirectory(root.clone());
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let mut environment = env::lock();
        environment.set("HOME", &home);
        environment.set("USERPROFILE", &home);

        let deleted = workspaces::launch_root().unwrap();
        let room =
            Room::startup(&deleted, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
        save_name(&room);
        call(
            &room,
            serde_json::json!({"kind":"create_workspace","name":"Survivor"}),
        );
        let entries = call(&room, serde_json::json!({"kind":"saved_workspaces"}));
        let survivor = PathBuf::from(
            entries["workspaces"]
                .as_array()
                .unwrap()
                .iter()
                .find(|workspace| workspace["name"] == "Survivor")
                .unwrap()["path"]
                .as_str()
                .unwrap(),
        );
        let deleted_entry = entries["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|workspace| workspace["path"] == deleted.to_string_lossy().as_ref())
            .unwrap();
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":deleted_entry["path"],"name":deleted_entry["name"]}),
        );
        drop(room);
        assert!(!deleted.exists());
        assert_eq!(workspaces::launch_root().unwrap(), survivor);

        let binary = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
            .expect("package job supplies AGENTCONDUCTOR_DESKTOP_BINARY");
        let mut app = ChildProcess(
            Command::new(binary)
                .env("HOME", &home)
                .env("USERPROFILE", &home)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        let selected_survivor = loop {
            if let Some(status) = app.0.try_wait().unwrap() {
                panic!("package exited before opening a workspace: {status}");
            }
            match Lease::acquire(&survivor.join("ui-window.lock")) {
                Err(error) if error.starts_with("lease is held:") => break true,
                Ok(lease) => drop(lease),
                Err(error) => panic!("could not inspect survivor UI lease: {error}"),
            }
            if Instant::now() >= deadline {
                break false;
            }
            thread::sleep(Duration::from_millis(50));
        };
        assert!(
            selected_survivor,
            "package did not open the saved workspace; default recreated: {}",
            deleted.exists()
        );
        assert!(
            !deleted.exists(),
            "the deleted default workspace was recreated"
        );
    }
}
