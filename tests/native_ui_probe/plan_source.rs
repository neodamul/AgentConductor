use super::*;
use crate::native_ui_probe::{RequestWorker, workspace_save_request};
use eframe::egui;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct TemporaryWorkspace(std::path::PathBuf);

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn imported_plan_request_uses_the_saved_revision_and_selected_path() {
    let packet = request("read_plan", "revision-7", Some("/tmp/plan.md"));
    assert_eq!(
        serde_json::from_slice::<Value>(&packet).unwrap(),
        serde_json::json!({
            "kind": "read_plan",
            "revision": "revision-7",
            "path": "/tmp/plan.md"
        })
    );
}

#[test]
fn manual_plan_action_creates_a_reviewable_plan_without_starting_an_agent() {
    let root = TemporaryWorkspace(std::env::temp_dir().join(format!(
        "agentconductor-native-manual-plan-{}-{}",
        std::process::id(),
        crate::native_guard::session_id().unwrap()
    )));
    let repository = root.0.join("repository");
    std::fs::create_dir_all(&repository).unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
    ] {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(&repository)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    std::fs::write(repository.join("base.txt"), "base\n").unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["-C", repository.to_str().unwrap(), "add", "."])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .args(["-C", repository.to_str().unwrap(), "commit", "-m", "base"])
            .output()
            .unwrap()
            .status
            .success()
    );

    let room =
        Arc::new(crate::ui::Room::startup(&root.0, &std::env::current_exe().unwrap()).unwrap());
    let initial = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let profile = initial["templates"][0].clone();
    room.dispatch(&workspace_save_request(
        &initial,
        "Manual plan",
        repository.to_str().unwrap(),
        "Add a useful feature",
        crate::plan::WorkspaceMode::Shared,
        std::slice::from_ref(&profile),
        profile["id"].as_str().unwrap(),
    ))
    .unwrap();
    let context = egui::Context::default();
    let mut probe = Probe {
        snapshot: Some(room.dispatch(br#"{"kind":"snapshot"}"#).unwrap()),
        worker: Some(RequestWorker::new(room.clone(), context).unwrap()),
        ..Default::default()
    };

    begin(&mut probe, Action::Draft, true);
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        probe.poll();
        if probe.error.is_some()
            || (probe.pending.is_none()
                && probe
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot["proposal"]["phase"] == "ready"))
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let persisted = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(persisted["proposal"]["phase"], "ready", "{persisted:?}");
    assert_eq!(probe.workspace_step, 2);
    assert!(probe.error.is_none(), "{:?}", probe.error);
    assert_eq!(
        probe.snapshot.as_ref().unwrap()["proposal"]["phase"],
        "ready",
        "{:?}",
        probe.snapshot
    );
    assert_eq!(
        probe.snapshot.as_ref().unwrap()["proposal"]["plan"]["tasks"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert!(probe.snapshot.as_ref().unwrap()["job"].is_null());
    drop(probe);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
}
