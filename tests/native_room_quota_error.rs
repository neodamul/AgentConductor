#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room;
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

const ERROR: &str = "Claude /usage returned an unsupported response";

#[test]
fn quota_error_is_visible_on_the_run_screen_in_both_languages() {
    for (language, prefix) in [("en", "Quota unavailable"), ("ko", "사용량 없음")] {
        let snapshot = json!({
            "report":{
                "state":"running",
                "plan":{"goal":"Quota diagnostics","tasks":[]},
                "tasks":[]
            },
            "quotas":{"claude":{"error":ERROR}},
            "job":{},
            "tool_requests":[]
        });
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut state = room::State::default();
        let mut output = context.run_ui(Default::default(), |ui| {
            room::show(&mut state, Some(&snapshot), false, None, language, ui);
        });
        let labels = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Label)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(
            labels
                .iter()
                .any(|label| { label.contains(prefix) && label.contains(ERROR) })
        );
    }
}
