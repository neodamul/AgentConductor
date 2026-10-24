#![cfg(unix)]
//! R-43 phase 2 / DD-81 / QAS-68: exact, authorized tool decisions from the messenger.
//! HOME and the curl override are set once in this test binary.
use agentconductor::gates::{Context, Ledger, ToolRequest};
use agentconductor::messenger::{self, Kind, Settings, inbound};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
#[path = "support/workflow.rs"]
mod support;

#[test]
fn codes_replies_and_signatures() {
    assert_eq!(inbound::code("3fa9c21bdeadbeef"), "3FA9C21B");
    assert_eq!(
        inbound::parse("allow 3fa9c21b"),
        Some((true, "3FA9C21B".into()))
    );
    assert_eq!(
        inbound::parse("<@U9> Deny `3FA9C21B`."),
        Some((false, "3FA9C21B".into()))
    );
    assert_eq!(inbound::parse("allow everything"), None);
    assert_eq!(inbound::parse("please allow 3fa9c21"), None);
    // RFC 4231 test case 2.
    assert_eq!(
        inbound::hmac(b"Jefe", b"what do ya want for nothing?"),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    let slack = r#"{"ok":true,"messages":[
      {"user":"U1","text":"deny aaaaaaaa","ts":"20.0"},
      {"bot_id":"B1","user":"U1","text":"allow aaaaaaaa","ts":"30.0"},
      {"subtype":"message_changed","user":"U1","text":"allow aaaaaaaa","ts":"31.0"},
      {"user":"U1","text":"allow aaaaaaaa","ts":"10.0"}]}"#;
    let replies = inbound::slack_replies(slack).unwrap();
    assert_eq!(replies.len(), 2, "bots and edits ignored");
    assert_eq!(replies[0].ts, "10.0", "oldest first");
    assert!(inbound::slack_replies(r#"{"ok":false,"error":"not_in_channel"}"#).is_err());

    let sign = |c: &str, d: &str, u: &str, t: &str| {
        inbound::hmac(b"relay-key", format!("{c}|{d}|{u}|{t}").as_bytes())
    };
    let body = serde_json::json!({"decisions":[
        {"code":"AAAAAAAA","decision":"allow","user":"kim","ts":"1","signature":sign("AAAAAAAA","allow","kim","1")},
        {"code":"AAAAAAAA","decision":"allow","user":"lee","ts":"2","signature":sign("AAAAAAAA","allow","kim","2")},
        {"code":"AAAAAAAA","decision":"maybe","user":"kim","ts":"3","signature":sign("AAAAAAAA","maybe","kim","3")}]});
    let (ok, refused) = inbound::relay_replies(&body.to_string(), "relay-key").unwrap();
    assert_eq!(ok.len(), 1);
    assert_eq!(ok[0].user, "kim");
    assert_eq!(refused.len(), 2);
}

fn fake_curl(root: &Path) -> PathBuf {
    let dir = root.join("curl");
    for sub in ["fetch", "post"] {
        std::fs::create_dir_all(dir.join(sub)).unwrap();
    }
    let script = root.join("fake-curl");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nd={0}\ncat > $d/last.stdin\nif grep -q 'conversations.history\\|relay.example' $d/last.stdin; then n=$(ls $d/fetch | wc -l | tr -d ' '); cp $d/last.stdin $d/fetch/$n.stdin; echo \"$@\" > $d/fetch/$n.args; cat $d/response.json; else n=$(ls $d/post | wc -l | tr -d ' '); cp $d/last.stdin $d/post/$n.stdin; fi\n",
            dir.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    unsafe { std::env::set_var("AGENTCONDUCTOR_MESSENGER_CURL", &script) };
    dir
}

fn count(dir: &Path) -> usize {
    std::fs::read_dir(dir).unwrap().count()
}

fn pending(run: &Path, task: &str) -> String {
    let mut ledger = Ledger::open(&run.join("state.sqlite")).unwrap();
    let context = Context {
        epoch: 1,
        task: task.into(),
        session: "s".into(),
        workspace: "w".into(),
        plan_sha256: "p".into(),
        workspace_sha256: "x".into(),
    };
    let request = ToolRequest {
        tool: "Bash".into(),
        input: serde_json::json!({"command":"rm -rf build"}),
        reason: format!("call-{task}"),
    };
    ledger.request(&context, &request, 1).unwrap()
}

#[test]
fn listed_approver_decides_once_and_nothing_is_fetched_without_pending_requests() {
    let (root, _plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let curl = fake_curl(&root);
    let workspace = root.join("desktop");
    std::fs::create_dir_all(&workspace).unwrap();
    messenger::save_secret(&workspace, Some("https://hooks.example.test/x")).unwrap();
    messenger::save_secret_value(&workspace, Some("token"), Some("xoxb-test-token")).unwrap();
    let run = root.join("runs/run-1");
    std::fs::create_dir_all(&run).unwrap();
    let settings = Settings {
        enabled: true,
        kind: Kind::Slack,
        decisions: true,
        slack_channel: Some("C1".into()),
        approvers: vec!["U1".into()],
        ..Default::default()
    };
    messenger::link_run(&run, &workspace, "Team", "goal", &settings).unwrap();
    let mut decided = vec![];
    let mut apply = |id: &str, allow: bool| {
        decided.push((id.to_string(), allow));
        Ok(())
    };

    // No pending request: no network at all.
    Ledger::open(&run.join("state.sqlite")).unwrap();
    let idle = inbound::poll_once(&run, &mut apply).unwrap();
    assert!(!idle.fetched);
    assert_eq!(count(&curl.join("fetch")) + count(&curl.join("post")), 0);

    let id = pending(&run, "build");
    let code = inbound::code(&id);
    std::fs::write(
        curl.join("response.json"),
        serde_json::json!({"ok":true,"messages":[
            {"user":"U2","text":format!("allow {code}"),"ts":"100.1"},
            {"user":"U1","text":"allow 00000000","ts":"100.2"},
            {"user":"U1","text":format!("allow {}", code.to_lowercase()),"ts":"100.3"}]})
        .to_string(),
    )
    .unwrap();
    let first = inbound::poll_once(&run, &mut apply).unwrap();
    assert_eq!(first.announced, 1);
    assert_eq!(first.applied, vec![(id.clone(), true, "U1".to_string())]);
    assert_eq!(first.ignored.len(), 2, "{:?}", first.ignored);
    let announce = std::fs::read_to_string(curl.join("post/0.stdin")).unwrap();
    assert!(announce.contains(&code) && announce.contains("build") && !announce.contains("rm -rf"));
    assert_eq!(
        count(&curl.join("post")),
        2,
        "announcement and confirmation"
    );
    let fetch = std::fs::read_to_string(curl.join("fetch/0.stdin")).unwrap();
    assert!(
        fetch.contains("Authorization: Bearer xoxb-test-token") && fetch.contains("channel=C1")
    );
    let args = std::fs::read_to_string(curl.join("fetch/0.args")).unwrap();
    assert!(!args.contains("xoxb") && args.contains("--config -"));

    // The same replies again: nothing applied, nothing re-announced.
    let again = inbound::poll_once(&run, &mut apply).unwrap();
    assert_eq!(again.announced, 0);
    assert!(again.applied.is_empty());
    assert!(
        std::fs::read_to_string(curl.join("fetch/2.stdin"))
            .unwrap()
            .contains("oldest=100.3")
    );
    assert_eq!(decided, vec![(id, true)]);
    std::fs::remove_dir_all(root).unwrap();
}

/// Live round trip through the system curl against a local HTTPS stand-in for Slack:
/// `LIVE_SLACK_DIR=<dir with reply.json> AGENTCONDUCTOR_SLACK_API=https://localhost:8443/api
/// CURL_CA_BUNDLE=cert.pem cargo test --test messenger_inbound live -- --ignored`.
#[test]
#[ignore]
fn live_round_trip_through_the_system_curl() {
    let dir = PathBuf::from(std::env::var("LIVE_SLACK_DIR").expect("LIVE_SLACK_DIR"));
    let root = dir.join("case");
    let _ = std::fs::remove_dir_all(&root);
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let workspace = root.join("desktop");
    std::fs::create_dir_all(&workspace).unwrap();
    messenger::save_secret(&workspace, Some("https://localhost:8443/hook")).unwrap();
    messenger::save_secret_value(&workspace, Some("token"), Some("xoxb-live")).unwrap();
    let run = root.join("run");
    std::fs::create_dir_all(&run).unwrap();
    let settings = Settings {
        enabled: true,
        decisions: true,
        slack_channel: Some("C1".into()),
        approvers: vec!["U1".into()],
        ..Default::default()
    };
    messenger::link_run(&run, &workspace, "Live", "goal", &settings).unwrap();
    let id = pending(&run, "deploy");
    std::fs::write(
        dir.join("reply.json"),
        serde_json::json!({"ok":true,"messages":[{"user":"U1","text":format!("deny {}", inbound::code(&id)),"ts":"5.5"}]}).to_string(),
    )
    .unwrap();
    let mut decided = vec![];
    let outcome = inbound::poll_once(&run, &mut |id: &str, allow: bool| {
        decided.push((id.to_string(), allow));
        Ok(())
    })
    .unwrap();
    assert_eq!(outcome.announced, 1);
    assert_eq!(decided, vec![(id, false)]);
}
