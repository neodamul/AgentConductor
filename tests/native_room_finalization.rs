use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn approved_merge_stays_visible_as_a_busy_finalization_stage() {
    let snapshot = json!({
        "revision":"rev-2",
        "report":{
            "state":"approved",
            "candidate":{"commit":"abc123"},
            "plan":{"tasks":[{"id":"task-1"}]},
            "tasks":[{"id":"task-1","status":"completed"}]
        },
        "job":{"kind":"merge","state":"running"}
    });
    let mut state = State::default();
    state.evidence = Some(json!({"revision":"rev-2"}));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let update = output.platform_output.accesskit_update.as_ref().unwrap();
    let labels = update
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    let stage_visible = |stage| {
        labels
            .iter()
            .any(|label| *label == stage || label.strip_prefix("Current stage: ") == Some(stage))
    };
    let stages_visible =
        stage_visible("03 Verify") && stage_visible("04 Merge") && stage_visible("05 Workspace");
    let merge_action_disabled = update.nodes.iter().any(|(_, node)| {
        node.role() == Role::Button
            && node.label().is_some_and(|label| label.contains("Merge"))
            && node.is_disabled()
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        stages_visible,
        "finalization stages were incomplete: {labels:?}"
    );
    assert!(visible.contains("Merging approved changes"), "{visible}");
    assert!(
        visible.contains("Applying approved changes to workspace"),
        "{visible}"
    );
    assert!(
        merge_action_disabled,
        "merge action was not disabled: {labels:?}"
    );
}

#[test]
fn finalization_progress_moves_from_verify_through_merge_to_workspace() {
    for (state, job, current) in [
        ("waiting_approval", json!({"state":"idle"}), "03 Verify"),
        (
            "approved",
            json!({"kind":"merge","state":"running"}),
            "04 Merge",
        ),
        ("completed", json!({"state":"idle"}), "05 Workspace"),
    ] {
        let snapshot = json!({"report":{"state":state},"job":job});
        let mut app = State::default();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            room::show(&mut app, Some(&snapshot), false, None, "en", ui);
        });
        let labels = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        let current_visible = labels.iter().any(|label| {
            *label == current || label.strip_prefix("Current stage: ") == Some(current)
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(current_visible, "{state}: {labels:?}");
    }
}
