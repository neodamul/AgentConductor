use agentconductor::native_ui_probe::room::task_activity::transcript_in_language;
use agentconductor::native_ui_probe::workspace::navigation::{
    NavigationContext, State, show_in_language,
};
use eframe::egui;

#[test]
fn workspace_header_uses_selected_language_without_probe_branding() {
    for (language, expected) in [("en", "Workspace"), ("ko", "워크스페이스")] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut state = State::default();
        let mut output = context.run_ui(Default::default(), |ui| {
            show_in_language(
                &mut state,
                NavigationContext {
                    current_name: "Demo",
                    current_path: "/tmp/demo",
                    can_change: true,
                    dirty: false,
                    show_cache: true,
                    language,
                },
                ui,
            );
        });
        let visible = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(visible.contains(expected), "{language}: {visible}");
        assert!(!visible.contains("NATIVE UI PROBE"), "{visible}");
    }
}

#[test]
fn synthesized_agent_activity_summaries_are_localized() {
    let cases = [
        (
            "codex",
            serde_json::json!({"type":"item.completed","item":{"type":"file_change","changes":[{},{}]}}),
            "파일 2개 변경",
            "2 files changed",
        ),
        (
            "codex",
            serde_json::json!({"type":"item.completed","item":{"type":"command_execution","exit_code":0}}),
            "종료 코드 0",
            "exit 0",
        ),
        (
            "claude",
            serde_json::json!({"type":"assistant","message":{"content":[{"type":"thinking","thinking":"one\ntwo\nthree\nfour\nfive"}]}}),
            "외 1줄",
            "1 more lines",
        ),
    ];
    for (provider, event, korean, english) in cases {
        let ko = transcript_in_language(provider, &event, "ko");
        let en = transcript_in_language(provider, &event, "en");
        assert!(ko.iter().any(|entry| entry.text.contains(korean)), "{ko:?}");
        assert!(
            en.iter().any(|entry| entry.text.contains(english)),
            "{en:?}"
        );
    }
}
