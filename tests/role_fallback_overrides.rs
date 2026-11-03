//! #409 / DD-222: role overrides belong to reviewed tasks, never shared profiles.
use agentconductor::{
    plan::{Plan, Task},
    roles,
};
use serde_json::json;
fn plan() -> Plan {
    let agent = |id: &str| json!({"id":id,"executable":format!("agent-{id}"),"args":["{model}"],"permission_mode":"external_sandbox","catalog":{"models":["small","large"],"source":"fixture","queried_at":1,"cli_version":"1"}});
    let mut a = agent("a");
    a["fallback"] = json!({"agent":"b","model":"small"});
    serde_json::from_value(json!({"goal":"Role rules","workspace_mode":"shared","agents":[a,agent("b"),agent("c")],"tasks":[
 {"id":"dev","agent":"a","role":"development","prompt":"Develop","paths":["dev.txt"],"depends_on":[],"test_command":["true"]},
 {"id":"test","agent":"a","role":"testing","prompt":"Test","paths":["test.txt"],"depends_on":[],"test_command":["true"]}
 ]})).unwrap()
}
#[test]
fn role_fallback_overrides_one_task_and_preserves_other_role_and_profiles() {
    let mut p = plan();
    let before = serde_json::to_value(&p.agents).unwrap();
    let rules=serde_json::from_value(json!({"development":[{"agent":"a","model":"large","fallback":{"agent":"c","model":"large"}}],"testing":["a"]})).unwrap();
    roles::validate(&rules, &p.agents).unwrap();
    roles::apply(&rules, &mut p.tasks).unwrap();
    p.validate().unwrap();
    assert_eq!(
        serde_json::to_value(&p.tasks[0]).unwrap()["fallback"],
        json!({"agent":"c","model":"large"})
    );
    assert_eq!(p.tasks[0].model.as_deref(), Some("large"));
    assert_eq!(serde_json::to_value(&p.agents).unwrap(), before);
    assert_eq!(
        p.tasks[0].fallback_for(&p.agents).unwrap().unwrap().agent,
        "c"
    );
    assert_eq!(
        p.tasks[1].fallback_for(&p.agents).unwrap().unwrap().agent,
        "b"
    );
}
#[test]
fn explicit_none_disables_profile_fallback_and_legacy_serialization_is_unchanged() {
    let mut p = plan();
    let rules = serde_json::from_value(
        json!({"development":[{"agent":"a","fallback":false}],"testing":["a"]}),
    )
    .unwrap();
    roles::validate(&rules, &p.agents).unwrap();
    roles::apply(&rules, &mut p.tasks).unwrap();
    assert!(p.tasks[0].fallback_for(&p.agents).unwrap().is_none());
    let legacy = serde_json::to_value(&p.tasks[1]).unwrap();
    assert!(legacy.get("fallback").is_none());
    let restored: Task = serde_json::from_value(legacy).unwrap();
    assert_eq!(
        restored.fallback_for(&p.agents).unwrap().unwrap().agent,
        "b"
    );
}
#[test]
fn role_and_task_boundaries_reject_unknown_self_same_account_and_incompatible_models() {
    for bad in [
        json!(true),
        json!({"agent":"missing"}),
        json!({"agent":"a"}),
        json!({"agent":"c","model":"unknown"}),
        json!({"agent":"c","model":""}),
    ] {
        let p = plan();
        let rules =
            serde_json::from_value(json!({"development":[{"agent":"a","fallback":bad}]})).unwrap();
        assert!(roles::validate(&rules, &p.agents).is_err(), "{bad}");
        let mut raw = serde_json::to_value(p).unwrap();
        raw["tasks"][0]["fallback"] = bad;
        assert!(
            serde_json::from_value::<Plan>(raw)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    let mut raw = serde_json::to_value(plan()).unwrap();
    raw["agents"][0]["quota_group"] = json!("shared");
    raw["agents"][2]["quota_group"] = json!("shared");
    raw["tasks"][0]["fallback"] = json!({"agent":"c"});
    assert!(
        serde_json::from_value::<Plan>(raw)
            .unwrap()
            .validate()
            .is_err()
    );
}

#[test]
fn renaming_or_removing_a_fallback_updates_only_its_role_policy() {
    use agentconductor::{native_guard, ui::setup::Startup};
    let root = std::env::temp_dir().join(format!(
        "ac-role-policy-{}",
        native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let call = |mut packet: serde_json::Value| {
        packet["revision"] =
            startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"].clone();
        startup.dispatch(packet.to_string().as_bytes()).unwrap()
    };
    let initial = plan();
    call(
        json!({"kind":"save","config":{"profiles":initial.agents,"repository":"","goal":"Policy","coordinator":"a","roles":{"development":[{"agent":"a","fallback":{"agent":"c","model":"large"}}],"testing":["a"]}}}),
    );
    let mut profiles = serde_json::to_value(initial.agents).unwrap();
    profiles[2]["id"] = json!("renamed");
    let renamed =
        call(json!({"kind":"save_profiles","profiles":profiles,"renames":{"c":"renamed"}}));
    assert_eq!(
        renamed["config"]["roles"]["development"][0]["fallback"]["agent"],
        "renamed"
    );
    assert!(
        renamed["config"]["roles"]["testing"][0]
            .get("fallback")
            .is_none()
    );
    profiles.as_array_mut().unwrap().pop();
    let removed = call(json!({"kind":"save_profiles","profiles":profiles}));
    assert_eq!(
        removed["config"]["roles"]["development"][0]["fallback"],
        json!(false)
    );
    assert_eq!(removed["config"]["profiles"][0]["fallback"]["agent"], "b");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
