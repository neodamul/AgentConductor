use agentconductor::quota::{parse_claude, parse_codex};
use serde_json::json;

#[test]
fn reported_used_percent_becomes_remaining_for_each_account_window() {
    let data = json!({"result":{"rateLimits":{"primary":{"usedPercent":99}},"rateLimitsByLimitId":{"codex":{"primary":{"usedPercent":27,"windowDurationMins":300,"resetsAt":2200},"secondary":{"usedPercent":64,"windowDurationMins":10080,"resetsAt":9000}},"other":{"primary":{"usedPercent":0,"windowDurationMins":60,"resetsAt":2000}}}}});
    let observed = parse_codex(&data, 1000).unwrap();
    let view = observed.view(1001);
    assert_eq!(view.windows.len(), 3);
    assert_eq!(view.windows[0].remaining_percent, Some(73.0));
    assert_eq!(view.windows[1].remaining_percent, Some(36.0));
    assert_eq!(view.windows[2].remaining_percent, Some(100.0));
    assert_eq!(view.windows[0].resets_at, Some(2200));
    assert_eq!(view.source, "codex app-server account/rateLimits/read");
}
#[test]
fn missing_usage_and_cost_are_not_subscription_balance() {
    let data = json!({"context_window":{"remaining_percentage":73},"cost":{"total_cost_usd":0.23}});
    assert!(parse_claude(&data, 1000).unwrap().windows.is_empty());
    assert!(
        parse_codex(&json!({"result":{"rateLimits":null}}), 1000)
            .unwrap()
            .windows
            .is_empty()
    );
    let unknown = parse_codex(
        &json!({"result":{"rateLimits":{"primary":{"usedPercent":null,"resetsAt":2000}}}}),
        1000,
    )
    .unwrap();
    assert_eq!(unknown.view(1001).windows[0].remaining_percent, None);
}
#[test]
fn claude_windows_are_independent_and_overage_is_not_negative_balance() {
    let snapshot=parse_claude(&json!({"rate_limits":{"five_hour":{"used_percentage":27,"resets_at":2000},"seven_day":null,"spend_limit":{"used_percentage":112,"resets_at":3000}}}),1000).unwrap();
    assert_eq!(snapshot.windows.len(), 2);
    assert_eq!(snapshot.view(1001).windows[0].remaining_percent, Some(73.0));
    assert_eq!(snapshot.view(1001).windows[1].remaining_percent, Some(0.0));
    assert_eq!(snapshot.windows[1].used_percent, Some(112.0));
}
#[test]
fn expired_future_or_old_observations_are_not_current_capacity() {
    let snapshot = parse_claude(
        &json!({"rate_limits":{"five_hour":{"used_percentage":27,"resets_at":2000}}}),
        1000,
    )
    .unwrap();
    for now in [999, 1301, 2000] {
        let view = snapshot.view(now);
        assert!(view.stale);
        assert_eq!(view.windows[0].remaining_percent, None);
    }
    assert!(
        parse_claude(
            &json!({"rate_limits":{"five_hour":{"used_percentage":-1}}}),
            1000
        )
        .is_err()
    );
    assert!(
        parse_codex(
            &json!({"result":{"rateLimits":{"primary":{"usedPercent":"27"}}}}),
            1000
        )
        .is_err()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn bounded_codex_metadata_query_requests_no_generation_and_preserves_auth() {
    let root = std::env::temp_dir().join(format!(
        "ac-quota-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let executable = root.join("quota-fixture");
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/quota_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let profile: agentconductor::plan::AgentProfile = serde_json::from_value(
        json!({"id":"codex","parser":"codex","executable":executable,"args":["exec","-","--json"]}),
    )
    .unwrap();
    let snapshot = agentconductor::quota::query(&profile, &root.join("query"))
        .await
        .unwrap();
    assert_eq!(
        snapshot.view(snapshot.observed_at).windows[0].remaining_percent,
        Some(73.0)
    );
    let mut enterprise = profile.clone();
    enterprise
        .environment
        .insert("QUOTA_TEST_ENTERPRISE".into(), "1".into());
    let budget = agentconductor::quota::query(&enterprise, &root.join("enterprise"))
        .await
        .unwrap();
    assert_eq!(budget.budgets[0].remaining.as_deref(), Some("640"));
    assert_eq!(budget.budgets[0].allocated.as_deref(), Some("1000"));
    let requests = std::fs::read_to_string(root.join("query/requests.jsonl")).unwrap();
    assert!(requests.contains("account/rateLimits/read"));
    assert!(!requests.contains("thread/start") && !requests.contains("turn/start"));
    assert!(
        agentconductor::quota::query(&profile, &root.join("query"))
            .await
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
#[cfg(unix)]
#[tokio::test]
async fn reviewed_run_persists_quota_without_changing_execution_evidence() {
    let (root, mut plan) = support::setup("shared");
    let executable = root.join("quota-fixture");
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/quota_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    plan.agents.push(serde_json::from_value(json!({"id":"codex","parser":"codex","executable":executable,"args":[],"permission_mode":"unverified"})).unwrap());
    let run = root.join("run");
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let before = agentconductor::workflow::revision(&report).unwrap();
    agentconductor::workflow::refresh_quota(&run, "codex")
        .await
        .unwrap();
    assert_eq!(
        agentconductor::workflow::revision(&agentconductor::workflow::load(&run).unwrap()).unwrap(),
        before
    );
    let store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
    let snapshot = agentconductor::quota::stored(&store, &report.plan.agents[1], i64::MAX).unwrap();
    assert_eq!(snapshot["status"], "stale");
    assert!(snapshot["windows"][0]["remaining_percent"].is_null());
    let current = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let snapshot = agentconductor::quota::stored(&store, &report.plan.agents[1], current).unwrap();
    assert_eq!(snapshot["windows"][0]["remaining_percent"], 73.0);
    let room = agentconductor::ui::Room::open(
        &run,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let board = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        board["quotas"]["codex"]["windows"][0]["remaining_percent"],
        73.0
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn provider_plan_type_survives_cache_round_trip_without_inference() {
    let observed=parse_codex(&json!({"result":{"rateLimitsByLimitId":{"codex":{"planType":"enterprise","primary":{"usedPercent":27,"windowDurationMins":300}}}}}),1000).unwrap();
    assert_eq!(observed.plan_type.as_deref(), Some("enterprise"));
    let roundtrip: agentconductor::quota::Snapshot =
        serde_json::from_value(serde_json::to_value(observed).unwrap()).unwrap();
    assert_eq!(
        roundtrip.view(1001).plan_type.as_deref(),
        Some("enterprise")
    );
    let old = parse_codex(&json!({"rateLimits":{"primary":{"usedPercent":1}}}), 1000).unwrap();
    assert!(old.plan_type.is_none());
}

#[test]
fn enterprise_individual_budget_is_available_without_rolling_windows() {
    let data = json!({"result":{"rateLimitsByLimitId":{"codex":{"planType":"enterprise","primary":null,"secondary":null,"individualLimit":{"limit":"1000","used":"360","remainingPercent":64,"resetsAt":9000},"credits":{"hasCredits":true,"unlimited":false,"balance":"730"}}}}});
    let snap = parse_codex(&data, 1000).unwrap();
    let view = serde_json::to_value(snap.view(1001)).unwrap();
    assert_eq!(view["budgets"][0]["remaining_percent"], 64.0);
    assert_eq!(view["budgets"][0]["allocated"], "1000");
    assert_eq!(view["budgets"][0]["used"], "360");
    assert_eq!(view["budgets"][0]["scope"], "individual");
    assert_eq!(view["budgets"][1]["remaining"], "730");
    assert!(view["budgets"][1]["allocated"].is_null());
    let profile: agentconductor::plan::AgentProfile = serde_json::from_value(
        json!({"id":"enterprise","parser":"codex","executable":"codex","args":[]}),
    )
    .unwrap();
    let path = std::env::temp_dir().join(format!(
        "ac-enterprise-{}.sqlite",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let mut store = agentconductor::state::Store::open(&path).unwrap();
    agentconductor::quota::persist(&mut store, &profile, Ok(snap)).unwrap();
    assert_eq!(
        agentconductor::quota::stored(&store, &profile, 1001).unwrap()["status"],
        "available"
    );
    assert!(agentconductor::quota::stored(&store,&profile,1301).unwrap()["budgets"][0]["remaining_percent"].is_null());
    drop(store);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn empty_bucket_map_keeps_legacy_quota_and_unknown_allocation_is_not_invented() {
    let snap=parse_codex(&json!({"result":{"rateLimitsByLimitId":{},"rateLimits":{"planType":"enterprise","credits":{"hasCredits":true,"unlimited":false,"balance":"730"}}}}),1000).unwrap();
    let view = serde_json::to_value(snap.view(1001)).unwrap();
    assert_eq!(view["budgets"][0]["remaining"], "730");
    assert!(view["budgets"][0]["remaining_percent"].is_null());
    assert!(view["budgets"][0]["allocated"].is_null());
    let no_limit=parse_codex(&json!({"rateLimits":{"planType":"enterprise","credits":{"hasCredits":true,"unlimited":true,"balance":null}}}),1000).unwrap();
    assert_eq!(
        serde_json::to_value(no_limit.view(1001)).unwrap()["budgets"][0]["unlimited"],
        true
    );
}

#[test]
fn budget_quantities_are_exact_bounded_and_expire_independently() {
    let data = json!({"rateLimits":{"planType":"enterprise","individualLimit":{"limit":"10.10","used":"3.07","remainingPercent":70,"resetsAt":1100}}});
    let snap = parse_codex(&data, 1000).unwrap();
    let fresh = serde_json::to_value(snap.view(1001)).unwrap();
    assert_eq!(fresh["budgets"][0]["remaining"], "7.03");
    assert!(serde_json::to_value(snap.view(1100)).unwrap()["budgets"][0]["remaining"].is_null());
    for bad in ["-1", "NaN", "1e100", "secret", "100000000000000000000000"] {
        let mut invalid = data.clone();
        invalid["rateLimits"]["individualLimit"]["limit"] = bad.into();
        assert!(parse_codex(&invalid, 1000).is_err());
    }
    let legacy: agentconductor::quota::Snapshot =
        serde_json::from_value(json!({"source":"old","observed_at":1000,"windows":[]})).unwrap();
    assert!(legacy.budgets.is_empty());
}
