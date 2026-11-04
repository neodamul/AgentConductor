//! #420: a release may be checked without tagging, publishing or changing repository state.
#[path = "support/release_readiness.rs"]
mod readiness;
use readiness::{Evidence, failures};
fn ready() -> Evidence {
    Evidence {
        cargo: "0.2.0".into(),
        documents: vec![("README.md".into(), Some("0.2.0".into()))],
        release_notes: true,
        open_issues: vec![],
        screenshots: Ok(()),
    }
}
#[test]
fn every_release_blocker_is_reported_and_a_complete_candidate_passes() {
    assert!(failures(&ready()).is_empty());
    let mut bad = ready();
    bad.cargo = "0.1.58".into();
    bad.documents[0].1 = Some("0.1.58".into());
    bad.documents.push(("missing manual".into(), None));
    bad.release_notes = false;
    bad.open_issues = vec![420, 428];
    bad.screenshots = Err("stale captures".into());
    let reasons = failures(&bad);
    assert_eq!(reasons.len(), 6, "{reasons:?}");
    for part in [
        "Cargo",
        "README.md",
        "missing manual",
        "release notes",
        "#420",
        "stale captures",
    ] {
        assert!(
            reasons.iter().any(|r| r.contains(part)),
            "missing {part}: {reasons:?}"
        );
    }
}

#[test]
fn document_markers_and_versioned_sections_are_exact() {
    use readiness::{has_release_notes, marked_version};
    assert_eq!(
        marked_version("Version **0.2.0** · guide", "Version **", "**").as_deref(),
        Some("0.2.0")
    );
    assert_eq!(marked_version("version 0.2.0", "Version **", "**"), None);
    assert!(!has_release_notes("## Unreleased\n- feature\n"));
    assert!(!has_release_notes("## 0.2.0\n\n## 0.1.58\n- old"));
    assert!(has_release_notes(
        "## Unreleased\n\n## [0.2.0] - date\n- shipped"
    ));
}
#[test]
fn missing_files_are_blockers_rather_than_assumed_versions() {
    let root = std::env::temp_dir().join(format!(
        "ac-release-gate-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let evidence = readiness::collect(&root, "0.2.0", vec![], Ok(()));
    assert_eq!(evidence.documents.len(), 5);
    assert!(evidence.documents.iter().all(|(_, v)| v.is_none()));
    assert_eq!(failures(&evidence).len(), 6);
    std::fs::remove_dir(root).unwrap();
}
#[test]
#[ignore = "Requires an actual release candidate and read-only GitHub access; run explicitly before publishing"]
fn milestone_020_release_gate() {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let queried = Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            "neodamul/AgentConductor",
            "--milestone",
            "0.2.0",
            "--state",
            "open",
            "--limit",
            "1000",
            "--json",
            "number",
        ])
        .current_dir(root)
        .output()
        .expect("GitHub CLI is required for milestone verification");
    assert!(
        queried.status.success(),
        "Cannot establish milestone state: {}",
        String::from_utf8_lossy(&queried.stderr)
    );
    let issues: serde_json::Value =
        serde_json::from_slice(&queried.stdout).expect("GitHub must return issue JSON");
    let open = issues
        .as_array()
        .expect("issue array")
        .iter()
        .map(|issue| issue["number"].as_u64().expect("issue number"))
        .collect();
    let python = if cfg!(windows) { "python" } else { "python3" };
    let checked = Command::new(python)
        .arg("scripts/check_screenshots.py")
        .arg(root)
        .current_dir(root)
        .output()
        .expect("existing screenshot verifier starts");
    let screenshots = if checked.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        ))
    };
    let evidence = readiness::collect(root, env!("CARGO_PKG_VERSION"), open, screenshots);
    let reasons = failures(&evidence);
    assert!(
        reasons.is_empty(),
        "Release blocked:\n{}",
        reasons.join("\n")
    );
}

#[test]
fn publishing_020_is_guarded_by_the_readiness_check() {
    let workflow = include_str!("../.github/workflows/release.yml");
    assert!(workflow.contains(
        "cargo test --locked --test release_readiness milestone_020_release_gate -- --ignored"
    ));
    assert!(
        workflow.contains("needs: [verify, screenshots]"),
        "publication must depend on the gate job"
    );
}
