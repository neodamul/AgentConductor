use super::*;
use serde_json::json;

#[test]
fn independent_deadlines_and_failures_do_not_starve_other_profiles() {
    let profiles = json!([
        {"id":"a","parser":"claude","quota_refresh_minutes":5},
        {"id":"b","parser":"codex","quota_refresh_minutes":1},
        {"id":"local","parser":"custom"}
    ]);
    let mut last = Default::default();
    assert_eq!(next(&profiles, &last, 1000).0.as_deref(), Some("a"));
    last.insert("a".into(), 1000);
    assert_eq!(next(&profiles, &last, 1000).0.as_deref(), Some("b"));
    last.insert("b".into(), 1000);
    assert_eq!(next(&profiles, &last, 1000), (None, Some(60_000)));
    assert_eq!(next(&profiles, &last, 61_000).0.as_deref(), Some("b"));
    last.insert("b".into(), 61_000);
    assert_eq!(next(&profiles, &last, 301_000).0.as_deref(), Some("a"));
    assert_eq!(next(&profiles, &last, 0).0.as_deref(), Some("a"));
}

#[test]
fn custom_queries_and_changed_intervals_use_existing_policy() {
    let profiles = json!([
        {"id":"local","parser":"custom","quota_command":["usage"],"quota_refresh_minutes":0}
    ]);
    let last = [("local".into(), 1000)].into_iter().collect();
    assert_eq!(next(&profiles, &last, 1000), (None, Some(300_000)));
    let mut edited = profiles.clone();
    edited[0]["quota_refresh_minutes"] = 1.into();
    assert_eq!(next(&edited, &last, 61_000).0.as_deref(), Some("local"));
    assert_eq!(next(&json!([]), &last, 1000), (None, None));
}

#[test]
fn automatic_refresh_defers_busy_hidden_and_unsaved_profiles() {
    let mut probe = Probe {
        snapshot: Some(json!({"config":{"profiles":[]}})),
        profiles: vec![json!({"id":"unsaved","parser":"claude"})],
        ..Default::default()
    };
    assert_eq!(probe.due_quota(1000, false), (None, None));
    probe.snapshot.as_mut().unwrap()["config"]["profiles"] =
        json!([{"id":"saved","parser":"claude"}]);
    assert_eq!(probe.due_quota(1000, false).0.as_deref(), Some("saved"));
    assert_eq!(probe.due_quota(1000, true), (None, None));
    for (field, value) in [
        ("job", json!({"state":"running"})),
        ("metadata_job", json!({"state":"running"})),
        ("metadata_queue", json!([{"kind":"quota","agent":"saved"}])),
        ("run_state", json!("running")),
    ] {
        probe.snapshot.as_mut().unwrap()[field] = value;
        assert_eq!(probe.due_quota(1000, false), (None, None), "{field}");
        probe.snapshot.as_mut().unwrap()[field] = Value::Null;
    }
    probe.pending = Some(Pending::Save);
    assert_eq!(probe.due_quota(1000, false), (None, None));
}

#[test]
fn native_worker_failure_is_rate_limited_and_preserves_drafts() {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    let root = std::env::temp_dir().join(format!(
        "ac-quota-schedule-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let room =
        Arc::new(crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap());
    let context = eframe::egui::Context::default();
    let mut probe = Probe::connect(room.clone(), context.clone()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while probe.snapshot.is_none() {
        probe.poll();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    // The controller deliberately has no such registered agent: no CLI can launch.
    probe.snapshot.as_mut().unwrap()["config"]["profiles"] =
        json!([{"id":"absent","parser":"claude"}]);
    probe.name = "Unsaved name".into();
    probe.goal = "Unsaved goal".into();
    probe.workspace_step = 2;
    probe.updates.background_action(Instant::now(), false);
    let mut frame = context.run_ui(Default::default(), |ui| probe.update_ui(ui));
    frame.textures_delta.set.clear();
    frame.textures_delta.free.clear();
    assert!(matches!(probe.pending, Some(Pending::Metadata)) || probe.error.is_some());
    let attempted = probe.quota_refresh_last["absent"];
    while probe.pending.is_some() {
        probe.poll();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(
        probe.error.is_some(),
        "the controller must reject the absent agent"
    );
    assert_eq!(probe.due_quota(attempted + 1, false), (None, Some(299_999)));
    assert_eq!(probe.name, "Unsaved name");
    assert_eq!(probe.goal, "Unsaved goal");
    assert_eq!(probe.workspace_step, 2);
    let mut output = context.run_ui(Default::default(), |ui| {
        probe.refresh_quotas(ui.ctx(), attempted + 1);
    });
    assert!(
        output.viewport_output[&eframe::egui::ViewportId::ROOT].repaint_delay
            <= Duration::from_millis(299_999)
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        probe.pending.is_none(),
        "failure must not immediately enqueue again"
    );
    probe.quota_refresh_last.insert("absent".into(), 1);
    probe.quota_refresh_last.insert("other".into(), 17);
    probe.request_metadata(workspace::profile_tools::Event::Quota("absent".into()));
    assert!(probe.quota_refresh_last["absent"] > 1);
    assert_eq!(probe.quota_refresh_last["other"], 17);
    while probe.pending.is_some() {
        probe.poll();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    probe.snapshot.as_mut().unwrap()["icons"] = json!({"claude":"retained-icon"});
    probe.refresh_metadata();
    while probe.pending.is_some() {
        probe.poll();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(
        probe.snapshot.as_ref().unwrap()["icons"]["claude"],
        "retained-icon"
    );
    assert_eq!(probe.name, "Unsaved name");
    drop(probe);
    crate::native_ui_probe::worker_test_support::close(room);
    std::fs::remove_dir_all(root).unwrap();
}
