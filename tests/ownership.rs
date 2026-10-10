#![cfg(unix)]
use agentconductor::process::Ownership;

fn owner(phase: &str, pid: Option<u32>) -> Ownership {
    Ownership {
        coordinator_pid: 1,
        pid,
        process_group: pid.map(|id| id as i32),
        phase: phase.into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    }
}

#[test]
fn unknown_launch_window_is_never_treated_as_stopped() {
    assert!(owner("launching", None).ensure_quiescent().is_err());
    assert!(owner("running", None).ensure_quiescent().is_err());
    assert!(owner("terminal", None).ensure_quiescent().is_err());
    assert!(owner("not_spawned", Some(42)).ensure_quiescent().is_err());
    assert!(owner("terminal", Some(0)).ensure_quiescent().is_err());
}

#[test]
fn live_process_is_rejected_even_when_record_says_terminal() {
    assert!(
        owner("running", Some(std::process::id()))
            .ensure_quiescent()
            .is_err()
    );
    assert!(
        owner("terminal", Some(std::process::id()))
            .ensure_quiescent()
            .is_err()
    );
}

#[test]
fn known_unspawned_or_absent_process_is_quiescent() {
    assert!(owner("not_spawned", None).ensure_quiescent().is_ok());
    assert!(
        owner("running", Some(i32::MAX as u32))
            .ensure_quiescent()
            .is_ok()
    );
}

#[test]
fn legacy_ownership_has_an_unknown_signal_instead_of_an_invented_exit_status() {
    let owner: Ownership = serde_json::from_value(serde_json::json!({"coordinator_pid":1,"pid":null,"process_group":null,"phase":"not_spawned","exit_code":null,"stopped":false,"timed_out":false})).unwrap();
    assert_eq!(owner.termination_signal, None);
    assert_eq!(owner.exit_code, None);
}
