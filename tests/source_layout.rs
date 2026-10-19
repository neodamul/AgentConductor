use std::path::{Path, PathBuf};

const CURRENT_CHANGE_FILES: &[&str] = &[
    "src/commands/execution.rs",
    "src/coordinator.rs",
    "src/coordinator/document.rs",
    "src/plan.rs",
    "src/plan/profile.rs",
    "src/planner.rs",
    "src/planner/runner.rs",
    "src/progress.rs",
    "src/protocol.rs",
    "src/quota.rs",
    "src/quota/custom.rs",
    "src/quota/parse.rs",
    "src/quota/query.rs",
    "src/quota/refresh.rs",
    "src/ui/setup.rs",
    "src/ui/setup/metadata.rs",
    "src/ui/setup/quota.rs",
    "src/ui/setup/runtime.rs",
    "src/workflow/execution/worker.rs",
    "src/workflow/recovery.rs",
    "src/workflow/recovery/resume_due.rs",
    "src/native_ui_probe/room.rs",
    "src/native_ui_probe/room/decisions.rs",
    "src/native_ui_probe/room/finalization.rs",
    "src/native_ui_probe/room/lifecycle.rs",
    "src/native_ui_probe/room/refresh.rs",
    "tests/custom_profile.rs",
    "tests/custom_quota_metadata.rs",
    "tests/plan.rs",
    "tests/planning_overlap_repair.rs",
    "tests/support/overlap_repair_agent.rs",
    "tests/source_layout.rs",
    "tests/native_room_finalization.rs",
];

fn source_files(root: &Path, result: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            source_files(&path, result);
        // src/ui/messages/*.json are locale data, exempt from the code line limit.
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs" | "py" | "js" | "css" | "html")
        ) {
            result.push(path);
        }
    }
}

#[test]
fn source_and_test_files_are_bounded_and_test_bodies_are_separate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["src", "tests", "scripts", "examples"] {
        let directory = root.join(directory);
        if directory.is_dir() {
            source_files(&directory, &mut files);
        }
    }
    if root.join("build.rs").is_file() {
        files.push(root.join("build.rs"));
    }
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let relative = path.strip_prefix(root).unwrap().display();
        let lines = text.lines().count();
        if lines > 500 {
            violations.push(format!("{relative}: {lines} lines (maximum 500)"));
        }
        if path.starts_with(root.join("src"))
            && (text.contains("#[test]") || text.contains("#[tokio::test"))
        {
            violations.push(format!("{relative}: test body belongs under tests/"));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn native_ui_probe_facade_stays_under_300_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native_ui_probe.rs");
    let lines = std::fs::read_to_string(path).unwrap().lines().count();
    assert!(
        lines <= 300,
        "src/native_ui_probe.rs has {lines} lines (maximum 300)"
    );
}

#[test]
fn changed_rust_files_stay_under_300_lines() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let violations: Vec<_> = CURRENT_CHANGE_FILES
        .iter()
        .filter_map(|relative| {
            let path = root.join(relative);
            let lines = std::fs::read_to_string(&path).unwrap().lines().count();
            (lines > 300).then(|| format!("{relative}: {lines} lines (maximum 300)"))
        })
        .collect();
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
