//! #862 / DD-231: bug drafts share the updater repository and never accept arbitrary URLs.
use agentconductor::{bug_report, ui::Room, update_source};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

fn decoded_body(url: &str) -> String {
    let text = url.split_once("?body=").unwrap().1.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        if text[i] == b'%' {
            out.push(
                u8::from_str_radix(std::str::from_utf8(&text[i + 1..i + 3]).unwrap(), 16).unwrap(),
            );
            i += 3;
        } else {
            out.push(text[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}

#[test]
fn drafts_use_public_default_and_enterprise_repository_routes() {
    for (input, base) in [
        ("", "https://github.com/neodamul/AgentConductor"),
        ("team/tool", "https://github.com/team/tool"),
        (
            "https://github.com/team/tool.git",
            "https://github.com/team/tool",
        ),
        (
            "https://git.corp.example:8443/team/tool.git",
            "https://git.corp.example:8443/team/tool",
        ),
    ] {
        let draft = bug_report::draft(input).unwrap();
        let url = draft["url"].as_str().unwrap();
        assert!(
            url.starts_with(&format!("{base}/issues/new?body=")),
            "{url}"
        );
        assert!(url.is_ascii() && !url.contains([' ', '\n', '\r', '#', '&']));
        assert!(
            url.len() < 2000,
            "the fixed template fits browser URL limits"
        );
        let body = decoded_body(url);
        assert_eq!(body, draft["body"]);
        for section in [
            "## Summary",
            "## Steps to reproduce",
            "## Expected behavior",
            "## Actual behavior",
            "## Environment",
        ] {
            assert!(body.contains(section));
        }
        assert!(body.contains(&agentconductor::update::current()));
        assert!(body.contains(std::env::consts::OS));
        assert!(body.contains(std::env::consts::ARCH));
        assert!(!body.contains("/Users/") && !body.contains("Bearer "));
    }
}

#[test]
fn malformed_targets_and_untrusted_navigation_fields_are_rejected() {
    for bad in [
        "http://github.com/team/tool",
        "javascript:alert(1)",
        "https://user:token@host/team/tool",
        "team/tool?body=other",
        "team/tool/issues/new",
        "team/tool\n--exec",
    ] {
        assert!(bug_report::draft(bad).is_err(), "{bad}");
    }
    for bad in [
        json!({"kind":"report_bug","url":"https://attacker.invalid"}),
        json!({"kind":"report_bug","open":"true"}),
    ] {
        assert!(bug_report::handle(bad).is_err());
    }
}

#[test]
fn the_shared_native_request_reads_updated_repository_preferences_without_opening_a_browser() {
    let (root, _) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let room = Room::startup(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    for repository in ["team/public", "https://ghe.corp.example/team/internal", ""] {
        room.dispatch(
            json!({"kind":"set_update_settings","repository":repository})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
        let report = room
            .dispatch(br#"{"kind":"report_bug","open":false}"#)
            .unwrap();
        assert_eq!(report["opened"], false);
        assert_eq!(report["repository"], update_source::repository());
        assert_eq!(report["url"], bug_report::draft(repository).unwrap()["url"]);
    }
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "opens the default browser for manual draft verification; never submits an issue"]
fn opens_the_default_browser_on_an_operator_request() {
    let root = std::env::temp_dir().join(format!("ac-bug-report-browser-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut env = test_env::lock();
    env.set("HOME", &root);
    let report = bug_report::handle(json!({"kind":"report_bug","open":true})).unwrap();
    assert_eq!(report["opened"], true);
    println!("Opened configured report draft: {}", report["url"]);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn release_notes_open_the_running_version_on_the_update_repository() {
    for (input, base) in [
        ("", "https://github.com/neodamul/AgentConductor"),
        (
            "https://git.corp.example:8443/team/tool.git",
            "https://git.corp.example:8443/team/tool",
        ),
    ] {
        let notes = bug_report::release_notes(input).unwrap();
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(notes["version"], version);
        assert_eq!(notes["url"], format!("{base}/releases/tag/v{version}"));
    }
    assert!(bug_report::release_notes("javascript:alert(1)").is_err());
}
