use super::*;

#[test]
#[ignore = "requires an explicitly configured existing local agentmemory daemon"]
fn configured_daemon_round_trip_preserves_local_execution_authority() {
    use agentconductor::memory::{
        Client,
        checkpoints::{self, Envelope},
    };
    let port = std::env::var("AC_MEMORY_TEST_PORT")
        .expect("set AC_MEMORY_TEST_PORT explicitly")
        .parse()
        .unwrap();
    let client = Client::from_environment(port).unwrap();
    let (root, mut store, report) = setup();
    let now = || {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    };
    let issued_at = now();
    let mut request = issue(&report);
    request.issued_at_ms = issued_at;
    request.expires_at_ms = issued_at + 60000;
    let directive = cooperative::issue(&mut store, request, issued_at).unwrap();
    let directive_packet = Envelope::Directive {
        directive: directive.clone(),
    };
    let published = client.publish(&directive_packet).unwrap();
    assert_eq!(client.read(&published.memory_id).unwrap(), directive_packet);
    let mut accepted = ack(&directive, AckState::Accepted);
    accepted.reported_at_ms = now();
    let receipt = client
        .publish(&Envelope::Acknowledgment {
            acknowledgment: accepted,
        })
        .unwrap();
    assert_eq!(
        checkpoints::import(&mut store, &client, &receipt.memory_id, now()).unwrap()["applied"],
        true
    );
    let progress = cooperative::Progress {
        address: directive.address.clone(),
        sequence: 1,
        reported_at_ms: now(),
        phase: "verification fixture".into(),
        summary: "Existing daemon delivered addressed checkpoint observations".into(),
        next_step: "Verify local authority remains unchanged".into(),
    };
    let progress_receipt = client.publish(&Envelope::Progress { progress }).unwrap();
    assert_eq!(
        checkpoints::import(&mut store, &client, &progress_receipt.memory_id, now()).unwrap()["applied"],
        true
    );
    assert_eq!(
        checkpoints::import(&mut store, &client, &progress_receipt.memory_id, now()).unwrap()["applied"],
        false
    );
    assert_eq!(
        store.document("workflow").unwrap(),
        serde_json::to_value(&report).unwrap()
    );
    let bundle = agentconductor::briefing::collect(&root, now()).unwrap();
    assert_eq!(
        bundle["sessions"][0]["memory_report"]["source"],
        "agentmemory exact-ID adapter"
    );
    assert_eq!(
        bundle["sessions"][0]["memory_report"]["freshness"],
        "recent"
    );
    // DD-44: a later report is found without its memory ID; earlier ones are not re-applied.
    let later = cooperative::Progress {
        address: directive.address.clone(),
        sequence: 2,
        reported_at_ms: now(),
        phase: "discovery fixture".into(),
        summary: "Found by scoped discovery".into(),
        next_step: "Nothing".into(),
    };
    client
        .publish(&Envelope::Progress { progress: later })
        .unwrap();
    let found = checkpoints::discover(&mut store, &client, &directive.address, now()).unwrap();
    assert_eq!(found["imported"], 1, "{found}");
    assert_eq!(found["already_imported"], 2, "{found}");
    assert_eq!(found["skipped_directives"], 1, "{found}");
    let synced = agentconductor::memory::sync(&store, port).unwrap();
    assert!(synced > 0);
    assert!(store.pending_memory().unwrap().is_empty());
    println!(
        "{}",
        serde_json::json!({"verification":"existing daemon only; workflow is a controlled fixture","run_id":directive.address.run_id,"directive_memory":published.memory_id,"ack_memory":receipt.memory_id,"progress_memory":progress_receipt.memory_id,"authority_unchanged":true,"duplicate_applied":false})
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
