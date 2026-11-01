use agentconductor::subprocess::background_fallback_allowed;

#[test]
fn breakaway_fallback_is_allowed_only_outside_a_host_job() {
    assert!(background_fallback_allowed(Some(false)));
    assert!(!background_fallback_allowed(Some(true)));
    assert!(!background_fallback_allowed(None));
}
