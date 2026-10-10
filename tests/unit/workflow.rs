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
