use agentconductor::native_ui_probe::room::finalization::{Stage, can_start_merge, progress};
use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui;
use serde_json::json;

#[test]
fn review_merge_and_workspace_are_distinct_monotonic_stages() {
    let verify = progress(&json!({"report":{"state":"waiting_approval"}}));
    assert_eq!(verify.stage, Stage::Verify);
    assert!(!verify.merge_running);

    let merge = progress(&json!({
        "report":{"state":"approved"},
        "job":{"kind":"merge","state":"running"}
    }));
    assert_eq!(merge.stage, Stage::Merge);
    assert!(merge.merge_running);
    assert!(!can_start_merge(
        &json!({
            "report":{"state":"approved"},
            "job":{"kind":"merge","state":"running"}
        }),
        true,
        false
    ));

    let workspace = progress(&json!({"report":{"state":"completed"}}));
    assert_eq!(workspace.stage, Stage::Workspace);
}

#[test]
fn approved_candidate_is_merge_stage_even_before_job_starts() {
    let approved = progress(&json!({"report":{"state":"approved"}}));
    assert_eq!(approved.stage, Stage::Merge);
    assert!(!approved.merge_running);
    assert!(can_start_merge(
        &json!({"report":{"state":"approved"}}),
        true,
        false
    ));
}

#[test]
fn native_room_renders_localized_finalization_stages_and_merge_activity() {
    let snapshot = json!({
        "revision":"rev-2",
        "report":{"state":"approved","candidate":{"commit":"abc123"}},
        "job":{"kind":"merge","state":"running"}
    });
    let context = egui::Context::default();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("03 Verify") || text.contains("03 검증"));
    assert!(text.contains("04 Merge") || text.contains("04 병합"));
    assert!(text.contains("05 Workspace") || text.contains("05 워크스페이스"));
    assert!(text.contains("Merging approved changes") || text.contains("승인된 변경 병합 중"));
    assert!(text.contains("Merging approved changes") || text.contains("승인된 변경 병합 중"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
