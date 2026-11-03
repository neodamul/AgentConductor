#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{profile_tools, sidebar};
use eframe::egui::{self, accesskit::Role};
use serde_json::{Value, json};

const ERROR: &str = "Claude /usage returned an unsupported response";

fn labels(output: &mut egui::FullOutput) -> Vec<String> {
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Label)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    labels
}

fn snapshot() -> Value {
    json!({
        "config":{"profiles":[{"id":"claude","parser":"claude"}]},
        "quotas":{"claude":{"status":"error","error":ERROR}},
        "quota_badges":{"claude":{"text":"—","title":"Quota unavailable"}},
        "metadata_job":{}
    })
}

#[test]
fn quota_error_is_visible_in_the_agent_sidebar_in_both_languages() {
    for (language, prefix) in [("en", "Quota unavailable"), ("ko", "사용량 없음")] {
        let profiles = vec![json!({"id":"claude","parser":"claude"})];
        let snapshot = snapshot();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            sidebar::show(&profiles, &snapshot, language, true, false, ui);
        });
        assert!(
            labels(&mut output)
                .iter()
                .any(|label| label.contains(prefix) && label.contains(ERROR))
        );
    }
}

#[test]
fn quota_error_is_visible_in_agent_settings_in_both_languages() {
    for (language, prefix) in [("en", "Quota unavailable"), ("ko", "사용량 없음")] {
        let mut profiles = vec![json!({"id":"claude","parser":"claude"})];
        let snapshot = snapshot();
        let mut coordinator = String::new();
        let mut roles = agentconductor::roles::RoleRules::default();
        let mut editor = profile_tools::Editor::default();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            profile_tools::show_in_language(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &mut editor,
                profile_tools::RenderContext {
                    snapshot: &snapshot,
                    editable: true,
                    language,
                    show_registry: true,
                },
                ui,
            );
        });
        assert!(
            labels(&mut output)
                .iter()
                .any(|label| label.contains(prefix) && label.contains(ERROR))
        );
    }
}
