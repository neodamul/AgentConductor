#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::navigation::{
    NavigationContext, State, show_in_language,
};
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn running_plan_exposes_confirmed_stop_and_delete_for_the_current_app_workspace() {
    let mut state = State::default();
    state.open();
    state.set_workspaces(vec![
        json!({"name":"Blocked","path":"/work/current","registered":true}),
    ]);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_in_language(
            &mut state,
            NavigationContext {
                current_name: "Blocked",
                current_path: "/work/current",
                can_change: false,
                planning_running: true,
                dirty: false,
                show_cache: false,
                language: "en",
            },
            ui,
        );
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = context.run_ui(Default::default(), |ui| {
        show_in_language(
            &mut state,
            NavigationContext {
                current_name: "Blocked",
                current_path: "/work/current",
                can_change: false,
                planning_running: true,
                dirty: false,
                show_cache: false,
                language: "en",
            },
            ui,
        );
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
    assert!(labels.contains(&"Stop planning and delete"), "{labels:?}");
}

#[test]
fn a_stopped_plan_is_not_archived_until_job_and_metadata_are_idle() {
    use agentconductor::native_ui_probe::workspace::navigation::stop::can_archive_after_stop;

    assert!(!can_archive_after_stop(&json!({
        "job":{"kind":"plan","state":"running"},
        "metadata_queue":[],"metadata_job":{"state":"idle"}
    })));
    assert!(!can_archive_after_stop(&json!({
        "job":{"kind":"plan","state":"stopped"},
        "metadata_queue":[{"kind":"quota"}],"metadata_job":{"state":"idle"}
    })));
    assert!(!can_archive_after_stop(&json!({
        "job":{"kind":"plan","state":"stopped"},
        "metadata_queue":[],"metadata_job":{"state":"running"}
    })));
    assert!(can_archive_after_stop(&json!({
        "job":{"kind":"plan","state":"stopped"},
        "metadata_queue":[],"metadata_job":{"state":"idle"}
    })));
}
