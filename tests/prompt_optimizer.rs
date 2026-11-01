//! GitHub #180: the bundled prompt-optimizer skill builds the coordinator prompt and bounds its reply.
use agentconductor::prompt_optimizer::{MAX_GOAL_BYTES, build_prompt, proposal_from};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn vendored(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("third_party/prompt-optimizer")
        .join(name)
}

#[test]
fn the_prompt_carries_the_unmodified_skill_references_goal_and_language() {
    let prompt = build_prompt("Make the checkout faster", "Korean").unwrap();
    for file in [
        "SKILL.md",
        "references/core-patterns.md",
        "references/meta-optimization-loop.md",
        "references/model-family-notes.md",
        "references/transformed-examples.md",
    ] {
        let text = std::fs::read_to_string(vendored(file)).unwrap();
        assert!(prompt.contains(&text), "{file} is embedded as is");
    }
    assert!(prompt.contains("Make the checkout faster"));
    assert!(prompt.contains("Reply in Korean"));
    assert!(prompt.contains("read-only"));
}

#[test]
fn an_empty_or_oversized_goal_is_refused_before_anything_runs() {
    assert!(build_prompt("  \n ", "English").is_err());
    assert!(build_prompt(&"a".repeat(MAX_GOAL_BYTES + 1), "English").is_err());
    assert!(build_prompt(&"a".repeat(MAX_GOAL_BYTES), "English").is_ok());
}

#[test]
fn the_reply_becomes_a_bounded_proposal() {
    let long = "é".repeat(20_000);
    let reply = json!({
        "optimized": format!("  {long}  "),
        "changes": (0..20).map(|n| format!("change {n} {}", "x".repeat(500))).collect::<Vec<_>>(),
        "questions": ["Which env?", 7, "  ", "Who signs off?"]
    });
    let proposal = proposal_from(&reply).unwrap();
    assert!(proposal.optimized.len() <= 16 * 1024 && proposal.optimized.starts_with('é'));
    assert_eq!(proposal.changes.len(), 12);
    assert!(proposal.changes.iter().all(|c| c.chars().count() <= 401));
    assert_eq!(proposal.questions, ["Which env?", "Who signs off?"]);
    let minimal = proposal_from(&json!({"optimized":"Goal"})).unwrap();
    assert!(minimal.changes.is_empty() && minimal.questions.is_empty());
    for bad in [json!({}), json!({"optimized":"  "}), json!({"optimized":3})] {
        assert!(proposal_from(&bad).is_err(), "{bad}");
    }
}

#[test]
fn the_vendored_files_match_the_recorded_origin_sums() {
    let source = std::fs::read_to_string(vendored("SOURCE.md")).unwrap();
    assert!(source.contains("d18b7aa8ba878354e5c348310230e652f7690f9c"));
    let mut checked = 0;
    for line in source.lines().filter(|l| l.starts_with("    ")) {
        let (name, sum) = line.trim().split_once("  ").unwrap();
        let digest = Sha256::digest(std::fs::read(vendored(name)).unwrap());
        assert_eq!(format!("{digest:x}"), sum, "{name}");
        checked += 1;
    }
    assert_eq!(checked, 6, "skill, license and four references");
}
