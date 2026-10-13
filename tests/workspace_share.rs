#![cfg(unix)]
//! R-52 / DD-59 / QAS-44: workspace settings exported for Git and imported safely.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const FILE: &str = "agentconductor.workspace.json";

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn export_is_safe_to_commit_and_import_reproduces_it_locally() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo").canonicalize().unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let a = Startup::open(&root.join("dev-a"), exe).unwrap();
    let mut native = serde_json::to_value(&plan.agents[0]).unwrap();
    native["id"] = "builder".into();
    native["environment"] = serde_json::json!({"PRIVATE_TOKEN":"secret-a"});
    native["catalog"] =
        serde_json::json!({"models":["m"],"source":"fixture","queried_at":1,"cli_version":"1"});
    let mut sandboxed = serde_json::to_value(&plan.agents[0]).unwrap();
    sandboxed["id"] = "tester".into();
    sandboxed["permission_mode"] = "external_sandbox".into();
    mutate(&a, serde_json::json!({"kind":"save","config":{"profiles":[native, sandboxed],"repository":repo,"goal":"local goal","coordinator":"builder","workspace_mode":"worktree","name":"Team","roles":{"testing":["tester"],"development":["builder"]}}})).unwrap();
    mutate(
        &a,
        serde_json::json!({"kind":"export_settings","folder":repo}),
    )
    .unwrap();
    let text = std::fs::read_to_string(repo.join(FILE)).unwrap();
    assert!(text.ends_with('\n') && !text.contains('\r'));
    assert!(
        !text.contains("secret-a") && !text.contains("PRIVATE_TOKEN"),
        "no secrets"
    );
    assert!(
        !text.contains("\"catalog\"") && !text.contains("local goal"),
        "no catalogs or goals"
    );
    assert!(
        !text.contains(repo.to_str().unwrap()),
        "no absolute repository path"
    );
    let file: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(file["format"], "agentconductor.workspace/v1");
    assert_eq!(file["repository"], ".");
    mutate(
        &a,
        serde_json::json!({"kind":"export_settings","folder":repo}),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.join(FILE)).unwrap(),
        text,
        "deterministic"
    );

    // Developer B already has a local "builder" with its own secret and catalog.
    let b = Startup::open(&root.join("dev-b"), exe).unwrap();
    let mut local = serde_json::to_value(&plan.agents[0]).unwrap();
    local["id"] = "builder".into();
    local["environment"] = serde_json::json!({"PRIVATE_TOKEN":"secret-b"});
    local["catalog"] = serde_json::json!({"models":["b-model"],"source":"fixture","queried_at":1,"cli_version":"1"});
    mutate(&b, serde_json::json!({"kind":"save","config":{"profiles":[local],"repository":"","goal":"b goal","coordinator":"builder","workspace_mode":"shared"}})).unwrap();
    let preview = b
        .dispatch(
            serde_json::json!({"kind":"preview_settings","folder":repo})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    assert!(
        preview["warnings"].to_string().contains("tester"),
        "{preview}"
    );
    assert!(b.dispatch(serde_json::json!({"kind":"import_settings","revision":"stale","folder":repo,"keep_verified":false}).to_string().as_bytes()).is_err());
    mutate(
        &b,
        serde_json::json!({"kind":"import_settings","folder":repo,"keep_verified":false}),
    )
    .unwrap();
    let config = snapshot(&b)["config"].clone();
    assert_eq!(config["name"], "Team");
    assert_eq!(config["coordinator"], "builder");
    assert_eq!(config["workspace_mode"], "worktree");
    assert_eq!(config["repository"], repo.to_str().unwrap());
    assert_eq!(config["goal"], "b goal", "goals stay local");
    assert_eq!(config["roles"]["testing"][0]["agent"], "tester");
    let builder = config["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "builder")
        .unwrap();
    assert_eq!(
        builder["catalog"]["models"][0], "b-model",
        "local catalog kept"
    );
    let tester = config["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "tester")
        .unwrap();
    assert_eq!(
        tester["permission_mode"], "unverified",
        "other machines' sandbox claims are not trusted"
    );
    let raw = std::fs::read_to_string(root.join("dev-b/startup.json")).unwrap();
    assert!(raw.contains("secret-b"), "local secret kept");

    mutate(
        &b,
        serde_json::json!({"kind":"import_settings","folder":repo,"keep_verified":true}),
    )
    .unwrap();
    let config = snapshot(&b)["config"].clone();
    let tester = config["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "tester")
        .unwrap();
    assert_eq!(
        tester["permission_mode"], "external_sandbox",
        "confirmed by the importing operator"
    );

    std::fs::write(repo.join(FILE), "{\"format\":\"other/v9\"}\n").unwrap();
    assert!(
        mutate(
            &b,
            serde_json::json!({"kind":"import_settings","folder":repo,"keep_verified":false})
        )
        .is_err()
    );
    drop((a, b));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_menu_offers_export_and_import_with_preview() {
    let script = include_str!("../src/ui/startup.js");
    let html = include_str!("../src/ui/startup.html");
    assert!(
        script.contains("\"startup.export_settings\"")
            && script.contains("\"startup.import_settings\"")
    );
    assert!(
        script.contains("kind:'preview_settings'") && script.contains("kind:'import_settings'")
    );
    assert!(
        html.contains("<dialog id=\"start-import\"")
            && html.contains("id=\"start-import-verified\"")
    );
}
