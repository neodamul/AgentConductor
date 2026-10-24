use agentconductor::{
    availability::Restriction,
    git::Repository,
    process::Ownership,
    state::Store,
    ui::Room,
    workflow::{self, Report},
};
use serde_json::json;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
fn fixture(seconds: i64) -> (std::path::PathBuf, std::path::PathBuf, Report) {
    let (root, mut plan) = support::setup("shared");
    let mut backup = plan.agents[0].clone();
    backup.id = "backup".into();
    backup.quota_group = Some("backup".into());
    plan.agents[0].quota_group = Some("paid".into());
    plan.agents.push(backup);
    let repo = Repository::open(&root.join("repo")).unwrap();
    let base = repo.head().unwrap();
    let run = root.join("run");
    std::fs::create_dir(&run).unwrap();
    let restriction = Restriction {
        available_at: Some(now() + seconds),
        observed_at: now(),
        reason: "quota".into(),
        source: "fixture".into(),
    };
    let tasks:Vec<_>=plan.tasks.iter().enumerate().map(|(i,t)|{let artifacts=run.join(format!("task-{i}"));std::fs::create_dir(&artifacts).unwrap();json!({"id":t.id,"status":"waiting_availability","attempt":0,"restriction":restriction,"cwd":repo.root,"started_ms":null,"finished_ms":null,"session_id":null,"error":"quota","base":base,"snapshot":null,"artifacts":artifacts})}).collect();
    let mut report:Report=serde_json::from_value(json!({"schema_version":1,"state":"waiting_availability","repo":repo.root,"base":base,"branch":repo.branch().unwrap(),"plan":plan,"tasks":tasks,"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":repo.root,"account_restrictions":{"paid":restriction}})).unwrap();
    report.checkpoints = vec![workflow::WorkspaceCheckpoint {
        cwd: repo.root.clone(),
        head: base.clone(),
        tree: repo
            .checkpoint(&base, &run.join("paused-fixture.index"))
            .unwrap()
            .tree,
        index_hash: workflow::digest(&repo.command(&["ls-files", "--stage", "-z"]).unwrap()),
    }];
    persist(&run, &report);
    (root, run, report)
}
fn persist(run: &Path, report: &Report) {
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record(
            "workflow",
            &serde_json::to_value(report).unwrap(),
            "availability test fixture",
        )
        .unwrap();
    std::fs::write(run.join("report.json"), serde_json::to_vec(report).unwrap()).unwrap();
}
fn owner(run: &Path) -> Ownership {
    serde_json::from_slice(
        &std::fs::read(run.join("availability-coordinator/process/owner.json")).unwrap(),
    )
    .unwrap()
}
fn wait_for(mut check: impl FnMut() -> bool) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !check() {
        assert!(
            std::time::Instant::now() < until,
            "availability coordinator deadline"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
#[test]
fn closing_the_ui_restart_recovery_and_reassignment_preserve_and_cancel_one_reservation() {
    let (root, run, mut report) = fixture(3600);
    // Reassignment cancels even when the target aliases the same limited account.
    report.plan.agents[1].quota_group = Some("paid".into());
    report.tasks[1].status = "completed".into();
    report.tasks[1].restriction = None;
    report.tasks[2].status = "completed".into();
    report.tasks[2].restriction = None;
    persist(&run, &report);
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let entries = workflow::refresh_availability(&run, now()).unwrap();
    assert_eq!(entries.len(), 1);
    let deadline = entries[0].at;
    assert!(workflow::ensure_availability_watcher(&run, exe).unwrap());
    wait_for(|| owner(&run).coordinator_pid == owner(&run).pid.unwrap());
    let first = owner(&run).pid.unwrap();
    let room = Room::open(&run, exe).unwrap();
    drop(room);
    assert!(workflow::ensure_availability_watcher(&run, exe).unwrap());
    assert_eq!(owner(&run).pid, Some(first));
    #[cfg(windows)]
    // SAFETY: `first` is the watcher PID read from the owner record; the handle is checked for null before use and closed once.
    unsafe {
        use windows_sys::Win32::System::Threading::*;
        let handle = OpenProcess(PROCESS_TERMINATE, 0, first);
        assert!(!handle.is_null());
        assert_ne!(TerminateProcess(handle, 1), 0);
        windows_sys::Win32::Foundation::CloseHandle(handle);
    }
    #[cfg(unix)]
    // SAFETY: kill takes only integers; `first` is the PID of the watcher process this test started, and the result is asserted.
    unsafe {
        assert_eq!(libc::kill(first as i32, libc::SIGKILL), 0);
    }
    wait_for(|| owner(&run).ensure_quiescent().is_ok());
    assert!(workflow::ensure_availability_watcher(&run, exe).unwrap());
    wait_for(|| {
        owner(&run).pid != Some(first) && owner(&run).coordinator_pid == owner(&run).pid.unwrap()
    });
    let second = owner(&run).pid;
    assert_eq!(
        Store::open(run.join("state.sqlite"))
            .unwrap()
            .document("availability_timer")
            .unwrap()["entries"][0]["at"],
        deadline
    );
    workflow::reassign(&run, &report.tasks[0].id, "backup", None).unwrap();
    wait_for(|| owner(&run).phase == "terminal" && owner(&run).ensure_quiescent().is_ok());
    assert_ne!(second, Some(first));
    let current = workflow::load(&run).unwrap();
    assert!(workflow::availability_timer(&current, now()).is_empty());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let current = runtime
        .block_on(workflow::resume_availability_due(&run, now() + 7200, 5))
        .unwrap();
    assert_eq!(current.tasks[0].attempt, 0);
    assert_eq!(current.plan.tasks[0].agent, "backup");
    assert!(!root.join("repo/one.txt").exists());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn the_independent_coordinator_resumes_due_work_without_a_ui() {
    let (root, run, _) = fixture(3);
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    workflow::ensure_availability_watcher(&run, exe).unwrap();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(240);
    loop {
        let o = owner(&run);
        assert!(
            !(o.phase == "terminal" && o.exit_code != Some(0)),
            "{}",
            std::fs::read_to_string(run.join("availability-coordinator/process/stderr.log"))
                .unwrap()
        );
        let r = workflow::load(&run).unwrap();
        if r.state == "waiting_approval" {
            assert!(r.tasks.iter().all(|t| t.status == "completed"));
            break;
        }
        assert!(
            !matches!(r.state.as_str(), "failed" | "blocked"),
            "{:?}",
            r.error
        );
        assert!(
            std::time::Instant::now() < until,
            "coordinator did not resume"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    wait_for(|| owner(&run).phase == "terminal" && owner(&run).ensure_quiescent().is_ok());
    assert!(
        Store::open(run.join("state.sqlite"))
            .unwrap()
            .document("availability_timer")
            .unwrap()["entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn metadata_sync_does_not_move_the_original_five_hour_deadline() {
    let (root, run, report) = fixture(3600);
    workflow::refresh_availability(&run, now()).unwrap();
    let first = report.tasks[0].restriction.as_ref().unwrap().observed_at;
    let mut changed = workflow::load(&run).unwrap();
    for t in &mut changed.tasks {
        t.restriction.as_mut().unwrap().observed_at = first + 3600;
    }
    for r in changed.account_restrictions.values_mut() {
        r.observed_at = first + 3600;
    }
    persist(&run, &changed);
    workflow::refresh_availability(&run, now() + 3600).unwrap();
    let restored = workflow::load(&run).unwrap();
    assert_eq!(
        restored.tasks[0].restriction.as_ref().unwrap().observed_at,
        first
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn human_stop_cancels_the_watcher_and_its_durable_reservation() {
    let (root, run, _) = fixture(3600);
    workflow::ensure_availability_watcher(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor")))
        .unwrap();
    workflow::stop(&run, "Cancel automatic resume").unwrap();
    wait_for(|| owner(&run).phase == "terminal" && owner(&run).ensure_quiescent().is_ok());
    assert!(
        Store::open(run.join("state.sqlite"))
            .unwrap()
            .document("availability_timer")
            .unwrap()["entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(workflow::load(&run).unwrap().state, "stopped");
    std::fs::remove_dir_all(root).unwrap();
}
