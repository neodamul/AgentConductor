#![cfg(unix)]
//! DD-72: metadata preferences persist without invalidating execution evidence.
use agentconductor::{plan::AgentProfile, ui::setup::Startup};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
fn snapshot(s: &Startup) -> serde_json::Value {
    s.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(s: &Startup, mut p: serde_json::Value) -> Result<serde_json::Value, String> {
    p["revision"] = snapshot(s)["revision"].clone();
    s.dispatch(p.to_string().as_bytes())
}
#[test]
fn default_and_bounds_are_backward_compatible() {
    let (_, mut plan) = support::setup("shared");
    let old = serde_json::json!({"id":"legacy","executable":"agent","args":[]});
    let p: AgentProfile = serde_json::from_value(old).unwrap();
    assert_eq!(p.quota_refresh_minutes, 5);
    assert!(
        serde_json::to_value(p)
            .unwrap()
            .get("quota_refresh_minutes")
            .is_none()
    );
    for bad in [0, 61] {
        plan.agents[0].quota_refresh_minutes = bad;
        assert!(plan.validate().is_err());
    }
    plan.agents[0].quota_refresh_minutes = 10;
    plan.validate().unwrap();
}
#[test]
fn cadence_change_preserves_reviewed_plan_and_survives_reopen() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("desktop");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let s = Startup::open(&dir, exe).unwrap();
    mutate(&s,serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":root.join("repo"),"goal":"Metadata preferences","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    mutate(&s, serde_json::json!({"kind":"draft"})).unwrap();
    let before = snapshot(&s);
    let mut profiles = before["config"]["profiles"].clone();
    profiles[0]["quota_refresh_minutes"] = 10.into();
    mutate(
        &s,
        serde_json::json!({"kind":"save_profiles","profiles":profiles}),
    )
    .unwrap();
    assert_eq!(snapshot(&s)["proposal"], before["proposal"]);
    drop(s);
    let reopened = Startup::open(&dir, exe).unwrap();
    assert_eq!(
        snapshot(&reopened)["config"]["profiles"][0]["quota_refresh_minutes"],
        10
    );
}
