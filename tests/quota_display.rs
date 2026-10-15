use agentconductor::quota::badge;
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
fn quota_badge_rendering_is_native_snapshot_data() {
    let startup = include_str!("../src/ui/startup.js");
    let room = include_str!("../src/ui/room.js");
    assert!(!startup.contains("function quotaBadge"));
    assert!(!room.contains("function quotaBadge"));
    assert!(startup.contains("state?.quota_badges?.[p.id]"));
    assert!(!room.contains("quotaBadge("));
    assert!(startup.contains("quota-controls"));
    assert!(startup.contains("quota-refresh"));
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
