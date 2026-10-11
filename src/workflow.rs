use crate::{
    adapters::InputMode,
    availability::{self, Restriction},
    git::{Repository, Snapshot},
    plan::{AgentProfile, Plan, Task, WorkspaceMode},
    process::{self, ProcessSpec},
    protocol::{Decoder, EventKind},
    state::Store,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyContext {
    pub task: String,
    pub attempt: u32,
    pub snapshot: Option<String>,
    pub paths: Vec<String>,
    pub artifacts: PathBuf,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskReport {
    #[serde(default)]
    pub approval_epoch: u64,
    #[serde(default)]
    pub pending_tool: Option<crate::gates::ToolRequest>,
    #[serde(default)]
    pub tool_request: Option<String>,
    #[serde(default)]
    pub session_established: bool,
    #[serde(default)]
    pub feedback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_context: Option<crate::cooperative::Continuation>,
    #[serde(default)]
    pub dependencies: Vec<DependencyContext>,
    #[serde(default)]
    pub attempt: u32,
    #[serde(default)]
    pub restriction: Option<Restriction>,
    #[serde(default)]
    pub retry_not_before: i64,
    #[serde(default)]
    pub automatic_retries: u32,
    pub id: String,
    pub status: String,
    pub cwd: PathBuf,
    pub started_ms: Option<u128>,
    pub finished_ms: Option<u128>,
    pub session_id: Option<String>,
    pub error: Option<String>,
    pub base: String,
    pub snapshot: Option<Snapshot>,
    pub artifacts: PathBuf,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub task: String,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination_signal: Option<i32>,
    pub timed_out: bool,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
    pub stdout_hash: String,
    pub stderr_hash: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceCheckpoint {
    pub cwd: PathBuf,
    pub head: String,
    pub tree: String,
    pub index_hash: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Report {
    #[serde(default)]
    pub account_restrictions: std::collections::BTreeMap<String, Restriction>,
    #[serde(default)]
    pub review_round: u32,
    #[serde(default)]
    pub latest_review: Option<crate::coordinator::ReviewProposal>,
    #[serde(default)]
    pub review_error: Option<String>,
    pub schema_version: u32,
    pub state: String,
    pub repo: PathBuf,
    pub base: String,
    pub branch: String,
    pub plan: Plan,
    pub tasks: Vec<TaskReport>,
    pub candidate: Option<Snapshot>,
    pub tests: Vec<TestReport>,
    pub diff_hash: Option<String>,
    pub error: Option<String>,
    pub verification_cwd: PathBuf,
    #[serde(default)]
    pub verification_attempt: u32,
    #[serde(default)]
    pub checkpoints: Vec<WorkspaceCheckpoint>,
}

fn workspace_checkpoint(cwd: &Path, index: &Path) -> Result<WorkspaceCheckpoint, String> {
    let git = Repository::open(cwd)?;
    let head = git.head()?;
    // One listing: a second listing could race with concurrent writers (live tool decisions).
    let tree = git.checkpoint(&head, index)?.tree;
    Ok(WorkspaceCheckpoint {
        cwd: git.root.clone(),
        head,
        tree,
        index_hash: digest(&git.command(&["ls-files", "--stage", "-z"])?),
    })
}

fn capture_checkpoints(report: &mut Report, dir: &Path) -> Result<(), String> {
    let mut paths =
        std::collections::BTreeSet::from([report.repo.clone(), report.verification_cwd.clone()]);
    paths.extend(report.tasks.iter().map(|t| t.cwd.clone()));
    report.checkpoints = paths
        .iter()
        .enumerate()
        .map(|(i, path)| workspace_checkpoint(path, &dir.join(format!("checkpoint-{i}.index"))))
        .collect::<Result<_, _>>()?;
    Ok(())
}

fn validate_checkpoints(report: &Report, dir: &Path) -> Result<(), String> {
    if report.checkpoints.is_empty() {
        return Err(
            "missing paused-workspace checkpoint; explicit recovery investigation required".into(),
        );
    }
    for (i, expected) in report.checkpoints.iter().enumerate() {
        let actual = workspace_checkpoint(
            &expected.cwd,
            &dir.join(format!("checkpoint-validation-{i}.index")),
        )?;
        if actual.head != expected.head
            || actual.tree != expected.tree
            || actual.index_hash != expected.index_hash
        {
            return Err(format!(
                "paused workspace changed; review required before resuming: {}",
                expected.cwd.display()
            ));
        }
    }
    Ok(())
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

static REVIEWED_REVISION: std::sync::OnceLock<(PathBuf, String)> = std::sync::OnceLock::new();

pub fn revision(report: &Report) -> Result<String, String> {
    Ok(digest(
        &serde_json::to_vec(report).map_err(|e| e.to_string())?,
    ))
}

/// One immutable precondition for a broker-launched CLI process.
pub fn expect_revision(dir: &Path, expected: &str) -> Result<(), String> {
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid reviewed run revision".into());
    }
    REVIEWED_REVISION
        .set((
            dir.canonicalize().map_err(|e| e.to_string())?,
            expected.into(),
        ))
        .map_err(|_| "reviewed revision was already bound".into())
}
fn file_digest(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut bytes = [0; 8192];
    loop {
        let n = file.read(&mut bytes).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn scopes(plan: &Plan) -> Vec<String> {
    plan.tasks.iter().flat_map(|t| t.paths.clone()).collect()
}
fn save(store: &mut Store, dir: &Path, report: &Report) -> Result<(), String> {
    // JSON references do not protect Git objects from garbage collection.
    let git = Repository::open(&report.repo)?;
    let run_key = digest(dir.to_string_lossy().as_bytes());
    for snapshot in report
        .tasks
        .iter()
        .filter_map(|t| t.snapshot.as_ref())
        .chain(report.candidate.iter())
    {
        git.command(&[
            "update-ref",
            &format!("refs/codex/agentconductor/{run_key}/{}", snapshot.commit),
            &snapshot.commit,
        ])?;
    }
    let value = serde_json::to_value(report).map_err(|e| e.to_string())?;
    let summary = format!(
        "AgentConductor run={} goal={} phase={} completed={} failed={}",
        dir.display(),
        report.plan.goal,
        report.state,
        report
            .tasks
            .iter()
            .filter(|t| t.status == "completed")
            .count(),
        report.tasks.iter().filter(|t| t.status == "failed").count()
    );
    store.record("workflow", &value, &summary)?;
    fs::write(
        dir.join("report.json"),
        serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let wiki = dir.join("knowledge");
    fs::create_dir_all(&wiki).map_err(|e| e.to_string())?;
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(wiki.join("log.md"))
        .map_err(|e| e.to_string())?;
    writeln!(log, "\n- observed_at_ms={} {}", now(), summary).map_err(|e| e.to_string())?;
    fs::write(wiki.join("index.md"),"# Run knowledge\n\n- [Progress log](log.md)\n- [Draft lessons](lessons.md)\n- [Reviewed runtime lessons](wiki/index.md)\n\nSQLite is authoritative; these pages are local progress summaries. Reviewed lesson links are populated after exact promotion.\n").map_err(|e|e.to_string())?;
    if !wiki.join("lessons.md").exists() {
        fs::write(
            wiki.join("lessons.md"),
            "# Lessons\n\nRecord context, correction, evidence, confidence, and scope here.\n",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn load(dir: &Path) -> Result<Report, String> {
    if !dir.join("state.sqlite").exists() {
        return Err("run state does not exist".into());
    }
    let store = Store::open(dir.join("state.sqlite"))?;
    serde_json::from_value(store.document("workflow")?).map_err(|e| e.to_string())
}

/// Metadata-only account refresh; it never changes plan or candidate evidence.
pub async fn refresh_quota(dir: &Path, agent: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let report = load(dir)?;
    if report.state == "running" {
        return Err("wait for the coordinator before refreshing account metadata".into());
    }
    let profile = report
        .plan
        .agents
        .iter()
        .find(|profile| profile.id == agent)
        .ok_or("unknown quota agent")?;
    if cfg!(target_os = "macos")
        && profile.parser == crate::protocol::Provider::Claude
        && Path::new(&profile.executable)
            .file_name()
            .and_then(|s| s.to_str())
            == Some("claude")
    {
        let query_dir = dir.join(format!(
            "quota-query-{}",
            crate::native_guard::session_id()?
        ));
        let result = crate::quota::query_claude(profile, &query_dir, &report.repo).await;
        let mut store = Store::open(dir.join("state.sqlite"))?;
        return crate::quota::persist(&mut store, profile, result);
    }
    if profile.parser == crate::protocol::Provider::Claude {
        let mut store = Store::open(dir.join("state.sqlite"))?;
        let current = crate::quota::stored(&store, profile, (now() / 1000) as i64)?;
        return store.record("quota-check", &serde_json::json!({"agent":agent,"status":current["status"],"observed_at":current["observed_at"]}), "Checked retained Claude quota; no new observation or process requested");
    }
    let query_dir = dir.join(format!(
        "quota-query-{}",
        crate::native_guard::session_id()?
    ));
    let result = crate::quota::query(profile, &query_dir).await;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    crate::quota::persist(&mut store, profile, result)
}

#[cfg(test)]
#[path = "../tests/unit/workflow.rs"]
mod fallback_evidence_tests;

mod permissions;
pub use permissions::*;
mod execution;
pub use execution::*;
mod scheduler;
use scheduler::*;
mod approval;
pub use approval::*;
mod editing;
pub use editing::*;
mod review;
pub use review::*;
mod assignment;
pub use assignment::*;
mod recovery;
pub use recovery::*;
mod phase_link;
pub use phase_link::*;
