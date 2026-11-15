//! GitHub #1009: a run notice names the Phase, each task with its state, and, when the run
//! cannot proceed, the error and the blocking cause.
use agentconductor::messenger::detail::run_detail;
use agentconductor::workflow::Report;
use serde_json::json;

fn report(state: &str, error: Option<&str>, review_error: Option<&str>) -> Report {
    let task = |id: &str, status: &str, error: Option<&str>| {
        json!({"id":id,"status":status,"cwd":"/w","base":"abc","attempt":1,"artifacts":"/r/t","error":error,
        "dependencies":[],"approval_epoch":0,"session_established":false,"retry_not_before":0,"automatic_retries":0,"timeouts":0})
    };
    let plan_task = |id: &str, prompt: &str| json!({"id":id,"agent":"claude","prompt":prompt,"paths":[id],"depends_on":[],"test_command":["npm","test"]});
    serde_json::from_value(json!({
        "schema_version":1,"state":state,"repo":"/w","base":"abc","branch":"refs/heads/main",
        "plan":{"goal":"Lotto","workspace_mode":"shared","agents":[],"tasks":[
            plan_task("t01-docs","Write the draw rules\nin detail"),plan_task("t02-tests","Add failing draw tests"),plan_task("t03-ui","Build the draw screen")]},
        "tasks":[task("t01-docs","completed",None),task("t02-tests","failed",Some("integration test failed: t02-tests")),task("t03-ui","pending",None)],
        "candidate":null,"tests":[],"diff_hash":null,"error":error,"review_error":review_error,"verification_cwd":"/w"
    }))
    .unwrap()
}

#[test]
fn a_phased_run_names_its_phase_and_every_task_with_its_state() {
    let phase = (2, 3, "Implementation".to_string());
    let text = run_detail(&report("running", None, None), Some(&phase), true);
    for needle in [
        "Phase 2/3 · Implementation",
        "작업 1/3 완료",
        "· t01-docs Write the draw rules — 완료",
        "· t02-tests Add failing draw tests — 실패",
        "· t03-ui Build the draw screen — 대기",
    ] {
        assert!(text.contains(needle), "{needle}\n{text}");
    }
    assert!(
        !text.contains("in detail"),
        "only the first prompt line\n{text}"
    );
    assert!(
        !text.contains("오류"),
        "a running run has no error line\n{text}"
    );
}

#[test]
fn a_run_that_cannot_proceed_says_the_error_and_the_blocking_cause() {
    let failed = report(
        "failed",
        Some("integration test failed: t02-tests (expected success)"),
        Some("three coordinator repair rounds did not resolve the failure"),
    );
    let ko = run_detail(&failed, None, true);
    for needle in [
        "오류: integration test failed: t02-tests (expected success)",
        "차단 원인: t02-tests — integration test failed: t02-tests",
        "차단 원인: three coordinator repair rounds",
    ] {
        assert!(ko.contains(needle), "{needle}\n{ko}");
    }
    let en = run_detail(&failed, None, false);
    for needle in [
        "Tasks 1/3 completed",
        "— failed",
        "Error: integration test failed",
        "Blocked by: t02-tests — ",
    ] {
        assert!(en.contains(needle), "{needle}\n{en}");
    }
    assert!(
        !en.contains("Phase"),
        "no Phase line without a linked Phase"
    );
}

#[test]
fn long_errors_are_clipped() {
    let long = "x".repeat(1000);
    let text = run_detail(&report("blocked", Some(&long), None), None, false);
    let line = text.lines().find(|l| l.starts_with("Error: ")).unwrap();
    assert!(line.chars().count() < 320, "{}", line.len());
}
