use agentconductor::native_ui_probe::workspace::navigation::{
    Event, State, needs_discard_confirmation, request,
};
use eframe::egui::{self, accesskit::Role};
use std::path::Path;

#[path = "support/workflow.rs"]
mod support;

#[test]
fn workspace_actions_use_the_existing_rust_dispatch_contract() {
    let cases = [
        (Event::List, serde_json::json!({"kind":"saved_workspaces"})),
        (
            Event::Create("Research".into()),
            serde_json::json!({"kind":"create_workspace","name":"Research"}),
        ),
        (
            Event::Open("/work/second".into()),
            serde_json::json!({"kind":"open_workspace","path":"/work/second"}),
        ),
        (
            Event::Forget("/work/old".into()),
            serde_json::json!({"kind":"forget_workspace","path":"/work/old"}),
        ),
        (
            Event::Delete {
                path: "/work/old".into(),
                name: "Old workspace".into(),
            },
            serde_json::json!({
                "kind":"delete_workspace",
                "path":"/work/old",
                "name":"Old workspace",
                "replacement":null
            }),
        ),
    ];
    for (event, expected) in cases {
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&request(&event)).unwrap(),
            expected
        );
    }
}

#[test]
fn workspace_sharing_requests_keep_revision_and_permission_confirmation_explicit() {
    use agentconductor::native_ui_probe::workspace::sharing::{
        export_request, import_request, preview_request,
    };

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&export_request("/share", "r1")).unwrap(),
        serde_json::json!({"kind":"export_settings","folder":"/share","revision":"r1"})
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&preview_request("/share")).unwrap(),
        serde_json::json!({"kind":"preview_settings","folder":"/share"})
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&import_request("/share", "r2", false))
            .unwrap(),
        serde_json::json!({"kind":"import_settings","folder":"/share","revision":"r2","keep_verified":false})
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&import_request("/share", "r2", true)).unwrap()
            ["keep_verified"],
        true
    );
}

#[test]
fn workspace_sharing_preview_discloses_warnings_before_the_import_action() {
    use agentconductor::native_ui_probe::workspace::sharing::{State, show};

    let mut state = State::default();
    state.set_folder("/share/team".into());
    state.set_preview(serde_json::json!({
        "settings":{"name":"Team","agents":[{"id":"claude"}],"coordinator":"claude"},
        "warnings":["claude: executable may not exist on this machine"]
    }));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show(&mut state, true, true, ui);
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"Import workspace settings from /share/team"));
    assert!(labels.contains(&"Import and replace settings"));
    assert!(labels.contains(&"Cancel import"));
    assert!(labels.contains(&"Keep verified local permission claims"));
    assert!(labels.contains(&"claude: executable may not exist on this machine"));
    assert!(
        labels
            .contains(&"Save or discard workspace changes before importing or exporting settings")
    );
}

#[test]
fn only_switching_or_creation_requires_confirmation_for_unsaved_edits() {
    assert!(needs_discard_confirmation(
        &Event::Create("Next".into()),
        true
    ));
    assert!(needs_discard_confirmation(
        &Event::Open("/work/next".into()),
        true
    ));
    assert!(!needs_discard_confirmation(&Event::List, true));
    assert!(!needs_discard_confirmation(
        &Event::Forget("/work/old".into()),
        true
    ));
    assert!(!needs_discard_confirmation(
        &Event::Create("Next".into()),
        false
    ));
}

#[test]
fn workspace_menu_exposes_accessible_open_forget_and_delete_actions() {
    let mut state = State::default();
    state.open();
    state.set_workspaces(vec![
        serde_json::json!({
            "name":"Current",
            "path":"/work/current",
            "registered":true
        }),
        serde_json::json!({
            "name":"Research",
            "path":"/work/research",
            "registered":true
        }),
    ]);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::navigation::show(
            &mut state,
            "Current",
            "/work/current",
            true,
            false,
            true,
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
    for action in [
        "Preview and clear local cache",
        "Create workspace",
        "Open workspace Research",
        "Remove Research from list",
        "Archive workspace Research",
    ] {
        assert!(labels.contains(&action), "{action}");
    }
}

#[test]
fn archive_confirmation_names_the_exact_workspace_and_preserves_project_data() {
    let mut state = State::default();
    state.confirm_delete("/work/research", "Research");
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::navigation::show(
            &mut state,
            "Current",
            "/work/current",
            true,
            false,
            false,
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
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"Confirm archive Research"));
    assert!(labels.contains(&"Cancel archive Research"));
    assert!(labels.contains(
        &"Archive workspace Research at /work/research; project source files are preserved"
    ));
}

#[test]
fn navigation_requests_create_reopen_forget_and_archive_through_room_dispatch() {
    let (root, _) = support::setup("shared");
    let home = root.join("home");
    unsafe { std::env::set_var("HOME", &home) };
    let source = root.join("repo/base.txt");
    let room_path = home.join(".agentconductor/desktop");
    let room = agentconductor::ui::Room::startup(
        &room_path,
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let mut snapshot: serde_json::Value = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    snapshot["config"]["name"] = "Current".into();
    snapshot["config"]["repository"] = root.join("repo").to_string_lossy().into_owned().into();
    room.dispatch(
        &serde_json::json!({
            "kind":"save",
            "revision":snapshot["revision"],
            "config":snapshot["config"]
        })
        .to_string()
        .into_bytes(),
    )
    .unwrap();
    let call = |event: Event| {
        room.dispatch(&request(&event))
            .unwrap_or_else(|error| panic!("{event:?}: {error}"))
    };
    let initial = call(Event::List);
    let first = initial["workspaces"][0]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(Path::new(&first).is_absolute());

    let created = call(Event::Create("Research".into()));
    let research = created["current"].as_str().unwrap().to_owned();
    assert!(Path::new(&research).join("startup.json").is_file());
    let opened = call(Event::Open(first.clone()));
    assert!(opened["switched"].as_bool().unwrap());

    assert_eq!(call(Event::Forget(research.clone()))["removed"], true);
    let forgotten = call(Event::List);
    assert!(
        forgotten["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["path"] == research && item["registered"] == false })
    );
    assert!(Path::new(&research).join("startup.json").is_file());

    let opened_research = call(Event::Open(research.clone()));
    assert!(opened_research["switched"].as_bool().unwrap());
    let reopened = call(Event::List);
    assert!(
        reopened["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["path"] == research && item["registered"] == true })
    );
    let archived = call(Event::Delete {
        path: research.clone(),
        name: "Research".into(),
    });
    assert_eq!(archived["deleted"], true);
    assert!(!Path::new(&research).exists());
    assert!(Path::new(&first).is_dir());
    assert!(source.is_file(), "archive must preserve project files");
}
