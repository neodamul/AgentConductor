use crate::native_ui_probe::{Probe, worker_test_support};
use std::{sync::Arc, time::Instant};

#[test]
fn restoring_workspace_does_not_schedule_repeated_chat_refresh() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-event-only-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let room =
        Arc::new(crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap());
    let mut probe = Probe::connect(room.clone(), eframe::egui::Context::default()).unwrap();
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    while probe.snapshot.is_none() {
        probe.poll();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }

    probe.coordinator_chat.started("chat-job".into());
    let snapshot = probe.snapshot.clone().unwrap();
    super::workspace_ops::load_workspace(&mut probe, &snapshot);

    assert!(
        probe.workspace_next_refresh.is_none(),
        "an active chat must wait for its progress/completion event"
    );
    drop(probe);
    worker_test_support::close(room);
    std::fs::remove_dir_all(root).unwrap();
}
