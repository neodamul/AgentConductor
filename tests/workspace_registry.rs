//! R-40 / DD-48 / QAS-32: named workspaces and an explicit registry. HOME is set once in
//! this test binary so the registry stays inside the fixture.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
fn save(
    room: &Room,
    name: &str,
    profiles: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let revision = call(room, serde_json::json!({"kind":"snapshot"}))?["revision"].clone();
    let config = serde_json::json!({"profiles":profiles,"repository":"","goal":"","coordinator":"mock","workspace_mode":"shared","name":name});
    call(
        room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    )
}

#[test]
fn name_field_sits_above_the_goal() {
    let name = HTML
        .find("id=\"start-name\"")
        .expect("workspace name field");
    assert!(name < HTML.find("id=\"start-goal\"").unwrap());
}

#[test]
fn workspaces_are_named_created_listed_and_removed_without_deleting_files() {
    let (root, plan) = support::setup("shared");
    let home = root.join("home");
    unsafe { std::env::set_var("HOME", &home) };
    let profiles = serde_json::to_value(&plan.agents).unwrap();
    let first = root.join("first");
    let room = Room::startup(&first, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    save(&room, "Payments team", &profiles).unwrap();
    assert!(save(&room, &"x".repeat(81), &profiles).is_err());
    assert!(save(&room, "bad\nname", &profiles).is_err());
    let snapshot = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(snapshot["config"]["name"], "Payments team");

    // A successfully reaped old worker PID may now belong to an unrelated process.
    let terminal = first.join("coordinator-ui-reused/process");
    std::fs::create_dir_all(&terminal).unwrap();
    let pid = std::process::id();
    std::fs::write(terminal.join("owner.json"), serde_json::to_vec(&serde_json::json!({"coordinator_pid":pid,"pid":pid,"process_group":pid,"phase":"terminal","exit_code":0,"stopped":false,"timed_out":false})).unwrap()).unwrap();
    let created = call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Client A"}),
    )
    .unwrap();
    let path = created["current"].as_str().unwrap().to_string();
    assert!(
        path.starts_with(
            home.canonicalize()
                .unwrap()
                .join(".agentconductor/workspaces/client-a")
                .to_str()
                .unwrap()
        )
    );
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["config"]["name"],
        "Client A"
    );
    let again = call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Client A"}),
    )
    .unwrap();
    assert_ne!(
        again["current"], created["current"],
        "a second workspace gets its own folder"
    );
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"create_workspace","name":""})
        )
        .is_err()
    );

    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    let names: Vec<_> = listed["registered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.contains(&"Payments team".to_string())
            && names.iter().filter(|n| *n == "Client A").count() == 2,
        "{names:?}"
    );

    let current = listed["current"].clone();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"forget_workspace","path":current})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"forget_workspace","path":path}),
    )
    .unwrap();
    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    assert!(
        !listed["registered"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["path"] == path.as_str())
    );
    assert!(
        Path::new(&path).join("startup.json").exists(),
        "removing from the list keeps files"
    );
    // Open existing workspace lists what is saved under ~/.agentconductor, removed ones too.
    let saved = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let saved = saved["workspaces"].as_array().unwrap().clone();
    let removed = saved
        .iter()
        .find(|w| w["path"] == path.as_str())
        .expect("removed one is still saved");
    assert_eq!(removed["name"], "Client A");
    assert_eq!(removed["registered"], false);
    assert!(
        !saved
            .iter()
            .any(|w| w["path"] == first.canonicalize().unwrap().to_str().unwrap()),
        "only the data directory is scanned"
    );
    std::fs::create_dir_all(home.join(".agentconductor/workspaces/empty-folder")).unwrap();
    let again = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    assert!(
        !again.to_string().contains("empty-folder"),
        "folders without settings are not workspaces"
    );
    assert!(call(&room, serde_json::json!({"kind":"open_workspace","path":home.join(".agentconductor/workspaces/empty-folder")})).is_err());
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"open_workspace","path":root.join("repo")})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"open_workspace","path":path}),
    )
    .unwrap();
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["config"]["name"],
        "Client A"
    );
    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    assert!(
        listed["registered"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["path"] == path.as_str()),
        "reopening re-registers"
    );
    // Deletion is exact-name confirmed and archives only idle app-owned data.
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":path,"name":"Wrong current name"})
        )
        .is_err(),
        "current workspace still requires exact-name confirmation"
    );
    call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Keep me"}),
    )
    .unwrap();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":path,"name":"Wrong name"})
        )
        .is_err()
    );
    assert!(Path::new(&path).join("startup.json").exists());
    let lease =
        agentconductor::leases::Lease::acquire(&Path::new(&path).join("ui-window.lock")).unwrap();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":path,"name":"Client A"})
        )
        .is_err(),
        "other window is protected"
    );
    drop(lease);
    let blocked = Path::new(&path).join("run-blocked");
    std::fs::create_dir(&blocked).unwrap();
    std::fs::write(blocked.join("report.json"), serde_json::to_vec(&serde_json::json!({"schema_version":1,"state":"running","repo":root.join("repo"),"base":"base","branch":"main","plan":plan,"tasks":[],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")})).unwrap()).unwrap();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":path,"name":"Client A"})
        )
        .is_err(),
        "running execution is protected"
    );
    std::fs::remove_dir_all(&blocked).unwrap();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":first,"name":"Payments team"})
        )
        .is_err(),
        "an arbitrary folder cannot be archived"
    );
    let deleted = call(
        &room,
        serde_json::json!({"kind":"delete_workspace","path":path,"name":"Client A"}),
    )
    .unwrap();
    assert!(!Path::new(&path).exists());
    assert!(
        Path::new(deleted["archived"].as_str().unwrap())
            .join("startup.json")
            .exists()
    );
    assert!(
        root.join("repo/base.txt").exists(),
        "project sources stay intact"
    );
    assert!(
        !call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap()["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["path"] == path)
    );
    // A new workspace reuses model lists already observed for the same CLI.
    let snapshot = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    let mut claude = snapshot["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "claude")
        .unwrap()
        .clone();
    claude["catalog"] = serde_json::json!({"models":["opus","sonnet"],"source":"fixture","queried_at":1,"cli_version":"1"});
    let config = serde_json::json!({"profiles":[claude],"repository":"","goal":"","coordinator":"claude","workspace_mode":"shared"});
    call(
        &room,
        serde_json::json!({"kind":"save","revision":snapshot["revision"],"config":config}),
    )
    .unwrap();
    call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Models reused"}),
    )
    .unwrap();
    // R-101 (supersedes R-66): a new workspace opens with the machine's master agent list,
    // including observed models; nothing is queried.
    let fresh = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    let agents = fresh["config"]["profiles"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["id"], "claude");
    assert_eq!(agents[0]["catalog"]["models"][1], "sonnet");
    let doomed = call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Abandon planning"}),
    )
    .unwrap()["current"]
        .clone();
    let removed = call(
        &room,
        serde_json::json!({"kind":"delete_workspace","path":doomed,"name":"Abandon planning"}),
    )
    .unwrap();
    assert_eq!(removed["deleted"], true);
    let fresh = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(fresh["config"]["goal"], "");
    assert!(fresh["proposal"].is_null());
    // User decision 2026-10-13: deleting the current workspace asks for no new one; the most
    // recently used other workspace opens, so several can be deleted in a row.
    assert_eq!(fresh["config"]["name"], "Models reused");
    assert!(!HTML.contains("start-delete-new-name"));
    // Deleting one after another never asks for a name; the last one leaves one empty workspace.
    // Deletes the current workspace under its listed name; returns how many were listed.
    let delete_current = || {
        let current = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap()["current"]
            .as_str()
            .map(|p| Path::new(p).canonicalize().unwrap())
            .unwrap();
        let listed = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
        let listed = listed["workspaces"].as_array().unwrap().clone();
        let entry = listed
            .iter()
            .find(|w| {
                Path::new(w["path"].as_str().unwrap())
                    .canonicalize()
                    .unwrap()
                    == current
            })
            .unwrap();
        call(
            &room,
            serde_json::json!({"kind":"delete_workspace","path":entry["path"],"name":entry["name"]}),
        )
        .unwrap();
        (current, listed.len())
    };
    let mut last = delete_current();
    while last.1 > 1 {
        last = delete_current();
    }
    let empty = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(empty["config"]["goal"], "");
    let now = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap()["current"].clone();
    assert_ne!(
        Path::new(now.as_str().unwrap()).canonicalize().unwrap(),
        last.0
    );
    // R-101: deleting a workspace keeps the master agent list.
    assert_eq!(fresh["config"]["profiles"][0]["id"], "claude");
    assert_ne!(
        call(&room, serde_json::json!({"kind":"workspaces"})).unwrap()["current"],
        doomed
    );
    assert!(root.join("repo/base.txt").exists());
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn menu_offers_new_and_open_existing_workspace() {
    let script = include_str!("../src/ui/startup.js");
    // Menu labels come from the message catalog by ID.
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
    // Saved workspaces come from a list, not a folder dialog; the picker is for the repository only.
    assert!(script.contains("kind:'saved_workspaces'") && script.contains("kind:'open_workspace'"));
    // Opening a saved workspace never uses the folder dialog (Browse/export/import may).
    let open = &script[script.find("open.onclick=").unwrap()..];
    let open = &open[..open.find("q('saved-list')").unwrap()];
    assert!(!open.contains("pick_folder"));
    assert!(HTML.contains("<dialog id=\"start-open-workspace\""));
}

#[test]
fn new_workspace_reuses_models_observed_on_this_machine() {
    let script = include_str!("../src/ui/startup.js");
    let reload = &script[script.find("async function reloadWorkspace(").unwrap()..];
    let reload = &reload[..reload.find("\n").unwrap()];
    assert!(
        reload.contains("addedAgents"),
        "a switched workspace queues its agents once"
    );
}
