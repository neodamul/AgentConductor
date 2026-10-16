#![cfg(unix)]
//! R-43 phase 1 / DD-79 / QAS-66: outbound messenger notifications. HOME and the curl
//! override are set once in this test binary.
use agentconductor::messenger::{self, Kind, Settings};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
#[path = "support/workflow.rs"]
mod support;

const URL: &str = "https://hooks.example.test/services/T000/B000/secret-token";

fn settings(kind: Kind) -> Settings {
    Settings {
        enabled: true,
        kind,
        events: messenger::EVENTS.iter().map(|e| e.to_string()).collect(),
        template: None,
        include_goal: false,
        ..Default::default()
    }
}

fn event(state: &str) -> messenger::Event {
    messenger::Event {
        state: state.into(),
        workspace: "Payments team".into(),
        goal: "Add \"export\"\nwith care".into(),
        completed: 3,
        total: 3,
    }
}

/// A fake curl that records its arguments and stdin, one numbered pair per call.
fn fake_curl(root: &Path) -> PathBuf {
    let out = root.join("sent");
    std::fs::create_dir_all(&out).unwrap();
    let script = root.join("fake-curl");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nn=$(ls {0} | wc -l | tr -d ' ')\necho \"$@\" > {0}/$n.args\ncat > {0}/$n.stdin\n",
            out.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    out
}

fn wait_for(dir: &Path, count: usize) -> Vec<String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "stdin"))
            .collect();
        files.sort();
        if files.len() >= count {
            std::thread::sleep(std::time::Duration::from_millis(100));
            return files
                .iter()
                .map(|p| std::fs::read_to_string(p).unwrap())
                .collect();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "only {} sent",
            files.len()
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn slack_and_company_payloads_are_valid_json_without_the_goal_by_default() {
    let slack = messenger::payload(&settings(Kind::Slack), &event("waiting_approval")).unwrap();
    let text = slack["text"].as_str().unwrap();
    assert!(
        text.contains("Payments team") && text.contains("3/3"),
        "{text}"
    );
    assert!(!text.contains("export"), "goal is opt-in");

    let mut company = settings(Kind::Webhook);
    company.include_goal = true;
    company.template =
        Some(r#"{"msg":"{{text}}","event":"{{event}}","ws":"{{workspace}}","goal":"{{goal}}","state":"{{state}}"}"#.into());
    let body = messenger::payload(&company, &event("completed")).unwrap();
    assert_eq!(body["event"], "completed");
    assert_eq!(body["ws"], "Payments team");
    assert_eq!(body["goal"], "Add \"export\"\nwith care");
    assert!(body["msg"].as_str().unwrap().contains("Add \"export\""));

    company.template = Some("{{text}}".into());
    assert!(
        messenger::payload(&company, &event("completed")).is_err(),
        "body must be a JSON object"
    );
    assert!(
        messenger::payload(&settings(Kind::Slack), &event("running")).is_err(),
        "not an event"
    );
}

#[test]
fn webhook_urls_are_https_local_secrets_and_runs_notify_once_per_state() {
    let (root, _plan) = support::setup("shared");
    unsafe {
        std::env::set_var("HOME", root.join("home"));
        std::env::set_var("AGENTCONDUCTOR_MESSENGER_CURL", root.join("fake-curl"));
    }
    let sent = fake_curl(&root);
    let workspace = root.join("desktop");
    std::fs::create_dir_all(&workspace).unwrap();
    assert!(messenger::save_secret(&workspace, Some("http://hooks.example.test/x")).is_err());
    messenger::save_secret(&workspace, Some(URL)).unwrap();
    let file = root.join("home/.agentconductor/messenger-secrets.json");
    assert_eq!(
        std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(messenger::secret(&workspace).as_deref(), Some(URL));

    let run = root.join("runs/run-1");
    let mut only_reviews = settings(Kind::Slack);
    only_reviews.events = vec!["waiting_approval".into(), "completed".into()];
    messenger::link_run(&run, &workspace, "Payments team", "goal", &only_reviews).unwrap();
    let link = std::fs::read_to_string(messenger::link_path(&run)).unwrap();
    assert!(
        !link.contains("secret-token"),
        "the link file holds no secret"
    );

    for state in [
        "running",
        "waiting_approval",
        "waiting_approval",
        "failed",
        "completed",
    ] {
        messenger::notify_run(&run, state, 3, 3);
    }
    let bodies = wait_for(&sent, 2);
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(
        std::fs::read_dir(&sent).unwrap().count(),
        4,
        "two sends: review and completed"
    );
    assert!(
        bodies[0].contains("url = \"https://hooks.example.test/services/T000/B000/secret-token\"")
    );
    assert!(bodies[0].contains("data-binary = \"{\\\"text\\\":"));
    let args = std::fs::read_to_string(sent.join("0.args")).unwrap();
    assert!(
        !args.contains("secret-token") && args.contains("--config -"),
        "{args}"
    );
    assert!(args.contains("--max-time 10") && args.contains("--fail"));

    // A run that is not linked, or a disabled workspace, sends nothing.
    messenger::notify_run(&root.join("runs/other"), "completed", 1, 1);
    let mut off = only_reviews.clone();
    off.enabled = false;
    messenger::link_run(&run, &workspace, "Payments team", "goal", &off).unwrap();
    messenger::notify_run(&run, "waiting_approval", 1, 3);
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_eq!(std::fs::read_dir(&sent).unwrap().count(), 4);

    messenger::save_secret(&workspace, None).unwrap();
    assert!(messenger::secret(&workspace).is_none());
    std::fs::remove_dir_all(root).unwrap();
}

/// Live check against a local HTTPS receiver: `LIVE_MESSENGER_URL=https://localhost:8443/hook
/// CURL_CA_BUNDLE=cert.pem cargo test --test messenger live -- --ignored`.
#[test]
#[ignore]
fn live_delivery_through_the_system_curl() {
    let url = std::env::var("LIVE_MESSENGER_URL").expect("LIVE_MESSENGER_URL");
    let mut company = settings(Kind::Webhook);
    company.template = Some(r#"{"text":"{{text}}","event":"{{event}}"}"#.into());
    let workspace = std::env::temp_dir();
    messenger::test(
        &workspace,
        &settings(Kind::Slack),
        Some(&url),
        "Live 테스트",
    )
    .unwrap();
    messenger::test(&workspace, &company, Some(&url), "Live \"quoted\"").unwrap();
    let refused = url.replace("8443", "9");
    assert!(messenger::test(&workspace, &company, Some(&refused), "x").is_err());
}
