#![cfg(unix)]
//! R-43 phase 1 / DD-79 / QAS-66: messenger settings are saved per workspace, travel with
//! the export and never carry the webhook URL. HOME is set once in this test binary.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const URL: &str = "https://hooks.example.test/services/T1/B1/very-secret";

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn settings_save_export_and_import_without_the_url() {
    let (root, plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let repo = root.join("repo").canonicalize().unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let a = Startup::open(&root.join("dev-a"), exe).unwrap();
    mutate(&a, serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":repo,"goal":"g","coordinator":"mock","workspace_mode":"shared","name":"Team"}})).unwrap();
    let initial = a.dispatch(br#"{"kind":"messenger"}"#).unwrap();
    assert_eq!(initial["settings"]["enabled"], false);
    assert_eq!(initial["has_url"], false);

    let settings = serde_json::json!({"enabled":true,"kind":"webhook","events":["completed","failed"],"template":"{\"body\":\"{{text}}\"}","include_goal":false});
    let bad = serde_json::json!({"enabled":true,"kind":"webhook","events":[],"template":"not json {{text}}","include_goal":false});
    assert!(
        mutate(
            &a,
            serde_json::json!({"kind":"save_messenger","settings":bad})
        )
        .is_err()
    );
    assert!(
        mutate(
            &a,
            serde_json::json!({"kind":"save_messenger","settings":settings,"url":"http://insecure"})
        )
        .is_err()
    );
    let saved = mutate(
        &a,
        serde_json::json!({"kind":"save_messenger","settings":settings,"url":URL}),
    )
    .unwrap();
    assert_eq!(saved["has_url"], true);
    let read = a.dispatch(br#"{"kind":"messenger"}"#).unwrap();
    assert_eq!(read["settings"], settings);
    assert!(
        !read.to_string().contains("very-secret"),
        "the URL is never returned"
    );
    // A plain workspace save keeps the messenger settings.
    let config = snapshot(&a)["config"].clone();
    let mut without = config.clone();
    without.as_object_mut().unwrap().remove("messenger");
    mutate(&a, serde_json::json!({"kind":"save","config":without})).unwrap();
    assert_eq!(snapshot(&a)["config"]["messenger"], settings);
    assert!(
        !std::fs::read_to_string(root.join("dev-a/startup.json"))
            .unwrap()
            .contains("very-secret")
    );

    mutate(
        &a,
        serde_json::json!({"kind":"export_settings","folder":repo}),
    )
    .unwrap();
    let text = std::fs::read_to_string(repo.join("agentconductor.workspace.json")).unwrap();
    assert!(!text.contains("very-secret") && !text.contains("hooks.example"));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&text).unwrap()["messenger"],
        settings
    );

    let b = Startup::open(&root.join("dev-b"), exe).unwrap();
    mutate(
        &b,
        serde_json::json!({"kind":"import_settings","folder":repo,"keep_verified":false}),
    )
    .unwrap();
    let imported = b.dispatch(br#"{"kind":"messenger"}"#).unwrap();
    assert_eq!(imported["settings"], settings);
    assert_eq!(
        imported["has_url"], false,
        "each machine enters its own URL"
    );

    // Removing the URL.
    mutate(
        &a,
        serde_json::json!({"kind":"save_messenger","settings":settings,"url":""}),
    )
    .unwrap();
    assert_eq!(
        a.dispatch(br#"{"kind":"messenger"}"#).unwrap()["has_url"],
        false
    );
    drop((a, b));
    std::fs::remove_dir_all(root).unwrap();
}
