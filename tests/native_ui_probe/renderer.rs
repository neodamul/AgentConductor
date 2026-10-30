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
fn metadata_completion_event_refreshes_every_workspace_step() {
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
        probe.updates.background_action(Instant::now(), false);
        probe.workspace_step = step;
        probe.name = "Unsaved workspace name".into();
        probe.goal = "Unsaved goal".into();
        probe.snapshot.as_mut().unwrap()["metadata_job"] =
            serde_json::json!({"kind":"quota:claude", "state":"running"});
        probe.workspace_next_refresh = Some(Instant::now() + Duration::from_secs(60));
        room.publish_event(crate::ui::events::UiEvent::invalidated(
            crate::ui::events::UiSurface::Workspace,
        ));
        while probe.poll_pending.is_none() && Instant::now() < deadline {
            probe.poll();
            std::thread::yield_now();
        }
        assert!(
            matches!(
                probe.poll_pending,
                Some(super::super::Pending::MetadataRefresh)
            ),
            "step {step} must refresh the shared sidebar when its event arrives; pending={}, poll_pending={}",
            probe.pending.is_some(),
            probe.poll_pending.is_some()
        );
        while probe.pending.is_some() || probe.poll_pending.is_some() {
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
        probe.workspace_next_refresh = Some(Instant::now() + Duration::from_secs(60));
        let mut idle = context.run_ui(egui::RawInput::default(), |ui| probe.update_ui(ui));
        idle.textures_delta.set.clear();
        idle.textures_delta.free.clear();
        assert!(probe.pending.is_none(), "idle metadata must not poll");
        drop(probe);
        worker_support::close(room);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn running_workspace_does_not_poll_without_an_invalidation_event() {
    use std::time::{Duration, Instant};
    let root = std::env::temp_dir().join(format!(
        "ac-idle-event-refresh-{}",
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
    probe.updates.background_action(Instant::now(), false);
    probe.workspace_step = 2;
    probe.snapshot.as_mut().unwrap()["job"]["state"] = "running".into();
    probe.workspace_next_refresh = Some(Instant::now() - Duration::from_secs(1));

    let mut idle = context.run_ui(egui::RawInput::default(), |ui| probe.update_ui(ui));
    idle.textures_delta.set.clear();
    idle.textures_delta.free.clear();

    assert!(probe.poll_pending.is_none(), "idle UI must await an event");
    assert!(
        probe.pending.is_none(),
        "idle UI must not enqueue a snapshot"
    );
    drop(probe);
    worker_support::close(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coordinator_chat_event_requests_the_chat_result() {
    use std::time::{Duration, Instant};
    let root = std::env::temp_dir().join(format!(
        "ac-chat-event-refresh-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let room =
        Arc::new(crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap());
    let context = egui::Context::default();
    let mut probe = Probe::connect(room.clone(), context).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while probe.snapshot.is_none() {
        probe.poll();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    probe.coordinator_chat.started("chat-job".into());
    room.publish_event(crate::ui::events::UiEvent::invalidated(
        crate::ui::events::UiSurface::Workspace,
    ));
    while probe.poll_pending.is_none() && Instant::now() < deadline {
        probe.poll();
        std::thread::yield_now();
    }

    assert!(matches!(
        probe.poll_pending,
        Some(super::super::Pending::ChatRefresh)
    ));
    drop(probe);
    worker_support::close(room);
    std::fs::remove_dir_all(root).unwrap();
}
