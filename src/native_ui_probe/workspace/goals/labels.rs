use serde_json::Value;

pub(super) fn goal_stage(goal: &Value, language: &str) -> String {
    let stage = goal["progress"]["stage"].as_str().unwrap_or_default();
    let key = match stage {
        "no_plan" => "startup.no_plan",
        "analysis" => "startup.analysis",
        "planned" => "startup.planned_9b0f1b",
        "phase" => "startup.in_progress",
        "completed" => "startup.completed",
        _ => return stage.to_owned(),
    };
    crate::ui::messages::in_language(language, key).to_owned()
}

pub(super) fn phase_action_label(language: &str, number: usize) -> String {
    crate::ui::messages::interpolate(
        crate::ui::messages::in_language(language, "startup.plan_phase"),
        &[("value1", &number.to_string())],
    )
}
