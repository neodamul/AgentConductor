pub fn api_worker() {
    println!("{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"api-limit-fixture\"}}");
    println!(
        "{{\"api_error\":\"usage_limit_reached\",\"api_error_status\":429,\"duration_api_ms\":645143,\"duration_ms\":175665,\"fast_mode_disabled_reason\":\"sdk_opt_in_required\",\"fast_mode_state\":\"off\"}}"
    );
    std::process::exit(1);
}
