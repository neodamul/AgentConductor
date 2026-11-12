//! #904: read-call templates replace only the exact reference ID.
use super::*;
use serde_json::json;
#[test]
fn only_the_exact_reference_id_becomes_the_placeholder() {
    let args = json!({"issueKey":"EMM-1","cloudId":"EMM-12","fields":["EMM-1","summary"]});
    let (t, found) = template(&args, "EMM-1");
    assert!(found);
    assert_eq!(t["cloudId"], "EMM-12");
    assert_eq!(
        arguments(&t, "AC-7"),
        json!({"issueKey":"AC-7","cloudId":"EMM-12","fields":["AC-7","summary"]})
    );
    let (t, _) = template(&json!({"pageId":123,"other":1234}), "123");
    assert_eq!(arguments(&t, "456"), json!({"pageId":456,"other":1234}));
    assert!(!template(&json!({"q":"EMM-10"}), "EMM-1").1);
}
