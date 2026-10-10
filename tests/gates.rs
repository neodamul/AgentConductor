use agentconductor::gates::{Context, Decision, Ledger, ToolRequest};
use serde_json::json;
use std::path::PathBuf;

fn setup() -> (PathBuf, Ledger, Context) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ac-gates-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let ledger = Ledger::open(&dir.join("state.sqlite")).unwrap();
    let context = Context {
        epoch: 0,
        task: "implementation".into(),
        session: "native-session-1".into(),
        workspace: "/workspace".into(),
        plan_sha256: "approved-plan".into(),
        workspace_sha256: "frozen-tree".into(),
    };
    (dir, ledger, context)
}

fn request() -> ToolRequest {
    ToolRequest {
        tool: "Bash".into(),
        input: json!({"command":"git push origin feature"}),
        reason: "External transmission requires approval".into(),
    }
}

#[test]
fn active_receipts_remain_addressable_after_the_listing_limit_and_detect_tampering() {
    let (dir, mut ledger, mut context) = setup();
    let oldest = ledger.request(&context, &request(), 100).unwrap();
    for epoch in 1..=1001 {
        context.epoch = epoch;
        ledger
            .request(&context, &request(), 100 + epoch as i64)
            .unwrap();
    }
    assert!(!ledger.requests().unwrap().iter().any(|r| r.id == oldest));
    assert_eq!(ledger.receipt(&oldest).unwrap().unwrap().context.epoch, 0);
    assert!(ledger.receipt("missing").unwrap().is_none());
    let db = rusqlite::Connection::open(dir.join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE tool_requests SET payload='{}' WHERE id=?1",
        [&oldest],
    )
    .unwrap();
    assert!(ledger.receipt(&oldest).is_err());
    drop((ledger, db));
    cleanup(dir);
}

#[test]
fn no_implicit_approval_expiry_and_single_consumption_survive_reopen() {
    let (dir, mut ledger, context) = setup();
    let id = ledger.request(&context, &request(), 100).unwrap();
    assert_eq!(
        ledger.consume(&id, &context, &request(), 101).unwrap(),
        Decision::Pending
    );
    assert!(
        ledger
            .decide(&id, "wrong-hash", true, &context, 101, 201)
            .is_err()
    );
    ledger.decide(&id, &id, true, &context, 101, 201).unwrap();
    drop(ledger);
    let mut ledger = Ledger::open(&dir.join("state.sqlite")).unwrap();
    assert_eq!(
        ledger.consume(&id, &context, &request(), 200).unwrap(),
        Decision::Allowed
    );
    assert_eq!(
        ledger.consume(&id, &context, &request(), 200).unwrap(),
        Decision::Denied
    );
    let mut other = request();
    other.input = json!({"command":"git push origin main"});
    let expired = ledger.request(&context, &other, 100).unwrap();
    ledger
        .decide(&expired, &expired, true, &context, 101, 201)
        .unwrap();
    assert_eq!(
        ledger.consume(&expired, &context, &other, 201).unwrap(),
        Decision::Denied
    );
    drop(ledger);
    cleanup(dir);
}

#[test]
fn changed_workspace_session_action_and_explicit_rejection_never_allow_execution() {
    let (dir, mut ledger, context) = setup();
    let id = ledger.request(&context, &request(), 100).unwrap();
    let mut changed = context.clone();
    changed.workspace_sha256 = "changed-tree".into();
    assert!(ledger.decide(&id, &id, true, &changed, 101, 201).is_err());
    ledger.decide(&id, &id, true, &context, 101, 201).unwrap();
    assert_eq!(
        ledger.consume(&id, &changed, &request(), 102).unwrap(),
        Decision::Denied
    );
    changed = context.clone();
    changed.session = "another-session".into();
    assert_eq!(
        ledger.consume(&id, &changed, &request(), 102).unwrap(),
        Decision::Denied
    );
    let mut changed_action = request();
    changed_action.input = json!({"command":"deploy production"});
    assert_eq!(
        ledger.consume(&id, &context, &changed_action, 102).unwrap(),
        Decision::Denied
    );
    let id = ledger.request(&context, &changed_action, 103).unwrap();
    ledger.decide(&id, &id, false, &context, 104, 104).unwrap();
    assert_eq!(
        ledger.consume(&id, &context, &changed_action, 105).unwrap(),
        Decision::Denied
    );
    drop(ledger);
    cleanup(dir);
}

#[test]
fn simultaneous_callbacks_cannot_consume_one_human_approval_twice() {
    let (dir, mut ledger, context) = setup();
    let id = ledger.request(&context, &request(), 100).unwrap();
    ledger.decide(&id, &id, true, &context, 101, 201).unwrap();
    drop(ledger);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut jobs = vec![];
    for _ in 0..2 {
        let (dir, context, id, barrier) =
            (dir.clone(), context.clone(), id.clone(), barrier.clone());
        jobs.push(std::thread::spawn(move || {
            let mut ledger = Ledger::open(&dir.join("state.sqlite")).unwrap();
            barrier.wait();
            ledger.consume(&id, &context, &request(), 102).unwrap()
        }));
    }
    barrier.wait();
    let results = jobs
        .into_iter()
        .map(|job| job.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        results.iter().filter(|d| **d == Decision::Allowed).count(),
        1
    );
    assert_eq!(
        results.iter().filter(|d| **d == Decision::Denied).count(),
        1
    );
    let store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    assert!(!store.recall("Tool approval").unwrap().is_empty());
    cleanup(dir);
}

/// Windows refuses to delete open or briefly scanned files; retry after handles close.
fn cleanup(dir: PathBuf) {
    for _ in 0..50 {
        if std::fs::remove_dir_all(&dir).is_ok() || !dir.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    std::fs::remove_dir_all(dir).unwrap();
}
