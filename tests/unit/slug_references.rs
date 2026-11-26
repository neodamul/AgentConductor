use super::*;
use crate::mcp::references;

#[test]
fn english_rename_and_collision_keep_ids_on_an_international_goal() {
    let refs = references("[Jira:EMM-1102] [Confluence:185]").unwrap();
    let old = with_references("goal_20261126", &refs).unwrap();
    let collision = unique(&old, std::slice::from_ref(&old));
    assert_eq!(
        collision,
        "goal_20261126_2__refs_jira_emm_1102_confluence_185"
    );
    assert!(is_placeholder(&collision));
    let name = renamed(&collision, "document_preview");
    assert_eq!(name, "document_preview__refs_jira_emm_1102_confluence_185");
    assert!(!is_placeholder(&name));
    assert!(validate(&name).is_ok());
}

#[test]
fn long_reference_names_do_not_truncate_ids() {
    let ids = (1..=5)
        .map(|n| format!("confluence:{n:032}"))
        .collect::<Vec<_>>()
        .join(" ");
    let refs = references(&ids).unwrap();
    let original = with_references(&"a".repeat(48), &refs).unwrap();
    let collision = unique(&original, std::slice::from_ref(&original));
    for r in &refs {
        assert!(collision.contains(&r.id));
    }
    assert!(validate(&collision).is_ok());
    let too_long = references(&format!("jira:EMM-{}", "1".repeat(220))).unwrap();
    assert!(with_references("goal", &too_long).is_err());
}
