#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sharing::{State, show};
use eframe::egui;

fn labels(context: &egui::Context, state: &mut State, language: &str, dirty: bool) -> Vec<String> {
    let mut output = context.run_ui(Default::default(), |ui| {
        show(state, language, true, dirty, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    labels
}

#[test]
fn sharing_preview_and_confirmation_are_localized() {
    let mut state = State::default();
    state.set_folder("/share/team".into());
    state.set_preview(serde_json::json!({
        "settings":{"name":"Team","agents":[{"id":"claude"}],"coordinator":"claude"},
        "warnings":[]
    }));
    let context = egui::Context::default();
    context.enable_accesskit();
    let labels = labels(&context, &mut state, "ko", true);

    for expected in [
        "워크스페이스 설정 공유",
        "워크스페이스 설정 내보내기",
        "워크스페이스 설정 가져오기",
        "설정을 가져오거나 내보내려면 워크스페이스 변경사항을 저장하거나 취소하세요.",
        "/share/team에서 워크스페이스 설정 가져오기",
        "Team · 에이전트 1명 · 코디네이터 claude",
        "검증된 로컬 권한 설정 유지",
        "설정을 가져와 기존 설정 대체",
        "가져오기 취소",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "{expected}: {labels:?}"
        );
    }
}

#[test]
fn import_and_export_results_follow_the_selected_language() {
    let context = egui::Context::default();
    context.enable_accesskit();

    let mut imported = State::default();
    imported.imported();
    assert!(
        labels(&context, &mut imported, "ko", false)
            .contains(&"워크스페이스 설정을 가져왔습니다".into())
    );
    assert!(
        labels(&context, &mut imported, "en", false)
            .contains(&"Workspace settings imported".into())
    );

    let mut exported = State::default();
    exported.exported(&serde_json::json!({"exported":"/share/team"}));
    assert!(
        labels(&context, &mut exported, "ko", false)
            .contains(&"/share/team에 설정을 내보냈습니다".into())
    );
    assert!(
        labels(&context, &mut exported, "en", false).contains(&"Exported to /share/team".into())
    );
}
