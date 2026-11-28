use agentconductor::{
    leases::Lease,
    ui::{Room, setup::Startup},
    workflow,
};
use serde_json::{Value, json};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

fn exe() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_agentconductor"))
}

fn busy_message() -> &'static str {
    agentconductor::ui::messages::text("native.repository_action_finishing")
}

#[test]
fn planning_while_the_project_is_locked_does_not_create_a_job_or_goal() {
    let (root, mut plan) = support::setup("shared");
    let repo = root.join("repo");
    plan.agents[0].permission_mode = "read_only".into();
    let startup = Startup::open(&root.join("workspace"), exe()).unwrap();
    let send = |value: Value| startup.dispatch(value.to_string().as_bytes());
    let initial = send(json!({"kind":"snapshot"})).unwrap();
    send(
        json!({"kind":"save","revision":initial["revision"],"config":{"name":"Fixture",
        "goal":"Build fixture","profiles":plan.agents,"coordinator":"mock","repository":repo}}),
    )
    .unwrap();

    let lease = Lease::repository(&repo).unwrap();
    let locked = send(json!({"kind":"snapshot"})).unwrap();
    assert_eq!(locked["repository_busy"], true);
    for action in ["prepare", "draft"] {
        assert_eq!(
            send(json!({"kind":action,"revision":locked["revision"]})).unwrap_err(),
            busy_message()
        );
    }
    let after = send(json!({"kind":"snapshot"})).unwrap();
    assert!(after["proposal"].is_null());
    assert!(after["job"].is_null());
    assert!(after["slugs"].as_array().unwrap().is_empty());
    drop(lease);
    let released = send(json!({"kind":"snapshot"})).unwrap();
    assert_eq!(released["repository_busy"], false);
    assert!(send(json!({"kind":"draft","revision":released["revision"]})).is_ok());
    let previous = send(json!({"kind":"snapshot"})).unwrap();
    let lease = Lease::repository(&repo).unwrap();
    assert_eq!(
        send(json!({"kind":"prepare","revision":previous["revision"]})).unwrap_err(),
        busy_message()
    );
    assert_eq!(
        send(json!({"kind":"snapshot"})).unwrap()["proposal"],
        previous["proposal"]
    );
    drop(lease);
}

#[tokio::test]
async fn room_refuses_approval_and_quota_while_the_coordinator_owns_the_run() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let dir = root.join("run");
    let report = workflow::run(plan, &repo, &dir, 30).await.unwrap();
    let candidate = report.candidate.as_ref().unwrap().commit.clone();
    let room = Room::open(&dir, exe()).unwrap();
    let send = |value: Value| room.dispatch(value.to_string().as_bytes());
    let lease = Lease::acquire(&dir.join("coordinator.lock")).unwrap();
    let locked = send(json!({"kind":"snapshot","room":true})).unwrap();
    assert_eq!(locked["coordinator_busy"], true);
    for action in [
        json!({"kind":"approve","sha":candidate}),
        json!({"kind":"quota","agent":"mock"}),
    ] {
        assert_eq!(
            send(
                json!({"kind":"action","room":true,"revision":locked["revision"],"action":action})
            )
            .unwrap_err(),
            busy_message()
        );
    }
    let blocked = std::process::Command::new(exe())
        .arg("approve")
        .arg(&dir)
        .arg(&candidate)
        .output()
        .unwrap();
    assert!(!blocked.status.success());
    let stderr = String::from_utf8(blocked.stderr).unwrap();
    assert!(stderr.contains(busy_message()), "{stderr}");
    assert!(!stderr.contains(agentconductor::leases::CONTENTION));
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    drop(lease);

    workflow::approve(&dir, &candidate).unwrap();
    let lease = Lease::repository(&repo).unwrap();
    let locked = send(json!({"kind":"snapshot","room":true})).unwrap();
    assert_eq!(locked["repository_busy"], true);
    assert_eq!(
        send(
            json!({"kind":"action","room":true,"revision":locked["revision"],
        "action":{"kind":"merge","sha":candidate}})
        )
        .unwrap_err(),
        busy_message()
    );
    drop(lease);
    assert_eq!(
        send(json!({"kind":"snapshot","room":true})).unwrap()["repository_busy"],
        false
    );
}

#[test]
fn reading_lock_state_does_not_create_or_rewrite_the_lock_file() {
    let (root, _) = support::setup("shared");
    let file = root.join("snapshot-only.lock");
    assert!(!Lease::held(&file).unwrap());
    assert!(!file.exists());
    std::fs::write(&file, b"unchanged").unwrap();
    assert!(!Lease::held(&file).unwrap());
    assert_eq!(std::fs::read(file).unwrap(), b"unchanged");
}

/// GitHub #1108: a busy probe holds the lock only for an instant and must not fail a real owner.
#[test]
fn a_brief_lock_holder_does_not_fail_acquire_but_a_lasting_one_does() {
    let path = std::env::temp_dir().join(format!("ac-1108-{}.lock", std::process::id()));
    let brief = Lease::acquire(&path).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(30));
        drop(brief);
    });
    let owner = Lease::acquire(&path).expect("a probe-length holder is waited out");
    release.join().unwrap();
    assert!(Lease::held(&path).unwrap());
    assert_eq!(
        Lease::acquire(&path).err().as_deref(),
        Some(agentconductor::leases::CONTENTION)
    );
    drop(owner);
    assert!(!Lease::held(&path).unwrap());
    let _ = std::fs::remove_file(path);
}
