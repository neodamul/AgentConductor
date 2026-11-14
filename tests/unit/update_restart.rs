use super::{command, execute};
use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

#[test]
fn mac_bundle_restart_uses_launch_services_and_explicit_cleanup_handoff() {
    let exe = Path::new("/Applications/Agent Conductor.app/Contents/MacOS/agentconductor");
    let args = vec![
        OsString::from("ui-new"),
        OsString::from("/tmp/work space/한글"),
    ];
    let (cmd, bundled) = command(exe, &args, 1234, true);
    assert!(bundled);
    assert_eq!(cmd.get_program(), OsStr::new("/usr/bin/open"));
    let actual: Vec<_> = cmd.get_args().map(OsStr::to_owned).collect();
    let expected: Vec<OsString> = [
        "-n",
        "-a",
        "/Applications/Agent Conductor.app",
        "--env",
        "AGENTCONDUCTOR_UPDATE_WAIT_PID=1234",
        "--args",
        "ui-new",
        "/tmp/work space/한글",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    assert_eq!(actual, expected);
}

#[test]
fn a_bare_executable_keeps_native_arguments_and_wait_pid() {
    for macos in [true, false] {
        let exe = Path::new("/tmp/agentconductor");
        let args = vec![OsString::from("ui-new"), OsString::from("/tmp/work space")];
        let (cmd, bundled) = command(exe, &args, 42, macos);
        assert!(!bundled);
        assert_eq!(cmd.get_program(), exe);
        assert_eq!(
            cmd.get_args().collect::<Vec<_>>(),
            args.iter().map(OsString::as_os_str).collect::<Vec<_>>()
        );
        assert!(
            cmd.get_envs().any(
                |(k, v)| k == OsStr::new(super::super::WAIT_PID) && v == Some(OsStr::new("42"))
            )
        );
    }
}

#[test]
fn a_failed_launch_keeps_the_caller_alive() {
    let missing = Command::new("/no-such-agentconductor-updater-command");
    assert!(execute(missing, true).is_err());
}

#[test]
fn an_unsuccessful_bundle_handoff_is_not_reported_as_success() {
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .arg("--not-a-test-runner-option")
        .stderr(std::process::Stdio::null());
    assert!(execute(child, true).is_err());
}

use std::process::Command;
