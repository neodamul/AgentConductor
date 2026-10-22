use agentconductor::native_ui_probe::room;
use eframe::egui;
use serde_json::json;

#[test]
fn native_room_preserves_accessible_summary_activity_and_inspector_controls() {
    let snapshot = json!({
        "revision":"r1",
        "job":{"state":"completed"},
        "report":{
            "state":"completed",
            "plan":{
                "goal":"Verify the native Agent Room layout",
                "agents":[{"id":"local","parser":"custom"}],
                "tasks":[{"id":"task-1","agent":"local","prompt":"Inspect the task"}]
            },
            "tasks":[{"id":"task-1","status":"completed","session_id":"fixture-session"}],
            "candidate":null
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1200.0, 800.0),
        )),
        ..Default::default()
    };
    let mut state = room::State::default();
    let mut output = context.run_ui(input, |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let controls = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            let label = node.label().or_else(|| node.value())?;
            Some((label.to_owned(), node.bounds()?))
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let x = |label: &str| {
        controls
            .iter()
            .find_map(|(text, bounds)| (text == label).then_some(bounds.x0))
            .unwrap_or_else(|| panic!("missing {label:?}; {controls:?}"))
    };
    let y = |label: &str| {
        controls
            .iter()
            .find_map(|(text, bounds)| (text == label).then_some(bounds.y0))
            .unwrap_or_else(|| panic!("missing {label:?}; {controls:?}"))
    };
    assert!(
        x("Verify the native Agent Room layout") < x("Activity")
            && x("Activity") < x("Task inspector"),
        "summary, activity and inspector must render as three left-to-right columns"
    );
    assert!(
        y("CURRENT GOAL") < y("RUN STATUS")
            && y("CURRENT GOAL") < y("Verify the native Agent Room layout"),
        "the active goal must appear above the run summary and task columns"
    );
    let labels = controls
        .iter()
        .map(|(label, _)| label.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Run status: Completed"));
    assert!(labels.contains(&"Activity"));
    assert!(labels.contains(&"Task inspector"));
}

#[test]
fn native_room_review_evidence_and_decisions_fit_the_viewport() {
    let snapshot = json!({
        "revision":"r1",
        "report":{
            "state":"waiting_approval",
            "plan":{
                "goal":"Verify review layout",
                "agents":[{"id":"local","parser":"custom"}],
                "tasks":[{"id":"task-1","agent":"local","prompt":"Inspect"}]
            },
            "tasks":[{"id":"task-1","status":"completed"}],
            "candidate":{"commit":"abc123"}
        }
    });
    let mut state = room::State::default();
    state.evidence = Some(json!({
        "revision":"r1",
        "report":{"candidate":{"commit":"abc123"},"tests":[]},
        "test_output":[],
        "diff":"+verified change"
    }));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1140.0, 760.0),
            )),
            ..Default::default()
        },
        |ui| {
            room::show(&mut state, Some(&snapshot), false, None, "en", ui);
        },
    );
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let controls = nodes
        .iter()
        .filter_map(|(_, node)| {
            let label = node.label().or_else(|| node.value())?;
            let bounds = node.bounds()?;
            Some((label.to_owned(), bounds))
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let activity_x = controls
        .iter()
        .find_map(|(label, bounds)| (label == "Activity").then_some(bounds.x0))
        .unwrap_or_else(|| panic!("Activity missing from {controls:?}"));
    let inspector_x = controls
        .iter()
        .find_map(|(label, bounds)| (label == "Task inspector").then_some(bounds.x0))
        .unwrap_or_else(|| panic!("Task inspector missing from {controls:?}"));
    let activity_span = inspector_x - activity_x;
    assert!(
        activity_span >= 550.0,
        "the review column should retain the legacy center-column emphasis: {activity_span} px"
    );
    let inspector_width = controls
        .iter()
        .find_map(|(control, bounds)| {
            (control == "Merge approved candidate").then_some(bounds.x1 - bounds.x0)
        })
        .unwrap_or_else(|| panic!("Merge approved candidate missing from {controls:?}"));
    assert!(
        inspector_width <= 330.0,
        "the task inspector should not dominate the review layout: {inspector_width} px"
    );
    for label in [
        "Approve verified candidate",
        "Reject candidate",
        "Merge approved candidate",
    ] {
        let bounds = controls
            .iter()
            .find_map(|(control, bounds)| (control == label).then_some(*bounds))
            .unwrap_or_else(|| panic!("{label} missing from {controls:?}"));
        assert!(
            bounds.x1 <= 1140.0,
            "{label} is clipped outside the window: {bounds:?}"
        );
    }
}

#[test]
fn native_room_decisions_fit_the_minimum_window_content_width() {
    let snapshot = json!({
        "revision":"r1",
        "report":{
            "state":"waiting_approval",
            "plan":{
                "goal":"Review the minimum-width layout",
                "agents":[{"id":"local","parser":"custom"}],
                "tasks":[{"id":"task-1","agent":"local","prompt":"Inspect"}]
            },
            "tasks":[{"id":"task-1","status":"completed"}],
            "candidate":{"commit":"abc123"}
        }
    });
    let mut state = room::State::default();
    state.evidence = Some(json!({
        "revision":"r1",
        "report":{"candidate":{"commit":"abc123"},"tests":[]},
        "test_output":[],
        "diff":"+verified change"
    }));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(792.0, 532.0),
            )),
            ..Default::default()
        },
        |ui| {
            room::show(&mut state, Some(&snapshot), false, None, "en", ui);
        },
    );
    let controls = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            let label = node.label()?;
            let bounds = node.bounds()?;
            Some((label.to_owned(), bounds))
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for label in [
        "Approve verified candidate",
        "Reject candidate",
        "Merge approved candidate",
    ] {
        let bounds = controls
            .iter()
            .find_map(|(control, bounds)| (control == label).then_some(*bounds))
            .unwrap_or_else(|| panic!("{label} missing from {controls:?}"));
        assert!(
            bounds.x0 >= 0.0 && bounds.x1 <= 792.0,
            "{label} overflows the minimum content width: {bounds:?}"
        );
    }
}
