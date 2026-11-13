use agentconductor::{
    plan::AgentProfile,
    protocol::{Decoder, EventKind},
};
use serde_json::json;
fn profile(mode: &str) -> AgentProfile {
    serde_json::from_value(json!({"id":"claude","parser":"claude","executable":"claude","args":[],"permission_mode":mode})).unwrap()
}
fn decode(d: &mut Decoder, v: serde_json::Value) -> agentconductor::protocol::Event {
    d.push(format!("{v}\n").as_bytes()).unwrap().remove(0)
}
#[test]
fn full_access_checks_the_effective_mode_and_refuses_unverified_success() {
    for mode in ["default", "acceptEdits", "plan", "dontAsk", "auto", ""] {
        let mut d = Decoder::for_profile(&profile("full_access"), 1024 * 1024);
        let e = decode(
            &mut d,
            json!({"type":"system","subtype":"init","session_id":"owned","permissionMode":mode}),
        );
        assert_eq!(e.kind, EventKind::Error, "{mode}");
        assert_eq!(e.payload["denied"], true);
    }
    let mut d = Decoder::for_profile(&profile("full_access"), 1024 * 1024);
    assert_eq!(
        decode(&mut d, json!({"type":"result","is_error":false})).kind,
        EventKind::Error
    );
}
#[test]
fn denied_results_never_become_success_for_any_claude_profile() {
    for mode in ["full_access", "native_callback", "read_only"] {
        let mut d = Decoder::for_profile(&profile(mode), 1024 * 1024);
        decode(
            &mut d,
            json!({"type":"system","subtype":"init","permissionMode":"bypassPermissions"}),
        );
        let e = decode(
            &mut d,
            json!({"type":"result","is_error":false,"permission_denials":[{"tool_name":"Write","tool_input":{"content":"private marker"}}]}),
        );
        assert_eq!(e.kind, EventKind::Error);
        assert_eq!(e.payload["denied"], true);
        assert!(!e.payload.to_string().contains("private marker"));
    }
}
#[test]
fn confirmed_access_and_callback_or_read_only_success_remain_valid() {
    for mode in ["full_access", "native_callback", "read_only"] {
        let mut d = Decoder::for_profile(&profile(mode), 1024 * 1024);
        assert_eq!(decode(&mut d,json!({"type":"system","subtype":"init","permissionMode":if mode=="full_access" {"bypassPermissions"} else {"default"}})).kind,EventKind::Session);
        assert_eq!(
            decode(
                &mut d,
                json!({"type":"result","is_error":false,"permission_denials":[]})
            )
            .kind,
            EventKind::Result
        );
    }
}

#[test]
fn mode_updates_denials_shapes_and_other_providers_follow_their_own_contracts() {
    let mut d = Decoder::for_profile(&profile("full_access"), 1024 * 1024);
    decode(
        &mut d,
        json!({"type":"system","subtype":"init","permissionMode":"bypassPermissions"}),
    );
    assert_eq!(
        decode(
            &mut d,
            json!({"type":"system","subtype":"status","permissionMode":"default"})
        )
        .kind,
        EventKind::Error
    );
    for denied in [json!({}), json!("unexpected")] {
        let mut d = Decoder::for_profile(&profile("read_only"), 1024 * 1024);
        assert_eq!(
            decode(
                &mut d,
                json!({"type":"result","is_error":false,"permission_denials":denied})
            )
            .kind,
            EventKind::Error
        );
    }
    let mut d = Decoder::for_profile(&profile("full_access"), 1024 * 1024);
    let e = decode(
        &mut d,
        json!({"type":"result","is_error":false,"api_error_status":429,"api_error":"usage_limit_reached"}),
    );
    assert_eq!(e.kind, EventKind::Error);
    assert_eq!(e.payload["api_error_status"], 429);
    assert!(e.payload.get("denied").is_none());
    let mut d = Decoder::new(agentconductor::protocol::Provider::Custom, 1024 * 1024);
    assert_eq!(
        decode(
            &mut d,
            json!({"type":"result","permission_denials":["provider-specific"]})
        )
        .kind,
        EventKind::Result
    );
}

#[test]
fn metadata_evidence_is_bound_to_the_execution_profile_and_unknown_is_not_verified() {
    use agentconductor::protocol::claude_permissions::{configuration_hint, observation};
    let root = std::env::temp_dir().join(format!(
        "ac-policy-store-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let mut store = agentconductor::state::Store::open(root.join("state.sqlite")).unwrap();
    let mut p = profile("full_access");
    assert_eq!(
        configuration_hint(&p, &store),
        Some("startup.claude_permission_unverified")
    );
    for mode in [json!("default"), json!("bypassPermissions"), json!(null)] {
        let reply = json!({"response":{"response":{"current_permission_mode":mode}}});
        store
            .record(
                "claude_permission/claude",
                &observation(&p, &reply, 100).unwrap(),
                "fixture",
            )
            .unwrap();
        assert_eq!(
            configuration_hint(&p, &store),
            match mode.as_str() {
                Some("default") => Some("startup.claude_permission_mismatch"),
                Some("bypassPermissions") => None,
                _ => Some("startup.claude_permission_unverified"),
            }
        );
    }
    let reply = json!({"response":{"response":{"current_permission_mode":"bypassPermissions"}}});
    store
        .record(
            "claude_permission/claude",
            &observation(&p, &reply, 100).unwrap(),
            "fixture",
        )
        .unwrap();
    p.executable = "other-claude".into();
    assert_eq!(
        configuration_hint(&p, &store),
        Some("startup.claude_permission_unverified")
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn read_only_coordinator_does_not_claim_worker_full_access_but_still_rejects_denials() {
    let root = std::env::temp_dir().join(format!(
        "ac-policy-coordinator-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("stdout.jsonl");
    let p = profile("full_access");
    for denied in [false, true] {
        let init = json!({"type":"system","subtype":"init","permissionMode":"default"});
        let result = json!({"type":"result","is_error":false,"result":"{\"tasks\":[]}","permission_denials":if denied{json!([{"tool_name":"Read"}])}else{json!([])}});
        std::fs::write(&path, format!("{init}\n{result}\n")).unwrap();
        let outcome = agentconductor::coordinator::read_profile_document(&p, &path, "tasks");
        if denied {
            assert!(outcome.unwrap_err().contains("permission_denials"));
        } else {
            assert_eq!(outcome.unwrap()["tasks"], json!([]));
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
