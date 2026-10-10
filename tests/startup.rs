#![cfg(unix)]
use agentconductor::ui::setup::Startup;
#[path = "support/workflow.rs"]
mod support;
#[test]
fn registration_is_persisted_redacted_and_revision_bound() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("desktop");
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&dir, exe).unwrap();
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut profiles = plan.agents;
    profiles[0]
        .environment
        .insert("PRIVATE_CONTEXT".into(), "private-fixture-value".into());
    let config = serde_json::json!({"profiles":profiles,"repository":root.join("repo"),"goal":"Implement three files","coordinator":"mock","workspace_mode":"shared"});
    let save = serde_json::json!({"kind":"save","revision":before["revision"],"config":config});
    startup.dispatch(save.to_string().as_bytes()).unwrap();
    assert!(startup.dispatch(save.to_string().as_bytes()).is_err());
    let after = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(!after.to_string().contains("private-fixture-value"));
    assert_eq!(after["config"]["goal"], "Implement three files");
    let mut invalid = config.clone();
    invalid["profiles"][0]["executable"] = serde_json::json!("bash");
    let save = serde_json::json!({"kind":"save","revision":after["revision"],"config":invalid});
    assert!(startup.dispatch(save.to_string().as_bytes()).is_err());
    assert_eq!(
        startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"],
        after["revision"]
    );
    drop(startup);
    let reopened = Startup::open(&dir, exe).unwrap();
    assert_eq!(
        reopened.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["config"]["goal"],
        "Implement three files"
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}
fn wait(startup: &Startup) -> serde_json::Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let state = snapshot(startup);
        if state["job"]["state"] != "running" {
            return state;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
#[test]
fn read_only_planning_is_reviewed_and_exact_consent_starts_only_registered_profiles() {
    let (root, plan) = support::setup("shared");
    let mut profiles = plan.agents;
    let mut planner = profiles[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan".into()];
    planner.permission_mode = "read_only".into();
    profiles.push(planner);
    let mut unused = profiles[0].clone();
    unused.id = "unused".into();
    unused.permission_mode = "unverified".into();
    unused.executable = "uninstalled-unused-cli".into();
    profiles.push(unused);
    let startup = Startup::open(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&startup,serde_json::json!({"kind":"save","config":{"profiles":profiles,"repository":root.join("repo"),"goal":"Create three scoped files","coordinator":"planner","workspace_mode":"worktree"}})).unwrap();
    mutate(&startup, serde_json::json!({"kind":"prepare"})).unwrap();
    let ready = wait(&startup);
    assert_eq!(ready["proposal"]["phase"], "ready");
    assert_eq!(
        ready["proposal"]["plan"]["tasks"].as_array().unwrap().len(),
        3
    );
    assert_eq!(ready["proposal"]["plan"]["workspace_mode"], "worktree");
    assert!(
        mutate(
            &startup,
            serde_json::json!({"kind":"start","digest":"not-approved"})
        )
        .is_err()
    );
    let review = std::path::PathBuf::from(ready["proposal"]["directory"].as_str().unwrap())
        .join("review.json");
    let original = std::fs::read(&review).unwrap();
    let mut changed: agentconductor::plan::Plan = serde_json::from_slice(&original).unwrap();
    changed.agents[0].executable = "unregistered-executable".into();
    std::fs::write(&review, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
    let altered = snapshot(&startup);
    assert!(
        mutate(
            &startup,
            serde_json::json!({"kind":"start","digest":altered["proposal"]["digest"]})
        )
        .is_err()
    );
    std::fs::write(&review, original).unwrap();
    let tasks = ready["proposal"]["plan"]["tasks"].clone();
    mutate(
        &startup,
        serde_json::json!({"kind":"edit","tasks":tasks,"workspace_mode":"shared"}),
    )
    .unwrap();
    assert!(
        mutate(
            &startup,
            serde_json::json!({"kind":"start","digest":ready["proposal"]["digest"]})
        )
        .is_err()
    );
    let reviewed = snapshot(&startup);
    let accepted = mutate(
        &startup,
        serde_json::json!({"kind":"start","digest":reviewed["proposal"]["digest"]}),
    )
    .unwrap();
    let completed = wait(&startup);
    assert_eq!(completed["job"]["state"], "completed");
    let report =
        agentconductor::workflow::load(std::path::Path::new(accepted["run"].as_str().unwrap()))
            .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.tests.len(), 3);
    assert_eq!(
        agentconductor::git::Repository::open(&root.join("repo"))
            .unwrap()
            .head()
            .unwrap(),
        report.base
    );
    drop(startup);
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::startup(&root.join("desktop"), exe).unwrap();
    assert!(
        room.dispatch(br#"{"kind":"open_run","path":"/untrusted"}"#)
            .is_err()
    );
    room.dispatch(br#"{"kind":"open_run"}"#).unwrap();
    let run = std::path::Path::new(accepted["run"].as_str().unwrap());
    assert!(agentconductor::ui::Room::open(run, exe).is_err());
    assert!(room.dispatch(br#"{"kind":"evidence"}"#).is_ok());
    room.dispatch(br#"{"kind":"home"}"#).unwrap();
    let second = agentconductor::ui::Room::open(run, exe).unwrap();
    assert!(room.dispatch(br#"{"kind":"open_run"}"#).is_err());
    drop(second);
    room.dispatch(br#"{"kind":"open_run"}"#).unwrap();
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn execution_rechecks_the_reviewed_repository_base_under_its_lease() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let base = agentconductor::git::Repository::open(&repo)
        .unwrap()
        .head()
        .unwrap();
    let changed = std::process::Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "commit",
            "--allow-empty",
            "-m",
            "Late repository change",
        ])
        .output()
        .unwrap();
    assert!(changed.status.success());
    let run = root.join("unexpected-run");
    assert!(
        agentconductor::workflow::run_reviewed(plan, &repo, &run, 5, &base)
            .await
            .err()
            .unwrap()
            .contains("repository base changed")
    );
    assert!(!run.exists());
    std::fs::remove_dir_all(root).unwrap();
}
