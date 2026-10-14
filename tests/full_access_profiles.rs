//! DD-131: full access belongs to the standard Claude and Codex executables only.
use agentconductor::ui::setup::Startup;
use serde_json::json;

fn save(startup: &Startup, executable: &str) -> serde_json::Value {
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut config = state["config"].clone();
    config["profiles"] = json!([{
        "id": "codex",
        "executable": executable,
        "args": ["exec", "-", "--json", "--dangerously-bypass-approvals-and-sandbox"],
        "parser": "codex",
        "permission_mode": "unverified"
    }]);
    config["coordinator"] = "codex".into();
    startup
        .dispatch(
            json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["config"]["profiles"][0].clone()
}

#[test]
fn only_the_codex_executable_keeps_full_access_and_another_falls_back_to_the_sandbox() {
    let root = std::env::temp_dir().join(format!(
        "ac-full-access-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();

    let codex = save(&startup, "codex");
    assert_eq!(codex["permission_mode"], "full_access");
    assert_eq!(
        codex["args"],
        json!([
            "exec",
            "-",
            "--json",
            "--dangerously-bypass-approvals-and-sandbox"
        ])
    );

    // A replaced executable (for example a test fixture) never inherits the bypass flags.
    let other = save(&startup, "quota-fixture");
    assert_eq!(other["permission_mode"], "operator_auto_approve");
    assert_eq!(
        other["args"],
        json!(["exec", "-", "--json", "--sandbox", "workspace-write"])
    );
    std::fs::remove_dir_all(root).unwrap();
}
