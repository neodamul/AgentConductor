use agentconductor::native_ui_probe::updates::{Action, State, request};
use eframe::egui::{self, accesskit::Role};

#[test]
fn update_actions_use_the_existing_rust_dispatcher() {
    for (action, kind) in [
        (Action::Status, "update_status"),
        (Action::Check, "check_update"),
        (Action::Apply, "apply_update"),
    ] {
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&request(action)).unwrap(),
            serde_json::json!({"kind":kind})
        );
    }
}

#[test]
fn update_panel_has_localized_actions_and_keeps_full_release_notes() {
    let status = serde_json::json!({
        "phase":"idle", "current":"0.1.24", "latest":"0.1.25",
        "newer":true, "installable":true,
        "page":"https://github.com/neodamul/AgentConductor/releases/tag/v0.1.25",
        "notes":"## Fixes\n\n- Fixed Windows paths.\n- Improved native accessibility."
    });
    for (language, check, apply, close, page) in [
        (
            "en",
            "Check for updates…",
            "Update and restart",
            "Close",
            "Release notes",
        ),
        (
            "ko",
            "업데이트 확인…",
            "업데이트 후 재시작",
            "닫기",
            "릴리스 노트",
        ),
    ] {
        let mut state = State::default();
        state.open = true;
        state.apply_status(status.clone());
        assert_eq!(state.status()["notes"], status["notes"]);
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            state.show(language, false, ui);
        });
        let labels = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Button)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for label in [check, apply, close] {
            assert!(labels.contains(&label), "{label}: {labels:?}");
        }
        assert!(
            labels.iter().any(|label| label.starts_with(page)),
            "{page}: {labels:?}"
        );
    }
}
