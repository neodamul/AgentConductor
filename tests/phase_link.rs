#![cfg(unix)]
//! DD-62: a phase run moves its task files to verification when tests pass and to
//! completed after the human-approved merge.
use agentconductor::{phases, slug_progress::progress};
use std::{path::Path, process::Command};
#[path = "support/workflow.rs"]
mod support;

fn cli(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn run_and_merge_advance_the_phase_tree() {
    let (root, plan) = support::setup("shared");
    let slug = root.join("slug");
    let specs: Vec<_> = plan
        .tasks
        .iter()
        .map(|t| {
            let mut t = t.clone();
            t.id = t.id.trim_end_matches(".txt").into();
            t
        })
        .collect();
    phases::write(
        &slug,
        "# Analysis\n",
        &[phases::PhaseSpec {
            id: "01-core".into(),
            name: "core".into(),
            tasks: specs,
        }],
    )
    .unwrap();
    let run_plan = phases::run_plan(&slug, "01-core", &plan).unwrap();
    let file = root.join("phase-plan.json");
    std::fs::write(&file, serde_json::to_vec(&run_plan).unwrap()).unwrap();
    let digest = serde_json::from_slice::<serde_json::Value>(
        &cli(&["validate".as_ref(), file.as_os_str()]).stdout,
    )
    .unwrap()["plan_digest"]
        .as_str()
        .unwrap()
        .to_string();
    let run = root.join("run");
    let output = cli(&[
        "run".as_ref(),
        file.as_os_str(),
        root.join("repo").as_os_str(),
        run.as_os_str(),
        digest.as_ref(),
        "--phase".as_ref(),
        slug.as_os_str(),
        "01-core".as_ref(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        progress(&slug)["phases"][0]["stage"],
        "verification",
        "tests passed → verification"
    );
    let report = agentconductor::workflow::load(&run).unwrap();
    let sha = report.candidate.unwrap().commit;
    assert!(
        cli(&["approve".as_ref(), run.as_os_str(), sha.as_ref()])
            .status
            .success()
    );
    let merged = cli(&["merge".as_ref(), run.as_os_str()]);
    assert!(
        merged.status.success(),
        "{}",
        String::from_utf8_lossy(&merged.stderr)
    );
    let state = progress(&slug);
    assert_eq!(state["stage"], "completed", "{state}");
    assert!(state["mismatches"].as_array().unwrap().is_empty());
    assert!(!Path::new(&slug.join("verification/01-core/one.md")).exists());
    std::fs::remove_dir_all(root).unwrap();
}
