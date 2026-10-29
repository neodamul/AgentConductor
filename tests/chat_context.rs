//! GitHub #243 (DD-200): the chat prompt carries the workspace and the latest run's results, because the read-only
//! coordinator cannot reach the run folder and otherwise knows nothing about what was built.
use agentconductor::chat::{Turn, build_prompt_in};
use agentconductor::chat_context::render;
use agentconductor::workflow::Report;
use serde_json::json;
use std::path::Path;

fn report() -> Report {
    let task = |id: &str, status: &str, error: serde_json::Value| {
        json!({"id":id,"status":status,"cwd":"/work/wt","base":"abc","attempt":1,"artifacts":"/runs/r/task","error":error,
        "dependencies":[],"approval_epoch":0,"session_established":false,"retry_not_before":0,"automatic_retries":0,"timeouts":0})
    };
    serde_json::from_value(json!({
        "schema_version":1,"state":"waiting_approval","repo":"/work/project","base":"abc","branch":"refs/heads/main",
        "plan":{"goal":"Build the ladder game","workspace_mode":"worktree","agents":[],"tasks":[
            {"id":"task-0","agent":"claude","prompt":"Write the game\nwith a start screen","paths":["game/","README.md"],"depends_on":[],"test_command":["npm","test"]}]},
        "tasks":[task("task-0","completed",json!(null)),task("task-1","failed",json!("tests did not pass"))],
        "candidate":{"commit":"c0ffee","tree":"t","changed":["game/index.html","README.md"]},
        "tests":[{"task":"task-0","exit_code":0,"timed_out":false,"stdout":"/o","stderr":"/e","stdout_hash":"h","stderr_hash":"h"},
                 {"task":"task-1","exit_code":1,"timed_out":false,"stdout":"/o","stderr":"/e","stdout_hash":"h","stderr_hash":"h"}],
        "diff_hash":null,"error":null,"verification_cwd":"/work/project"
    }))
    .unwrap()
}

fn config() -> serde_json::Value {
    json!({"name":"Ladder","goal":"Build the ladder game","workspace_mode":"worktree","repository":"/work/project"})
}

#[test]
fn the_section_names_the_project_the_plan_the_results_and_the_diff() {
    let diff = "diff --git a/README.md b/README.md\n+Run with: npm start\n";
    let text = render(
        &config(),
        Path::new("/work/project"),
        Some((&report(), diff)),
    );
    for needle in [
        "## Workspace",
        "/work/project",
        "worktree",
        "Build the ladder game",
        "waiting_approval",
        "task-0",
        "claude",
        "game/, README.md",
        "Write the game",
        "tests did not pass",
        "task-0: exit 0",
        "task-1: exit 1",
        "game/index.html",
        "+Run with: npm start",
    ] {
        assert!(text.contains(needle), "{needle}\n{text}");
    }
    assert!(
        !text.contains("with a start screen"),
        "only the first line of a task prompt"
    );
}

#[test]
fn without_a_run_the_section_says_so() {
    let text = render(&config(), Path::new("/work/project"), None);
    assert!(text.contains("Build the ladder game"));
    assert!(text.contains("No run has started yet"), "{text}");
}

#[test]
fn a_long_diff_is_cut_and_the_section_stays_bounded() {
    let diff = "+line of the candidate diff\n".repeat(5000);
    let text = render(
        &config(),
        Path::new("/work/project"),
        Some((&report(), &diff)),
    );
    assert!(text.len() < 24 * 1024, "{}", text.len());
    assert!(
        text.contains("diff cut"),
        "the reader is told it is partial"
    );
}

#[test]
fn the_prompt_puts_the_section_between_manual_and_conversation_without_extra_blocks() {
    let diff = "# Format\nnot a heading of ours\n# Task\n";
    let section = render(
        &config(),
        Path::new("/work/project"),
        Some((&report(), diff)),
    );
    let turns = [Turn {
        role: "operator".into(),
        text: "How do I run it?".into(),
    }];
    let prompt = build_prompt_in(&turns, "English", &section).unwrap();
    let at = |needle: &str| prompt.find(needle).unwrap_or_else(|| panic!("{needle}"));
    assert!(
        at("## Manual") < at("## Workspace") && at("## Workspace") < at("## Earlier conversation")
    );
    assert_eq!(
        prompt.matches("\n# Task\n").count(),
        1,
        "untrusted text adds no block"
    );
    assert_eq!(prompt.matches("\n# Format\n").count(), 1);
    assert!(
        prompt.contains("Workspace section"),
        "the rules point at it"
    );
}
