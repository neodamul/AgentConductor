use super::*;

use crate::native_ui_probe::worker_test_support as worker_support;

#[test]
fn native_renderer_matches_platform_backend() {
    #[cfg(windows)]
    assert!(matches!(renderer(), eframe::Renderer::Wgpu));
    #[cfg(not(windows))]
    assert!(matches!(renderer(), eframe::Renderer::Glow));
}

#[test]
fn event_snapshot_clears_completed_metadata_from_every_workspace_step() {
    use std::time::{Duration, Instant};
    for step in 0..3 {
        let root = std::env::temp_dir().join(format!(
            "ac-metadata-poll-{}",
            crate::native_guard::session_id().unwrap()
        ));
        let room =
            Arc::new(crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap());
        let context = egui::Context::default();
        let mut probe = Probe::connect(room.clone(), context.clone()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while probe.snapshot.is_none() {
            probe.poll();
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        probe.workspace_step = step;
        probe.name = "Unsaved workspace name".into();
        probe.goal = "Unsaved goal".into();
        probe.snapshot.as_mut().unwrap()["metadata_job"] =
            serde_json::json!({"kind":"quota:claude", "state":"running"});
        probe.refresh_background();
        while probe.background_refresh_pending {
            probe.poll();
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(!workspace::profile_tools::metadata_running(
            probe.snapshot.as_ref().unwrap()
        ));
        assert_eq!(probe.name, "Unsaved workspace name");
        assert_eq!(probe.goal, "Unsaved goal");
        assert_eq!(probe.workspace_step, step);
        drop(probe);
        worker_support::close(room);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn foreground_action_does_not_disable_periodic_room_refresh() {
    let probe = Probe {
        pending: Some(crate::native_ui_probe::Pending::Action("Running action…")),
        room_view: Some(Default::default()),
        ..Probe::default()
    };

    assert!(!probe.room_refresh_blocked());
}
