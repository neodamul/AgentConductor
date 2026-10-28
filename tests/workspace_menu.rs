//! Menu contracts for named and saved workspaces.

const HTML: &str = include_str!("../src/ui/startup.html");

#[test]
fn name_field_sits_above_the_goal() {
    let name = HTML
        .find("id=\"start-name\"")
        .expect("workspace name field");
    assert!(name < HTML.find("id=\"start-goal\"").unwrap());
}

#[test]
fn menu_offers_new_and_open_existing_workspace() {
    let script = include_str!("../src/ui/startup.js");
    let en: serde_json::Value =
        serde_json::from_str(include_str!("../src/ui/messages/en.json")).unwrap();
    for (id, text) in [
        ("startup.new_workspace", "New workspace…"),
        (
            "startup.open_existing_workspace",
            "Open existing workspace…",
        ),
    ] {
        assert!(script.contains(&format!("acMessage(\"{id}\")")), "{id}");
        assert_eq!(en[id], text);
    }
    assert!(!script.contains("Open existing folder"));
    assert!(script.contains("kind:'saved_workspaces'") && script.contains("kind:'open_workspace'"));
    let open = &script[script.find("open.onclick=").unwrap()..];
    let open = &open[..open.find("q('saved-list')").unwrap()];
    assert!(!open.contains("pick_folder"));
    assert!(HTML.contains("<dialog id=\"start-open-workspace\""));
}

#[test]
fn new_workspace_reuses_models_observed_on_this_machine() {
    let script = include_str!("../src/ui/startup.js");
    let reload = &script[script.find("async function reloadWorkspace(").unwrap()..];
    let reload = &reload[..reload.find('\n').unwrap()];
    assert!(
        reload.contains("addedAgents"),
        "a switched workspace queues its agents once"
    );
}
