use agentconductor::{
    availability::Restriction,
    workflow::{self, Report},
};
use serde_json::json;
fn waiting(at: Option<i64>, fallback: bool) -> Report {
    let restriction = Restriction {
        available_at: at,
        observed_at: 100000,
        reason: "quota".into(),
        source: "fixture".into(),
    };
    let mut agent =
        json!({"id":"paid","executable":"fixture","args":[],"quota_group":"paid-account"});
    if fallback {
        agent["fallback"] = json!({"agent":"backup"});
    }
    serde_json::from_value(json!({"schema_version":1,"state":"waiting_availability","repo":"repo","base":"base","branch":"main","plan":{"goal":"three tasks","workspace_mode":"shared","agents":[agent,{"id":"backup","executable":"backup","args":[]}],"tasks":[{"id":"one","agent":"paid","prompt":"one","paths":["one"],"test_command":["test"]}]},"tasks":[{"id":"one","status":"waiting_availability","attempt":1,"restriction":restriction,"cwd":"repo","started_ms":null,"finished_ms":null,"session_id":null,"error":"quota","base":"base","snapshot":null,"artifacts":"artifacts","retry_not_before":0}],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":"repo","account_restrictions":{"paid-account":restriction}})).unwrap()
}
#[test]
fn a_known_reset_within_five_hours_waits_and_the_boundary_is_inclusive() {
    let r = waiting(Some(103600), true);
    let t = workflow::availability_timer(&r, 100000);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].kind, "resume");
    assert_eq!(t[0].at, 103600);
    assert_eq!(
        workflow::availability_timer(&waiting(Some(118000), true), 100000)[0].kind,
        "resume"
    );
}
#[test]
fn long_resets_use_only_a_reviewed_fallback_and_unknown_resets_get_five_hours() {
    let t = workflow::availability_timer(&waiting(Some(118001), true), 100000);
    assert_eq!(t[0].kind, "fallback");
    assert_eq!(t[0].at, 100000);
    let t = workflow::availability_timer(&waiting(None, true), 100001);
    assert_eq!(t[0].at, 118000);
    assert_eq!(t[0].kind, "fallback");
    assert!(workflow::availability_timer(&waiting(None, false), 100000).is_empty());
    assert_eq!(
        workflow::availability_timer(&waiting(Some(120000), false), 100000)[0].kind,
        "resume"
    );
}
#[test]
fn reassignment_stop_and_completed_work_have_no_stale_timer() {
    let mut r = waiting(Some(103600), true);
    r.plan.tasks[0].agent = "backup".into();
    r.tasks[0].restriction = None;
    r.tasks[0].status = "pending".into();
    r.state = "ready".into();
    assert!(workflow::availability_timer(&r, 100000).is_empty());
    for state in ["stopped", "completed", "waiting_approval"] {
        let mut r = waiting(Some(103600), true);
        r.state = state.into();
        assert!(workflow::availability_timer(&r, 100000).is_empty());
    }
}
#[test]
fn durable_reload_keeps_the_deadline_and_exhausted_retry_budget_does_not_spin() {
    let r = waiting(Some(103600), true);
    let restored: Report = serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
    assert_eq!(
        workflow::availability_timer(&restored, 100000),
        workflow::availability_timer(&r, 100000)
    );
    let mut r = restored;
    r.tasks[0].automatic_retries = 3;
    assert!(workflow::availability_timer(&r, 100000).is_empty());
}

#[test]
fn only_fresh_exhausted_shared_windows_supply_an_account_deadline() {
    let r=agentconductor::quota::restriction(&json!({"status":"available","observed_at":100,"windows":[{"label":"five_hour","remaining_percent":0,"resets_at":200},{"label":"seven_day","remaining_percent":30,"resets_at":900}]}),100).unwrap();
    assert_eq!(r.available_at, Some(200));
    let r=agentconductor::quota::restriction(&json!({"status":"available","observed_at":100,"windows":[{"label":"five_hour","remaining_percent":0,"resets_at":200},{"label":"seven_day","remaining_percent":0,"resets_at":null}]}),100).unwrap();
    assert_eq!(r.available_at, None);
    assert!(agentconductor::quota::restriction(&json!({"status":"stale","observed_at":100,"windows":[{"label":"five_hour","remaining_percent":0,"resets_at":200}]}),100).is_none());
    assert!(agentconductor::quota::restriction(&json!({"status":"available","observed_at":100,"windows":[{"label":"seven_day_opus","remaining_percent":0,"resets_at":200}]}),100).is_none());
}
