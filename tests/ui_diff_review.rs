use agentconductor::{state::Store, ui::Room, workflow};
use serde_json::json;
#[test]
fn a_shortened_preview_still_checks_the_entire_patch() {
    let root = std::env::temp_dir().join(format!(
        "ac-diff-review-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(root.join("repo/.git")).unwrap();
    let root = root.canonicalize().unwrap();
    let run = root.join("run");
    std::fs::create_dir(&run).unwrap();
    let patch = "+large verified content\n".repeat(30000);
    let hash = workflow::digest(patch.as_bytes());
    let value = json!({"schema_version":1,"state":"waiting_approval","repo":root.join("repo"),"base":"base","branch":"main","plan":{"goal":"large change","agents":[],"tasks":[],"workspace_mode":"shared"},"tasks":[],"candidate":{"commit":"candidate","tree":"tree","changed":["large.txt"]},"tests":[],"diff_hash":hash,"error":null,"verification_cwd":root.join("repo")});
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &value, "diff review fixture")
        .unwrap();
    std::fs::write(run.join("report.json"), value.to_string()).unwrap();
    std::fs::write(run.join("diff.patch"), &patch).unwrap();
    let room = Room::open(
        &run,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let evidence = room.dispatch(br#"{"kind":"evidence"}"#).unwrap();
    assert_eq!(evidence["truncated"], true);
    assert!(evidence["diff"].as_str().unwrap().len() <= 512 * 1024);
    assert_eq!(evidence["report"]["diff_hash"], hash);
    let mut changed = patch.into_bytes();
    let last = changed.len() - 1;
    changed[last] = b'X';
    std::fs::write(run.join("diff.patch"), changed).unwrap();
    assert!(
        room.dispatch(br#"{"kind":"evidence"}"#).is_err(),
        "changes outside the preview must invalidate full-patch evidence"
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
