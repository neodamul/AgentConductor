#![cfg(unix)]
use agentconductor::cooperative::{self, AckState, Acknowledgment};
#[path = "support/cooperative.rs"]
mod cooperative_support;
#[path = "support/workflow.rs"]
mod support;
use cooperative_support::{issue, setup};

#[test]
fn company_cli_packet_commands_use_the_same_ledger_without_a_ui_server() {
    let (root, store, report) = setup();
    drop(store);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut request = issue(&report);
    request.issued_at_ms = now;
    request.expires_at_ms = now + 60000;
    let packet = root.join("packet.json");
    std::fs::write(&packet, serde_json::to_vec(&request).unwrap()).unwrap();
    let exe = env!("CARGO_BIN_EXE_agentconductor");
    let call = |command: &str| {
        std::process::Command::new(exe)
            .arg(command)
            .arg(&root)
            .arg(&packet)
            .output()
            .unwrap()
    };
    let issued = call("checkpoint-issue");
    assert!(
        issued.status.success(),
        "{}",
        String::from_utf8_lossy(&issued.stderr)
    );
    let record: cooperative::Directive = serde_json::from_slice(&issued.stdout).unwrap();
    std::fs::write(&packet, serde_json::to_vec(&record.address).unwrap()).unwrap();
    let polled = call("checkpoint-poll");
    assert!(polled.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&polled.stdout).unwrap()[0]["id"],
        record.id
    );
    let accepted = Acknowledgment {
        id: record.id.clone(),
        address: record.address.clone(),
        state: AckState::Accepted,
        reported_at_ms: now,
        reason: "Company CLI reached checkpoint".into(),
    };
    std::fs::write(&packet, serde_json::to_vec(&accepted).unwrap()).unwrap();
    let acked = call("checkpoint-ack");
    assert!(acked.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&acked.stdout).unwrap()["state"],
        "accepted"
    );
    let progress = cooperative::Progress {
        address: record.address.clone(),
        sequence: 1,
        reported_at_ms: now,
        phase: "verification".into(),
        summary: "Company CLI reached a safe checkpoint".into(),
        next_step: "Wait for the reviewed directive".into(),
    };
    std::fs::write(&packet, serde_json::to_vec(&progress).unwrap()).unwrap();
    let reported = call("checkpoint-report");
    assert!(reported.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&reported.stdout).unwrap(),
        true
    );
    std::fs::write(&packet, vec![b' '; 16385]).unwrap();
    assert!(!call("checkpoint-poll").status.success());
    std::fs::remove_dir_all(root).unwrap();
}
