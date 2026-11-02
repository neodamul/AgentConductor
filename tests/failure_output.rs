//! GitHub #323: a failed integration test shows both streams, so a stderr warning cannot hide the stdout failure.
use agentconductor::ui::failure_output::failure_output;

#[test]
fn a_stderr_warning_does_not_hide_the_stdout_failure() {
    let out = failure_output(
        "[UNDICI-EHPA] Warning: EnvHttpProxyAgent is experimental",
        "Error: Cannot find module 'tests'\ncode: 'MODULE_NOT_FOUND'\n# fail 1",
    );
    assert!(out.contains("EnvHttpProxyAgent"), "{out}");
    assert!(out.contains("MODULE_NOT_FOUND"), "{out}");
    assert!(out.contains("stderr:") && out.contains("stdout:"), "{out}");
    assert!(
        out.find("stderr:") < out.find("stdout:"),
        "stdout comes last: {out}"
    );
}

#[test]
fn one_stream_stays_plain_and_no_stream_is_empty() {
    assert_eq!(failure_output("", "boom\n"), "boom");
    assert_eq!(failure_output("bad\n", "  \n"), "bad");
    assert_eq!(failure_output("", ""), "");
}

#[test]
fn a_long_stream_keeps_its_last_lines_and_never_hides_the_other() {
    let long: String = (1..=100).map(|n| format!("line {n}\n")).collect();
    let out = failure_output(&long, "the real failure");
    assert!(
        out.contains("the real failure") && out.contains("line 100"),
        "{out}"
    );
    assert!(
        !out.contains("line 1\n") && out.contains("earlier lines"),
        "truncation is said: {out}"
    );
    let out = failure_output("a warning", &long);
    assert!(
        out.contains("a warning") && out.contains("line 100"),
        "{out}"
    );
    assert!(out.lines().count() < 30, "{out}");
}
