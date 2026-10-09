use agentconductor::availability::{detect, parse_time};
use serde_json::json;

#[test]
fn quota_reset_is_exact_and_ambiguous_prose_needs_manual_schedule() {
    let restriction = detect(
        &json!({"error":{"code":"rate_limit_exceeded","resets_at":2000000000}}),
        "",
        100,
    )
    .unwrap();
    assert_eq!(restriction.available_at, Some(2000000000));
    assert_eq!(
        detect(
            &json!({}),
            "You've hit your usage limit; resets tomorrow at 2pm",
            100
        )
        .unwrap()
        .available_at,
        None
    );
    assert!(
        detect(
            &json!({"error":{"code":"authentication_failed"}}),
            "login required",
            100
        )
        .is_none()
    );
}

#[test]
fn timestamp_offsets_and_relative_retry_are_unambiguous() {
    assert_eq!(
        parse_time("2026-10-10T09:00:00+09:00").unwrap(),
        parse_time("2026-10-10T00:00:00Z").unwrap()
    );
    assert!(parse_time("2026-10-10 09:00").is_err());
    assert_eq!(
        detect(
            &json!({"error":{"code":"overloaded","retry_after":30}}),
            "",
            100
        )
        .unwrap()
        .available_at,
        Some(130)
    );
    assert_eq!(
        detect(
            &json!({}),
            "Rate limit exceeded. Retry at 2026-10-10T09:00:00+09:00",
            100
        )
        .unwrap()
        .available_at,
        Some(parse_time("2026-10-10T00:00:00Z").unwrap())
    );
}

#[test]
fn multiple_windows_wait_for_last_exhausted_reset_and_do_not_guess() {
    let limit = json!({"error":{"code":"usage_limit_reached"}, "rateLimits":{"primary":{"usedPercent":100,"resetsAt":200},"secondary":{"usedPercent":100,"resetsAt":400}}});
    assert_eq!(detect(&limit, "", 100).unwrap().available_at, Some(400));
    assert!(detect(&json!({"resetsAt":300}), "normal progress", 100).is_none());
    assert_eq!(
        detect(
            &json!({"error":{"code":"rate_limit_exceeded","retry_after":-2}}),
            "",
            100
        )
        .unwrap()
        .available_at,
        None
    );
}

#[test]
fn timestamp_in_nested_error_and_structured_http_status_are_detected() {
    let message = json!({"error":{"message":"Usage limit reached. Retry at 2026-10-10T00:00:00Z"}});
    assert_eq!(
        detect(&message, &message.to_string(), 100)
            .unwrap()
            .available_at,
        Some(parse_time("2026-10-10T00:00:00Z").unwrap())
    );
    assert!(detect(&json!({"error":{"status":429,"retry_after":60}}), "", 100).is_some());
    assert!(detect(&json!({"error":{"status":401}}), "", 100).is_none());
}
