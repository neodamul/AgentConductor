#![cfg(unix)]
//! GitHub #72: a state the run re-enters after another state is a new event.
use agentconductor::messenger::{self, Kind, Settings};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

const URL: &str = "https://hooks.example.test/services/T000/B000/secret-token";

fn recorded(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "stdin"))
        .count()
}

fn wait_for(dir: &Path, count: usize) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while recorded(dir) < count {
        assert!(std::time::Instant::now() < deadline, "deliveries missing");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn a_state_entered_again_after_another_state_is_notified_again() {
    let (root, _plan) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    env.set("AGENTCONDUCTOR_MESSENGER_CURL", root.join("fake-curl"));
    let sent = root.join("sent");
    std::fs::create_dir_all(&sent).unwrap();
    let script = root.join("fake-curl");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nn=$(ls {0} | wc -l | tr -d ' ')\necho \"$@\" > {0}/$n.args\ncat > {0}/$n.stdin\n",
            sent.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let workspace = root.join("desktop");
    std::fs::create_dir_all(&workspace).unwrap();
    messenger::save_secret(&workspace, Some(URL)).unwrap();
    let run = root.join("runs/run-1");
    let settings = Settings {
        enabled: true,
        kind: Kind::Slack,
        events: vec!["failed".into(), "completed".into()],
        ..Default::default()
    };
    messenger::link_run(&run, &workspace, "Payments team", "goal", &settings).unwrap();

    // A retry passes through the non-notifiable "running" state before failing again.
    messenger::notify_run(&run, "failed", 1, 3);
    wait_for(&sent, 1);
    messenger::notify_run(&run, "failed", 1, 3);
    messenger::notify_run(&run, "running", 1, 3);
    messenger::notify_run(&run, "failed", 1, 3);
    wait_for(&sent, 2);
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(
        recorded(&sent),
        2,
        "a repeated save is silent, a re-entry is not"
    );
    std::fs::remove_dir_all(root).unwrap();
}
