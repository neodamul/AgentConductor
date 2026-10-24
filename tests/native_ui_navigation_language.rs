#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::navigation::{
    NavigationContext, State, show_in_language,
};
use agentconductor::ui::messages::{in_language, interpolate};
use eframe::egui::{self, accesskit::Role};

fn labels(state: &mut State, dirty: bool) -> (Vec<String>, Vec<String>) {
    labels_with_run(state, dirty, false)
}

fn labels_with_run(
    state: &mut State,
    dirty: bool,
    run_available: bool,
) -> (Vec<String>, Vec<String>) {
    state.set_header_state(
        run_available,
        if dirty {
            vec!["워크스페이스 설정".into(), "계획 작업".into()]
        } else {
            Vec::new()
        },
    );
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_in_language(
            state,
            NavigationContext {
                current_name: "Current",
                current_path: "/work/current",
                can_change: true,
                planning_running: false,
                dirty,
                show_cache: true,
                language: "ko",
            },
            ui,
        );
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = context.run_ui(Default::default(), |ui| {
        show_in_language(
            state,
            NavigationContext {
                current_name: "Current",
                current_path: "/work/current",
                can_change: true,
                planning_running: false,
                dirty,
                show_cache: true,
                language: "ko",
            },
            ui,
        );
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    let inputs = nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::TextInput)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (labels, inputs)
}

#[test]
fn workspace_menu_localizes_actions_and_the_new_workspace_input() {
    let mut state = State::default();
    state.open();
    state.set_workspaces(vec![
        serde_json::json!({"name":"Current","path":"/work/current","registered":true}),
        serde_json::json!({"name":"Research","path":"/work/research","registered":true}),
    ]);
    let (labels, inputs) = labels(&mut state, false);

    for label in [
        "저장된 워크스페이스",
        "워크스페이스 만들기",
        "현재",
        "워크스페이스 Research 열기",
        "Research 목록에서 제거",
        "워크스페이스 Research 보관",
    ] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "{label}: {labels:?}"
        );
    }
    assert_eq!(inputs, ["새 워크스페이스 이름"]);
}

#[test]
fn dirty_workspace_header_exposes_discard_changes_in_korean() {
    let mut state = State::default();
    let (labels, _) = labels(&mut state, true);
    assert!(
        labels.iter().any(|label| label == "변경 사항 버리기"),
        "dirty workspace header must expose Discard changes: {labels:?}"
    );
}

#[test]
fn back_to_run_is_only_shown_when_a_run_is_available() {
    let mut state = State::default();
    let (labels, _) = labels_with_run(&mut state, false, true);
    assert!(labels.iter().any(|label| label == "← 실행으로 돌아가기"));
}

#[test]
fn discard_confirmation_lists_unsaved_areas_and_preserves_saved_workspace() {
    let mut state = State::default();
    state.confirm_discard(vec!["워크스페이스 설정".into(), "계획 작업".into()], false);
    let (labels, _) = labels(&mut state, true);
    assert!(
        labels.iter().any(|label| {
            label.contains("워크스페이스 설정")
                && label.contains("계획 작업")
                && label.contains("저장된 워크스페이스는 그대로 유지됩니다")
        }),
        "{labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label == "저장하지 않은 변경 사항 버리기")
    );
    assert!(labels.iter().any(|label| label == "계속 편집"));
}

#[test]
fn archive_confirmation_is_localized_and_names_the_preserved_project_data() {
    let mut state = State::default();
    state.confirm_delete("/work/research", "Research");
    let (rendered, _) = labels(&mut state, false);
    for label in [
        "/work/research의 Research 워크스페이스를 보관할까요? 프로젝트 파일은 보존됩니다.",
        "Research 보관 확인",
        "Research 보관 취소",
    ] {
        assert!(
            rendered.iter().any(|actual| actual == label),
            "{label}: {rendered:?}"
        );
    }

    let (dirty_labels, _) = labels(&mut state, true);
    assert!(dirty_labels.iter().any(|label| {
        label.contains("저장하지 않은 변경 사항은 버려집니다")
            && label.contains("프로젝트 파일은 보존됩니다")
    }));
}

#[test]
fn create_and_open_confirmation_messages_warn_before_discarding_changes() {
    let create = interpolate(
        in_language("ko", "startup.confirm_create_discard_changes"),
        &[("value1", "Research")],
    );
    let open = interpolate(
        in_language("ko", "startup.confirm_open_discard_changes"),
        &[("value1", "Research"), ("value2", "/work/research")],
    );
    assert_eq!(
        create,
        "Research 워크스페이스를 만들고 저장하지 않은 변경 사항을 버릴까요?"
    );
    assert_eq!(
        open,
        "/work/research에 있는 Research 워크스페이스를 열까요? 저장하지 않은 변경 사항은 버려집니다."
    );
}
