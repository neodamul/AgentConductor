use agentconductor::{native_guard, state::Store, ui::setup::Startup};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub struct Fixture {
    pub root: PathBuf,
    pub startup: Option<Startup>,
}
impl Fixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ac-goal-actions-{}",
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
        f.mutate(json!({"kind":"save","config":{"profiles":[],"coordinator":"","repository":f.root.join("project"),"name":"Project","goal":"First goal","workspace_mode":"shared"}})).unwrap();
        f.mutate(json!({"kind":"create_slug","slug":"first_goal"}))
            .unwrap();
        f
    }
    pub fn call(&self, packet: Value) -> Result<Value, String> {
        self.startup
            .as_ref()
            .unwrap()
            .dispatch(packet.to_string().as_bytes())
    }
    pub fn snapshot(&self) -> Value {
        self.call(json!({"kind":"snapshot"})).unwrap()
    }
    pub fn mutate(&self, mut packet: Value) -> Result<Value, String> {
        packet["revision"] = self.snapshot()["revision"].clone();
        self.call(packet)
    }
    pub fn delete(&self) -> Value {
        json!({"kind":"delete_goal","slug":"first_goal","goal":"First goal","root":self.folder()})
    }
    pub fn folder(&self) -> PathBuf {
        self.root.join("project/first_goal")
    }
    pub fn edit_saved(&mut self, edit: impl FnOnce(&mut Value)) {
        drop(self.startup.take());
        let path = self.root.join("workspace/startup.json");
        let mut data: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        edit(&mut data);
        fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
        self.startup = Some(
            Startup::open(
                &self.root.join("workspace"),
                Path::new(env!("CARGO_BIN_EXE_agentconductor")),
            )
            .unwrap(),
        );
    }
    pub fn report(&self, state: &str) -> PathBuf {
        let run = self.root.join("workspace/run-first");
        fs::create_dir_all(&run).unwrap();
        let report = json!({"schema_version":1,"state":state,"repo":self.root.join("project"),"base":"base","branch":"main","plan":{"goal":"First goal","agents":[],"tasks":[],"workspace_mode":"shared"},"tasks":[],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":self.root.join("project")});
        fs::write(
            run.join("report.json"),
            serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
        Store::open(run.join("state.sqlite"))
            .unwrap()
            .record("workflow", &report, "fixture")
            .unwrap();
        run
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.startup.take());
        let _ = fs::remove_dir_all(&self.root);
    }
}
