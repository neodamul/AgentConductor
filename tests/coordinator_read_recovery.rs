use agentconductor::{
    coordinator,
    plan::AgentProfile,
    protocol::{Decoder, EventKind},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};
fn profile(mode: &str) -> AgentProfile {
    serde_json::from_value(json!({"id":"claude","parser":"claude","executable":"claude","args":[],"permission_mode":mode})).unwrap()
}
fn call(id: &str, name: &str) -> Value {
    json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":id,"name":name,"input":{}}]}})
}
fn outcome(id: &str, error: bool) -> Value {
    json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":id,"is_error":error,"content":"source evidence"}]}})
}
fn recovered() -> Vec<Value> {
    vec![
        call("denied-glob", "Glob"),
        outcome("denied-glob", true),
        call("denied-read", "Read"),
        outcome("denied-read", true),
        call("snapshot-glob", "Glob"),
        outcome("snapshot-glob", false),
        call("snapshot-read", "Read"),
        outcome("snapshot-read", false),
    ]
}
fn final_result() -> Value {
    json!({"type":"result","is_error":false,"result":"{\"phases\":[]}","permission_denials":[{"tool_name":"Glob","tool_use_id":"denied-glob"},{"tool_name":"Read","tool_use_id":"denied-read"}]})
}
fn path(events: &[Value]) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ac-read-recovery-{}.jsonl",
        agentconductor::native_guard::session_id().unwrap()
    ));
    fs::write(
        &p,
        events.iter().map(|v| format!("{v}\n")).collect::<String>(),
    )
    .unwrap();
    p
}
#[test]
fn a_recovered_read_only_coordinator_result_is_validated_and_accepted() {
    let mut events = recovered();
    events.push(final_result());
    let p = path(&events);
    let result = coordinator::read_profile_document(&profile("full_access"), &p, "phases");
    fs::remove_file(p).unwrap();
    assert_eq!(result.unwrap()["phases"], json!([]));
}
#[test]
fn unresolved_late_write_and_unproven_denials_remain_failures() {
    for mode in ["unresolved", "late", "write", "unproven", "malformed"] {
        let mut events = if mode == "unresolved" {
            vec![call("denied-read", "Read"), outcome("denied-read", true)]
        } else {
            recovered()
        };
        let mut result = final_result();
        if mode == "late" {
            events.extend([call("late", "Read"), outcome("late", true)]);
        }
        if mode == "write" {
            result["permission_denials"] = json!([{"tool_name":"Write","tool_use_id":"write"}]);
        }
        if mode == "unproven" {
            result["permission_denials"] = json!([{"tool_name":"Read","tool_use_id":"unseen"}]);
        }
        if mode == "malformed" {
            result["permission_denials"] = json!({});
        }
        events.push(result);
        let p = path(&events);
        let error =
            coordinator::read_profile_document(&profile("full_access"), &p, "phases").unwrap_err();
        fs::remove_file(p).unwrap();
        assert!(error.contains("permission"), "{mode}: {error}");
        assert!(
            !error.contains("select native_callback"),
            "planning must not offer an ineffective worker-only workaround"
        );
    }
}
#[test]
fn recovery_never_accepts_invalid_output_or_relaxes_worker_completion() {
    let mut events = recovered();
    let mut result = final_result();
    result["result"] = json!("not a plan");
    events.push(result);
    let p = path(&events);
    assert!(coordinator::read_profile_document(&profile("full_access"), &p, "phases").is_err());
    fs::remove_file(p).unwrap();
    for mode in ["full_access", "native_callback", "read_only"] {
        let mut d = Decoder::for_profile(&profile(mode), 1024 * 1024);
        d.push(
            format!(
                "{}\n",
                json!({"type":"system","subtype":"init","permissionMode":"bypassPermissions"})
            )
            .as_bytes(),
        )
        .unwrap();
        let mut events = recovered();
        events.push(final_result());
        let raw = events.iter().map(|v| format!("{v}\n")).collect::<String>();
        let decoded = d.push(raw.as_bytes()).unwrap();
        assert_eq!(decoded.last().unwrap().kind, EventKind::Error, "{mode}");
    }
}

#[test]
fn malformed_or_overflowing_read_history_cannot_authorize_recovery() {
    for mode in ["malformed", "overflow"] {
        let mut events = recovered();
        if mode == "malformed" {
            events.push(call("malformed", "Read"));
            let mut bad = outcome("malformed", false);
            bad["message"]["content"][0]["is_error"] = json!("false");
            events.push(bad);
        } else {
            for n in 0..65 {
                events.push(call(&format!("pending-{n}"), "Read"));
            }
        }
        events.push(final_result());
        let p = path(&events);
        assert!(coordinator::read_profile_document(&profile("full_access"), &p, "phases").is_err());
        fs::remove_file(p).unwrap();
    }
}
