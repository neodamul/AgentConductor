//! R-213 / DD-225: new Goal folders never adopt existing project entries.
use agentconductor::{native_guard, ui::setup::Startup};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
struct Fixture {
    root: PathBuf,
    startup: Option<Startup>,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ac-goal-rules-{}",
            native_guard::session_id().unwrap()
        ));
        fs::create_dir_all(root.join("project")).unwrap();
        let root = root.canonicalize().unwrap();
        let startup = Startup::open(
            &root.join("workspace"),
            Path::new(env!("CARGO_BIN_EXE_agentconductor")),
        )
        .unwrap();
        let f = Self {
            root,
            startup: Some(startup),
        };
        f.mutate(json!({"kind":"save","config":{"profiles":[{"id":"custom-cli","executable":"not-executed","args":[],"permission_mode":"external_sandbox"}],"coordinator":"custom-cli","roles":{"development":["custom-cli"]},"repository":f.root.join("project"),"goal":"First goal","name":"Workspace","workspace_mode":"shared"}})).unwrap();
        f
    }
    fn call(&self, v: Value) -> Result<Value, String> {
        self.startup
            .as_ref()
            .unwrap()
            .dispatch(v.to_string().as_bytes())
    }
    fn snapshot(&self) -> Value {
        self.call(json!({"kind":"snapshot"})).unwrap()
    }
    fn mutate(&self, mut v: Value) -> Result<Value, String> {
        v["revision"] = self.snapshot()["revision"].clone();
        self.call(v)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.startup.take());
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn explicit_name_cannot_claim_an_existing_project_directory_or_file() {
    for directory in [true, false] {
        let f = Fixture::new();
        let target = f.root.join("project/first_goal");
        let sentinel = if directory {
            fs::create_dir(&target).unwrap();
            target.join("keep.txt")
        } else {
            target.clone()
        };
        fs::write(&sentinel, "unrelated source").unwrap();
        let before = f.snapshot();
        assert!(
            f.mutate(json!({"kind":"create_slug","slug":"first_goal"}))
                .is_err()
        );
        assert_eq!(fs::read_to_string(&sentinel).unwrap(), "unrelated source");
        assert!(!target.join(".agentconductor").exists());
        assert_eq!(f.snapshot()["config"], before["config"]);
    }
}

#[cfg(unix)]
#[test]
fn explicit_name_refuses_links_and_automatic_names_skip_dangling_links() {
    let f = Fixture::new();
    let outside = f.root.join("outside");
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, f.root.join("project/first_goal")).unwrap();
    assert!(
        f.mutate(json!({"kind":"create_slug","slug":"first_goal"}))
            .is_err()
    );
    assert!(!outside.join(".agentconductor").exists());
    fs::remove_file(f.root.join("project/first_goal")).unwrap();
    std::os::unix::fs::symlink(f.root.join("missing"), f.root.join("project/first_goal")).unwrap();
    let created = f.mutate(json!({"kind":"create_slug"})).unwrap();
    assert_eq!(created["slug"], "first_goal_2");
    assert!(!f.root.join("missing").exists());
}

#[test]
fn display_edits_keep_folder_identity_and_new_goal_keeps_workspace_context() {
    let f = Fixture::new();
    let created = f
        .mutate(json!({"kind":"create_slug","slug":"first_goal"}))
        .unwrap();
    let mut config = f.snapshot()["config"].clone();
    config["goal"] = json!("Display text changed");
    config["name"] = json!("Renamed Workspace");
    f.mutate(json!({"kind":"save","config":config})).unwrap();
    let before = f.snapshot()["config"].clone();
    assert_eq!(before["active_slug"], "first_goal");
    assert_eq!(before["slugs"][0]["slug"], "first_goal");
    assert!(
        Path::new(created["root"].as_str().unwrap())
            .join(".agentconductor")
            .exists()
    );
    f.mutate(json!({"kind":"new_goal"})).unwrap();
    let after = f.snapshot()["config"].clone();
    for field in [
        "repository",
        "coordinator",
        "profiles",
        "roles",
        "workspace_mode",
        "name",
    ] {
        assert_eq!(after[field], before[field], "{field}");
    }
    assert_eq!(after["goal"], "");
    assert!(after["active_slug"].is_null());
    assert_eq!(after["slugs"], before["slugs"]);
    assert!(!f.root.join("project/.git").exists());
}
