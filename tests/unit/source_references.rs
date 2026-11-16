//! GitHub #1018: Goal references are stored in the one form the reference dialog writes.
use super::*;

#[test]
fn typed_and_rewritten_references_take_the_dialog_form() {
    let goal = "Fix login jira:EMM-52599 and [confluence:53553565], see [Jira:EMM-1].";
    assert_eq!(
        canonical(goal),
        "Fix login [Jira:EMM-52599] and [Confluence:53553565], see [Jira:EMM-1]."
    );
    assert_eq!(
        references(&canonical(goal)),
        ["jira:EMM-52599", "confluence:53553565", "jira:EMM-1"]
    );
}

#[test]
fn text_without_valid_references_is_unchanged() {
    for goal in [
        "no refs here",
        "myjira:EMM-1 is a word",
        "jira:lowercase-1",
        "confluence:abc",
        "한글 [Jira:EMM-2] 끝",
    ] {
        assert_eq!(canonical(goal), goal, "{goal}");
    }
    assert_eq!(canonical(""), "");
}
