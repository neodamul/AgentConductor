use agentconductor::{native_ui_probe::workspace::show_role_rules, roles::RoleAgent};
use eframe::egui;
use std::collections::BTreeMap;

#[test]
fn native_role_controls_show_only_registered_agents_and_observed_models() {
    let profiles = vec![serde_json::json!({
        "id": "codex",
        "catalog": {"models": ["gpt-observed"]}
    })];
    let mut rules = BTreeMap::from([(
        "testing".into(),
        vec![RoleAgent {
            agent: "codex".into(),
            model: Some("gpt-observed".into()),
        }],
    )]);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_role_rules(&mut rules, &profiles, true, ui)
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.iter().any(|label| label == "testing agent"));
    assert!(labels.iter().any(|label| label == "testing model"));
}
