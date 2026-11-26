use super::*;
use crate::mcp::references;

#[test]
fn english_rename_and_collision_keep_ids_on_an_international_goal() {
    let refs = references("[Jira:EMM-1102] [Confluence:185]").unwrap();
    let old = with_references("goal_20261126", &refs);
    let collision = unique(&old, std::slice::from_ref(&old));
    assert_eq!(
        collision,
        "goal_20261126_2__refs_jira_emm_1102_confluence_185"
    );
    assert!(is_placeholder(&collision));
    let name = renamed(&collision, "document_preview");
    assert_eq!(name, "document_preview__refs_jira_emm_1102_confluence_185");
    assert!(!is_placeholder(&name));
    assert!(validate_base(&name).is_err());
    assert!(validate_base("document_preview").is_ok());
    assert!(validate(&name).is_ok());
}

#[test]
fn a_long_reference_set_gets_a_compact_name_and_keeps_a_complete_id() {
    let ids = (1..=5)
        .map(|n| format!("confluence:{n:032}"))
        .collect::<Vec<_>>()
        .join(" ");
    let refs = references(&ids).unwrap();
    let base = "a".repeat(48);
    let original = with_references(&base, &refs);
    assert!(original.len() <= 96);
    assert!(original.contains(&refs[0].id));
    assert!(original.contains("_more_"));
    let collision = unique(&original, std::slice::from_ref(&original));
    assert!(collision.len() <= 96);
    assert!(validate(&collision).is_ok());
    assert_ne!(collision, original);
    let alternate = references(&ids.replace(&refs[4].id, &format!("{:032}", 6))).unwrap();
    assert_ne!(original, with_references(&base, &alternate));

    let long_id = references(&format!("jira:EMM-{}", "1".repeat(220))).unwrap();
    assert!(with_references("goal", &long_id).len() <= 96);
}
