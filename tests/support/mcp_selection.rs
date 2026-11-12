use super::Selection;
use serde_json::{Value, json};
#[test]
fn inspect_accepts_only_harmless_arguments() {
    for arguments in [None, Some(Value::Null), Some(json!({}))] {
        let mut reply = json!({"action":"inspect","server":"docs","tool":"getIssue"});
        if let Some(arguments) = arguments {
            reply["arguments"] = arguments;
        }
        assert!(serde_json::from_value::<Selection>(reply).is_ok());
    }
    assert!(
        serde_json::from_value::<Selection>(json!({"action":"inspect","server":"docs"})).is_ok()
    );
    for arguments in [
        json!({"issueKey":"AC-1"}),
        json!([]),
        json!(""),
        json!(false),
        json!(0),
    ] {
        assert!(
            serde_json::from_value::<Selection>(
                json!({"action":"inspect","server":"docs","arguments":arguments})
            )
            .is_err()
        );
    }
}
#[test]
fn other_actions_and_unknown_fields_remain_strict() {
    for action in ["read", "helper"] {
        let mut reply = json!({"action":action,"server":"docs","tool":"getIssue"});
        assert!(serde_json::from_value::<Selection>(reply.clone()).is_err());
        reply["arguments"] = json!({"issueKey":"AC-1"});
        assert!(serde_json::from_value::<Selection>(reply).is_ok());
    }
    for reply in [
        json!({"action":"inspect","server":"docs","unexpected":true}),
        json!({"action":"read","server":"docs","tool":"getIssue","arguments":{},"unexpected":true}),
        json!({"action":"unavailable","reason":"No server","arguments":{}}),
        json!({"action":"execute","server":"docs","arguments":{}}),
    ] {
        assert!(serde_json::from_value::<Selection>(reply).is_err());
    }
}
