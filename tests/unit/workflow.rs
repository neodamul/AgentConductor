#[test]
fn lost_ownership_evidence_refuses_an_automatic_transfer() {
    use serde_json::json;
    let root = std::env::temp_dir().join(format!(
        "ac-fallback-evidence-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let restriction =
        json!({"available_at":null,"observed_at":1,"reason":"quota","source":"fixture"});
    let plan = json!({"goal":"three roles","agents":[
            {"id":"paid","executable":"paid-fixture","args":[],"quota_group":"paid","fallback":{"agent":"company"}},
            {"id":"company","executable":"company-fixture","args":[],"quota_group":"company"}],
            "tasks":(["one","two","three"].map(|id|json!({"id":id,"agent":"paid","prompt":id,"paths":[format!("{id}.txt")],"test_command":["fixture-test"]})))});
    let tasks=["one","two","three"].map(|id|json!({"id":id,"status":"waiting_availability","attempt":1,"restriction":restriction,
            "cwd":root,"started_ms":null,"finished_ms":null,"session_id":null,"error":"quota","base":"base","snapshot":null,"artifacts":root.join(id)}));
    let mut report:super::Report=serde_json::from_value(json!({"schema_version":1,"state":"running","repo":root,"base":"base","branch":"main",
            "plan":plan,"tasks":tasks,"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root,
            "account_restrictions":{"paid":restriction}})).unwrap();
    let mut store = crate::state::Store::open(root.join("state.sqlite")).unwrap();
    let error = super::apply_fallbacks(&mut report, &root, &mut store).unwrap_err();
    assert!(error.contains("missing worker ownership evidence"));
    assert!(report.plan.tasks.iter().all(|task| task.agent == "paid"));
    assert!(!root.join("handoff-0-1.md").exists());
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_restricted_first_role_with_a_due_fallback_moves_before_later_roles_start() {
    // GitHub #687: before any task launches, t01's account (paid) is out until well past the five-hour horizon, so its reviewed
    // fallback is due and t01 is transferred; t02 (company) is untouched, so the plan order t01 -> t02 holds.
    use serde_json::json;
    let root = std::env::temp_dir().join(format!(
        "ac-fallback-order-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    // Run state lives beside the repository, never inside it (it would count as an out-of-scope change).
    let run = root.with_file_name(format!(
        "{}-run",
        root.file_name().unwrap().to_string_lossy()
    ));
    std::fs::create_dir(&run).unwrap();
    // The transfer inspects the task's worktree against its base commit, so the fixture is a real repository.
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    git(&["init", "-q"]);
    git(&[
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@example.test",
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        "base",
    ]);
    let base = git(&["rev-parse", "HEAD"]);
    let now = 1_000_000_i64;
    let restriction =
        json!({"available_at":now+15*3600,"observed_at":now,"reason":"quota","source":"fixture"});
    let plan = json!({"goal":"two roles","agents":[
            {"id":"paid","executable":"paid-fixture","args":[],"quota_group":"paid","fallback":{"agent":"company"}},
            {"id":"company","executable":"company-fixture","args":[],"quota_group":"company"}],
            "tasks":[
              {"id":"one","agent":"paid","prompt":"one","paths":["one.txt"],"test_command":["fixture-test"]},
              {"id":"two","agent":"company","prompt":"two","paths":["two.txt"],"test_command":["fixture-test"]}]});
    let tasks=["one","two"].map(|id|json!({"id":id,"status":"pending","attempt":0,"restriction":null,
            "cwd":root,"started_ms":null,"finished_ms":null,"session_id":null,"error":null,"base":base,"snapshot":null,"artifacts":root.join(id)}));
    let mut report:super::Report=serde_json::from_value(json!({"schema_version":1,"state":"running","repo":root,"base":base,"branch":"main",
            "plan":plan,"tasks":tasks,"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root,
            "account_restrictions":{"paid":restriction}})).unwrap();
    let mut store = crate::state::Store::open(run.join("state.sqlite")).unwrap();
    let changed =
        super::assignment::apply_fallbacks_at(&mut report, &run, &mut store, now + 60).unwrap();
    assert!(
        changed,
        "the due fallback is applied while nothing has started"
    );
    assert_eq!(
        report.plan.tasks[0].agent, "company",
        "the restricted first role moved to its fallback"
    );
    assert_eq!(report.plan.tasks[1].agent, "company");
    assert_eq!(
        report.tasks[0].status, "pending",
        "it is ready to run in plan order, not parked"
    );
    // Inside the horizon the role keeps waiting for its own account.
    report.plan.tasks[0].agent = "paid".into();
    report.tasks[0].status = "pending".into();
    let soon =
        json!({"available_at":now+3600,"observed_at":now,"reason":"quota","source":"fixture"});
    report
        .account_restrictions
        .insert("paid".into(), serde_json::from_value(soon).unwrap());
    assert!(
        !super::assignment::apply_fallbacks_at(&mut report, &run, &mut store, now + 60).unwrap()
    );
    assert_eq!(report.plan.tasks[0].agent, "paid");
    assert_eq!(report.tasks[0].status, "waiting_availability");
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(run).unwrap();
}
