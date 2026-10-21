use agentconductor::release_notes::section;
use std::path::Path;

fn normalize_lines(value: &str) -> String {
    value.replace("\r\n", "\n")
}

#[test]
fn extracts_only_the_requested_tag_section() {
    assert_eq!(
        section("## v1\n- first\n\n## v2\n- second\n", "v1").unwrap(),
        "- first\n"
    );
}

#[test]
fn rejects_missing_or_empty_sections() {
    assert!(section("## v1\n- note\n", "v2").is_err());
    assert!(section("## v1\n\n## v2\n- note\n", "v1").is_err());
}

#[test]
fn release_workflow_uses_the_rust_cli() {
    let workflow = include_str!("../.github/workflows/release.yml");
    assert!(workflow.contains("cargo run --release --locked -- release notes"));
    assert!(!workflow.contains("scripts/release_notes.py"));
}

#[test]
fn stable_release_contract_is_verified_in_rust() {
    let release = include_str!("../.github/workflows/release.yml");
    let validation = normalize_lines(include_str!("../.github/workflows/platform-validation.yml"));
    assert!(release.contains("tags: [\"v*\"]"));
    assert!(!release.contains("branches:"));
    assert!(validation.contains("workflow_call:"));
    assert!(release.contains("uses: ./.github/workflows/platform-validation.yml"));
    let publish = &release[release.find("  publish:").unwrap()..];
    assert!(publish.contains("needs: verify"));
    assert!(publish.contains("contents: write"));
    assert!(publish.contains("gh release create"));
    assert!(!publish.contains("--prerelease"));
    assert!(publish.contains("SHA256SUMS") && publish.contains("*.zip"));
    for line in [release, validation.as_str()]
        .into_iter()
        .flat_map(str::lines)
    {
        let Some(action) = line.trim().strip_prefix("uses: ") else {
            continue;
        };
        if action.starts_with("./") {
            continue;
        }
        let revision = action
            .rsplit_once('@')
            .unwrap()
            .1
            .split('#')
            .next()
            .unwrap()
            .trim();
        assert_eq!(revision.len(), 40, "{action}");
        assert!(
            revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "{action}"
        );
    }
    assert!(!Path::new("tests/test_release_workflow.py").exists());
}

#[test]
fn platform_validation_keeps_native_ui_regressions_in_the_cross_platform_job() {
    let validation = normalize_lines(include_str!("../.github/workflows/platform-validation.yml"));
    let triggers = &validation[..validation.find("\npermissions:").unwrap()];
    assert!(triggers.contains("workflow_dispatch:"));
    assert!(triggers.contains("workflow_call:"));
    assert!(triggers.contains("\n  push:"));
    assert!(triggers.contains("\n  pull_request:"));
    let ui_tests = validation
        .split("- name: Validate opt-in native Rust UI launch path")
        .nth(1)
        .and_then(|step| step.split("\n      - name:").next())
        .unwrap_or("");
    assert!(ui_tests.contains("run: |\n"));
    for target in [
        "native_ui_launch",
        "native_ui_probe",
        "native_room_render",
        "native_ui_profiles",
        "native_ui_roles",
        "native_ui_tasks",
        "native_ui_pending",
        "native_ui_activity",
        "native_ui_tool_dialog",
        "native_ui_cache",
        "native_ui_workspaces",
        "native_workspace_goal_analysis_layout",
        "native_workspace_phase_popup_layout",
        "native_workspace_phase_progress",
    ] {
        assert!(ui_tests.contains(&format!("--test {target}")), "{target}");
    }
}

#[test]
fn workflow_contract_normalizes_windows_line_endings() {
    let workflow = normalize_lines("run: |\r\n  cargo test\r\n");
    assert!(workflow.contains("run: |\n"));
}

#[test]
fn preserves_utf8_notes() {
    assert_eq!(
        section("## v1\n- usage ≈0.1 % → done\n", "v1").unwrap(),
        "- usage ≈0.1 % → done\n"
    );
}

#[test]
fn repository_retains_every_initial_preview_release_section() {
    let changelog = include_str!("../CHANGELOG.md");
    for tag in [
        "v0.1.0-preview.1",
        "v0.1.0-preview.2",
        "v0.1.0-preview.3",
        "v0.1.0-preview.4",
        "v0.1.0-preview.5",
        "v0.1.0-preview.6",
        "v0.1.0",
    ] {
        assert!(!section(changelog, tag).unwrap().trim().is_empty(), "{tag}");
    }
}
