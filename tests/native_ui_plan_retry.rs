use agentconductor::native_ui_probe::workspace::{can_retry_plan, plan_failure_is_current};
use serde_json::json;

#[test]
fn failed_plan_retry_is_available_after_a_request_error() {
    let failed = json!({"job":{"kind":"plan","state":"failed"}});
    assert!(can_retry_plan(&failed, true, false, false));
    assert!(!plan_failure_is_current(&failed, false, true));
}

#[test]
fn failed_plan_retry_requires_saved_settings_and_no_pending_request() {
    let failed = json!({"job":{"kind":"plan","state":"failed"}});
    assert!(!can_retry_plan(&failed, false, false, false));
    assert!(!can_retry_plan(&failed, true, true, false));
    assert!(!can_retry_plan(&failed, true, false, true));
    assert!(!can_retry_plan(
        &json!({"job":{"kind":"start","state":"failed"}}),
        true,
        false,
        false
    ));
}

#[test]
fn timeout_notice_uses_automatic_recovery_in_both_locales() {
    for locale in ["en", "ko"] {
        let path = format!("src/ui/messages/{locale}.json");
        let messages: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let notice = messages["startup.native_response_timed_out_refresh_before_retrying"]
            .as_str()
            .unwrap();
        assert!(notice.contains("automatically") || notice.contains("자동 확인"));
        assert!(notice.contains("reopen") || notice.contains("다시 열어"));
        assert!(!notice.contains("refresh before retrying"));
        assert!(!notice.contains("새로고침하세요"));
    }
}
