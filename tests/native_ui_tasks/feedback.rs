use super::{snapshot, task};
use agentconductor::native_ui_probe::workspace::{TaskDraft, task_controls};
use eframe::egui;

#[test]
fn dirty_plan_edit_highlights_save_with_theme_contrast() {
    let mut value = snapshot();
    value["proposal"]["plan"]["tasks"] = serde_json::json!([task()]);
    value["proposal"]["plan"]["workspace_mode"] = serde_json::json!("shared");
    let mut draft = TaskDraft::from_snapshot(&value);
    draft.as_mut().unwrap().tasks[0]["prompt"] = "Changed instructions".into();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut accent = egui::Color32::TRANSPARENT;
    let mut output = context.run_ui(Default::default(), |ui| {
        accent = ui.visuals().selection.bg_fill;
        let (_, dirty) = task_controls(&value, &mut draft, false, false, ui);
        assert!(dirty);
    });
    let has_accent = output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::epaint::Shape::Rect(rect) if rect.fill == accent),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    assert!(
        labels
            .iter()
            .any(|label| label == "Plan changes need review")
    );
    assert!(
        labels
            .iter()
            .any(|label| label == "Save changed plan and review again")
    );
    assert!(has_accent, "save button should use the active theme accent");
}

#[test]
fn saved_plan_points_to_the_exact_next_action_in_both_languages() {
    let mut value = snapshot();
    value["proposal"]["plan"]["tasks"] = serde_json::json!([task()]);
    value["proposal"]["plan"]["workspace_mode"] = serde_json::json!("shared");
    let expected = [
        (
            "en",
            "Plan saved. Next: review the updated tasks, then select ‘Review execution and approve’.",
        ),
        (
            "ko",
            "계획을 저장했습니다. 다음으로 작업을 검토한 뒤 ‘실행 검토 및 승인’을 선택하세요.",
        ),
    ];
    for (language, expected) in expected {
        let mut draft = TaskDraft::from_snapshot(&value);
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            let _ = agentconductor::native_ui_probe::workspace::task_controls_in_language(
                &value, &mut draft, false, true, language, ui,
            );
        });
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter_map(|(_, node)| node.label())
                .any(|label| label == expected)
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
}
