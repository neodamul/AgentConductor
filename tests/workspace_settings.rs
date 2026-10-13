#![cfg(unix)]
//! R-38/R-39, DD-46/DD-47, QAS-30/QAS-31: discoverable settings, folder picker and
//! in-place workspace switching. Environment variables are set once in this binary.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
fn exe() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_agentconductor"))
}

#[test]
fn settings_are_grouped_with_goal_first_and_repository_beside_worktrees() {
    let goal = HTML.find("id=\"start-goal\"").unwrap();
    let settings = HTML
        .find("startup.workspace_settings_5ebc59")
        .expect("visible settings heading");
    let coordinator = HTML.find("id=\"start-coordinator\"").unwrap();
    let repository = HTML.find("id=\"start-repository\"").unwrap();
    let browse = HTML
        .find("id=\"start-browse\"")
        .expect("folder picker button");
    let worktrees = HTML.find("id=\"start-worktrees\"").unwrap();
    // Step 1 (R-47 amendment) groups name, goal, coordinator and repository.
    assert!(settings < goal && goal < coordinator && coordinator < repository);
    assert!(repository < browse && browse < worktrees);
    // The path width applies to the path field only, never to the worktree checkbox.
    let css = include_str!("../src/ui/startup.css");
    assert!(
        !css.contains(".repository-row input{"),
        "row rule would size the checkbox"
    );
    // The coordinator selector keeps its original natural width (user request 2026-10-11).
    assert!(css.contains(".workspace-settings select{display:block;width:auto;min-width:210px}"));
    // Path, Browse and the worktree option share one row (user request 2026-10-11).
    let row = &HTML[HTML.find("class=\"repository-row\"").unwrap()..];
    let row = &row[..row.find("</div>").unwrap()];
    assert!(row.contains("id=\"start-browse\"") && row.contains("id=\"start-worktrees\""));
    let titlebar = &HTML[..HTML.find("</header>").unwrap()];
    assert!(
        titlebar.contains("id=\"start-workspace-menu\""),
        "workspace menu in title bar"
    );
}

#[test]
fn picker_saving_without_repository_and_switching_workspaces() {
    let (root, plan) = support::setup("shared");
    // Registry lives under HOME; picker answers come from the test override.
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let first = root.join("first");
    let second = root.join("second");
    let room = Room::startup(&first, exe()).unwrap();

    unsafe { std::env::set_var("AGENTCONDUCTOR_PICK_FOLDER", root.join("repo")) };
    let picked = call(&room, serde_json::json!({"kind":"pick_folder"})).unwrap();
    assert_eq!(picked["path"], root.join("repo").to_str().unwrap());
    unsafe { std::env::set_var("AGENTCONDUCTOR_PICK_FOLDER", "") };
    assert!(call(&room, serde_json::json!({"kind":"pick_folder"})).unwrap()["path"].is_null());

    unsafe { std::env::set_var("AGENTCONDUCTOR_PICK_EXECUTABLE", exe()) };
    assert_eq!(
        call(&room, serde_json::json!({"kind":"pick_executable"})).unwrap()["path"],
        exe().to_str().unwrap()
    );
    unsafe { std::env::set_var("AGENTCONDUCTOR_PICK_EXECUTABLE", "") };
    assert!(call(&room, serde_json::json!({"kind":"pick_executable"})).unwrap()["path"].is_null());
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"resolve_executable","executable":"no-such-ac-agent"})
        )
        .unwrap()["path"]
            .is_null()
    );
    let snapshot = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    let config = serde_json::json!({"profiles":plan.agents,"repository":"","goal":"No repository yet","coordinator":"mock","workspace_mode":"shared"});
    call(
        &room,
        serde_json::json!({"kind":"save","revision":snapshot["revision"],"config":config}),
    )
    .expect("a workspace saves without a repository");
    let snapshot = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"draft","revision":snapshot["revision"]})
        )
        .is_err()
    );

    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    assert_eq!(
        listed["current"],
        first.canonicalize().unwrap().to_str().unwrap()
    );
    call(
        &room,
        serde_json::json!({"kind":"switch_workspace","path":second}),
    )
    .unwrap();
    let now = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(
        now["root"],
        second.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(
        now["config"]["goal"], "",
        "the second workspace has its own settings"
    );
    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    let recent: Vec<_> = listed["registered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["path"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(recent[0], second.canonicalize().unwrap().to_str().unwrap());
    // Only workspaces the app manages are listed; this directly opened folder is not one.
    assert!(!recent.contains(&first.canonicalize().unwrap().to_str().unwrap().to_string()));

    // The first workspace's lease was released: another window may open it now,
    // and while it is open this window cannot switch to it.
    let other = Room::startup(&first, exe()).expect("released lease");
    let error = call(
        &room,
        serde_json::json!({"kind":"switch_workspace","path":first}),
    )
    .unwrap_err();
    assert!(error.contains("already open"), "{error}");
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["root"],
        second.canonicalize().unwrap().to_str().unwrap()
    );
    drop(other);
    call(
        &room,
        serde_json::json!({"kind":"switch_workspace","path":first}),
    )
    .unwrap();
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["config"]["goal"],
        "No repository yet"
    );
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"switch_workspace","path":"relative/dir"})
        )
        .is_err()
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
