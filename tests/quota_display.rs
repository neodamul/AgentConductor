use agentconductor::quota::{badge, budget_rows, window_labels, window_reset_labels};
use serde_json::json;

#[test]
fn quota_badges_sort_windows_and_keep_enterprise_percentages_conservative() {
    let badge = badge(
        &json!({"plan_type":"enterprise","windows":[
            {"label":"seven_day","window_minutes":10080,"remaining_percent":36},
            {"label":"five_hour","window_minutes":300,"remaining_percent":73}
        ],"budgets":[{"scope":"individual","remaining":"640","allocated":"1000","remaining_percent":64}]}),
        "en",
        1_800_000_000,
    );
    assert_eq!(badge.text, "64%/100%");
    assert!(badge.title.contains("5h: 73% remaining"));
    assert!(badge.title.contains("Weekly: 36% remaining"));
    assert!(badge.title.contains("Individual allocation: 640/1000"));
}

#[test]
fn quota_badges_cover_stale_unavailable_unlimited_reset_and_localized_labels() {
    let stale = badge(
        &json!({"status":"stale","windows":[{"remaining_percent":73}]}),
        "en",
        1_800_000_000,
    );
    assert_eq!(stale.text, "Stale");
    assert!(stale.title.starts_with("Quota observation is stale."));
    assert_eq!(
        badge(
            &json!({"windows":[{"remaining_percent":null}]}),
            "en",
            1_800_000_000
        )
        .text,
        "—"
    );
    assert_eq!(
        badge(
            &json!({"budgets":[{"scope":"credits","unlimited":true}]}),
            "en",
            1_800_000_000
        )
        .text,
        "∞"
    );
    let reset = badge(
        &json!({"windows":[{"label":"five_hour","remaining_percent":0,"resets_at":1800003540}]}),
        "en",
        1_800_000_000,
    );
    assert_eq!(reset.text, "0%");
    assert!(reset.title.contains("resets "));
    let ko = badge(
        &json!({"windows":[{"label":"seven_day","window_minutes":10080,"remaining_percent":56}]}),
        "ko",
        1_800_000_000,
    );
    assert_eq!(ko.text, "56%");
    assert!(ko.title.contains("주간: 56% 남음"));
}

#[test]
fn badge_labels_errors_and_reset_unknown_are_preserved() {
    let result = badge(
        &json!({"error":"ChatGPT sign-in","source":"provider","windows":[
            {"label":"seven_day_fable","window_minutes":10080,"remaining_percent":100},
            {"label":"primary","window_minutes":1440,"remaining_percent":0}
        ]}),
        "en",
        1_800_000_000,
    );
    assert_eq!(result.text, "0%/100%");
    assert!(result.title.contains("Daily: 0%"));
    assert!(result.title.contains("Weekly fable: 100%"));
    assert!(result.title.contains("reset unknown"));
    assert!(result.title.contains("ChatGPT sign-in"));
    assert!(result.title.contains("provider"));
}

#[test]
fn room_window_labels_use_rust_localized_agent_intervals() {
    let quotas = json!({
        "claude":{"windows":[
            {"label":"five_hour","window_minutes":300},
            {"label":"seven_day_opus","window_minutes":10080},
            {"label":"primary","window_minutes":1440},
            {"label":"rolling","window_minutes":30}
        ]},
        "codex":{"windows":[{"label":"seven_day_sonnet","window_minutes":10080}]}
    });
    let en = window_labels(&quotas, "en");
    assert_eq!(
        en["claude"],
        json!(["5h", "Weekly Opus", "Daily", "30-minute"])
    );
    assert_eq!(en["codex"], json!(["Weekly Sonnet"]));
    let ko = window_labels(&quotas, "ko");
    assert_eq!(ko["claude"], json!(["5h", "주간 Opus", "일간", "30분"]));
}

#[test]
fn room_budget_rows_are_localized_rust_snapshot_data() {
    let quotas = json!({
        "enterprise": {"budgets":[
            {"scope":"individual","remaining":"640","allocated":"1000","remaining_percent":64,"resets_at":1800003540},
            {"unlimited":true},
            {"scope":"credits","remaining":"12","allocated":"50"}
        ]},
        "stale": {"status":"stale","budgets":[{"remaining":"2","allocated":"10"}]}
    });
    let rows = budget_rows(&quotas, "en");
    assert_eq!(
        rows["enterprise"][0]["headline"],
        "64% remaining · Individual allocation"
    );
    assert_eq!(rows["enterprise"][0]["amounts"], "640/1000");
    assert!(
        rows["enterprise"][0]["reset_label"]
            .as_str()
            .unwrap()
            .starts_with("Resets 2027-")
    );
    assert_eq!(
        rows["enterprise"][1]["headline"],
        "Unlimited · Credit balance"
    );
    assert!(rows["enterprise"][1]["amounts"].is_null());
    assert_eq!(
        rows["enterprise"][2]["headline"],
        "24% remaining · Credit balance"
    );
    assert_eq!(rows["enterprise"][2]["amounts"], "12/50");
    assert_eq!(rows["stale"][0]["headline"], "Stale · Credit balance");
    let ko = budget_rows(&quotas, "ko");
    assert_eq!(ko["enterprise"][0]["headline"], "64% 남음 · 개인 할당량");
}

#[test]
fn room_quota_reset_times_are_localized_rust_snapshot_data() {
    let quotas = json!({
        "enterprise": {"windows":[{"resets_at":1800003540}],"budgets":[
            {"scope":"individual","remaining":"640","allocated":"1000","resets_at":1800003540},
            {"scope":"credits","remaining":"12","allocated":"50"}
        ]}
    });
    let en = window_reset_labels(&quotas, "en");
    assert!(
        en["enterprise"][0]
            .as_str()
            .unwrap()
            .starts_with("Resets 2027-")
    );
    assert!(en["enterprise"][1].is_null());
    let ko = window_reset_labels(&quotas, "ko");
    assert!(
        ko["enterprise"][0]
            .as_str()
            .unwrap()
            .starts_with("재설정 2027-")
    );

    let rows = budget_rows(&quotas, "en");
    assert!(
        rows["enterprise"][0]["reset_label"]
            .as_str()
            .unwrap()
            .starts_with("Resets 2027-")
    );
    assert!(rows["enterprise"][1]["reset_label"].is_null());
    assert!(
        budget_rows(&quotas, "ko")["enterprise"][0]["reset_label"]
            .as_str()
            .unwrap()
            .starts_with("재설정 2027-")
    );
}
