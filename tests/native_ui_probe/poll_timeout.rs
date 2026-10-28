use super::{Pending, Probe};
use eframe::egui;
use std::{
    fs,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn temp_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "agentconductor-poll-timeout-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn wait_for_reply(probe: &mut Probe) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while probe.pending.is_some() {
        probe.poll();
        assert!(Instant::now() < deadline, "native snapshot reply timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn initial_snapshot_errors_stay_visible_and_scheduled_room_snapshots_are_background() {
    let root = temp_root("startup");
    let room =
        Arc::new(crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap());
    let mut probe = Probe::connect_view(room.clone(), egui::Context::default(), false).unwrap();
    assert!(matches!(probe.pending, Some(Pending::Refresh)));
    wait_for_reply(&mut probe);
    let mut initial_failure = Probe {
        pending: Some(Pending::Refresh),
        ..Probe::default()
    };
    initial_failure.handle_error("initial snapshot failed");
    assert_eq!(
        initial_failure.error.as_deref(),
        Some("initial snapshot failed")
    );
    drop(probe);
    crate::native_ui_probe::worker_test_support::close(room);
    fs::remove_dir_all(root).unwrap();

    let root = temp_root("room");
    let room =
        Arc::new(crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap());
    let mut probe = Probe::connect_view(room.clone(), egui::Context::default(), true).unwrap();
    assert!(matches!(probe.pending, Some(Pending::Refresh)));
    wait_for_reply(&mut probe);
    probe.error = None;
    probe.refresh_background();
    assert!(matches!(probe.pending, Some(Pending::BackgroundRefresh)));
    wait_for_reply(&mut probe);
    assert!(probe.error.is_none());
    drop(probe);
    crate::native_ui_probe::worker_test_support::close(room);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn background_poll_errors_are_silent_but_explicit_errors_remain_visible() {
    let mut background = Probe {
        pending: Some(Pending::BackgroundRefresh),
        ..Probe::default()
    };
    background.handle_error("background snapshot failed");
    assert!(background.error.is_none());
    assert!(background.pending.is_none());

    let mut explicit = Probe {
        pending: Some(Pending::Refresh),
        ..Probe::default()
    };
    explicit.handle_error("manual refresh failed");
    assert_eq!(explicit.error.as_deref(), Some("manual refresh failed"));
    assert!(explicit.pending.is_none());

    let mut action = Probe {
        pending: Some(Pending::Action),
        ..Probe::default()
    };
    action.handle_error("action failed");
    assert_eq!(action.error.as_deref(), Some("action failed"));
    assert!(action.pending.is_none());
}

#[test]
fn stale_background_task_details_are_discarded_quietly_but_explicit_mismatches_show_errors() {
    let response = serde_json::json!({"task":{"id":"stale-task"}});
    let mut stale = Probe {
        pending: Some(Pending::BackgroundTaskDetails),
        room_view: Some(crate::native_ui_probe::room::State::default()),
        ..Probe::default()
    };
    stale.room_view.as_mut().unwrap().details_task = Some("current-task".into());
    crate::native_ui_probe::poll::task_details::apply(&mut stale, response.clone());
    assert!(stale.pending.is_none());
    assert!(stale.error.is_none());
    assert!(stale.room_view.unwrap().task_details.is_none());

    let mut closed = Probe {
        pending: Some(Pending::BackgroundTaskDetails),
        ..Probe::default()
    };
    crate::native_ui_probe::poll::task_details::apply(&mut closed, response.clone());
    assert!(closed.pending.is_none());
    assert!(closed.error.is_none());

    let mut explicit = Probe {
        pending: Some(Pending::TaskDetails),
        room_view: Some(crate::native_ui_probe::room::State::default()),
        ..Probe::default()
    };
    explicit.room_view.as_mut().unwrap().details_task = Some("current-task".into());
    crate::native_ui_probe::poll::task_details::apply(&mut explicit, response);
    assert!(explicit.pending.is_none());
    assert_eq!(
        explicit.error.as_deref(),
        Some("Task activity response did not match the selection")
    );
}

#[test]
fn late_refresh_callback_restarts_deadline_without_a_catch_up_burst() {
    let interval = Duration::from_secs(5);
    let first = Instant::now();
    let mut next = Some(first + interval);

    let resumed = first + Duration::from_secs(60);
    assert!(crate::native_ui_probe::room::refresh_due_at(
        resumed, interval, &mut next
    ));
    assert_eq!(next, Some(resumed + interval));
    assert!(!crate::native_ui_probe::room::refresh_due_at(
        resumed, interval, &mut next
    ));
    assert_eq!(next, Some(resumed + interval));
}
