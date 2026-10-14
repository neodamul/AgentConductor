use agentconductor::{
    protocol::{Decoder, EventKind, Provider},
    quota,
};
use serde_json::json;
#[test]
fn claude_api_usage_limit_json_is_an_error_and_preserves_only_explicit_reset() {
    let now = agentconductor::availability::parse_time("2026-10-14T01:50:00+09:00").unwrap();
    for typed in [false, true] {
        let mut payload = json!({"api_error":"usage_limit_reached","api_error_status":429,"duration_api_ms":645143,"duration_ms":175665,"fast_mode_disabled_reason":"sdk_opt_in_required","fast_mode_state":"off"});
        if typed {
            payload["type"] = json!("result");
        }
        let mut decoder = Decoder::new(Provider::Claude, 4096);
        let events = decoder
            .push(format!("{}{}", payload, char::from(10)).as_bytes())
            .unwrap();
        assert_eq!(events[0].kind, EventKind::Error);
        let s = quota::parse_claude_event(&payload, now).unwrap();
        assert!(s.windows.is_empty(), "429 gives no exact utilization");
        let mut store = agentconductor::state::Store::open(std::env::temp_dir().join(format!(
            "ac-api-limit-{}.sqlite",
            agentconductor::native_guard::session_id().unwrap()
        )))
        .unwrap();
        let profile = serde_json::from_value(
            json!({"id":"claude","executable":"claude","args":[],"parser":"claude"}),
        )
        .unwrap();
        quota::persist(&mut store, &profile, Ok(s)).unwrap();
        let view = quota::stored(&store, &profile, now).unwrap();
        let restriction = quota::restriction(&view, now)
            .expect("API limit reserves account even without utilization");
        assert_eq!(restriction.available_at, None);
        payload["retry_at"] = json!("2026-10-14T02:50:00+09:00");
        let s = quota::parse_claude_event(&payload, now).unwrap();
        quota::persist(&mut store, &profile, Ok(s)).unwrap();
        let view = quota::stored(&store, &profile, now).unwrap();
        assert_eq!(
            quota::restriction(&view, now).unwrap().available_at,
            Some(now + 3600)
        );
    }
    assert!(
        quota::parse_claude_event(
            &json!({"api_error":"authentication_failed","api_error_status":401}),
            now
        )
        .is_err()
    );
}
