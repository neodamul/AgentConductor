use crate::{
    adapters::InputMode,
    availability,
    git::Repository,
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

mod types;
pub use types::*;

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
    availability_timer::record_timer(store, dir, report, (now() / 1000) as i64)?;
    fs::write(
        dir.join("report.json"),
        serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    // DD-79: a linked run reports notifiable state changes; delivery never fails the run.
    let done = report
        .tasks
        .iter()
        .filter(|t| t.status == "completed")
        .count();
    crate::messenger::notify_run(dir, &report.state, done, report.tasks.len());
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
    // A pure read: Store::open would run schema statements that race a starting run (#65).
    let store = Store::read_only_snapshot(dir.join("state.sqlite"))?;
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
mod stray;
use scheduler::*;
mod approval;
pub use approval::*;
mod editing;
pub use editing::*;
mod review;
pub use review::*;
mod assignment;
mod pending_edit;
mod session_model;
pub use assignment::*;
pub use session_model::*;
mod recovery;
pub use recovery::*;
mod scope_recovery;
pub use scope_recovery::*;
mod phase_link;
pub use phase_link::*;
pub mod blockers;

mod automatic;
pub use automatic::*;
mod repair;
pub use repair::*;

mod availability_timer;
pub use availability_timer::{
    AvailabilityTimer, FALLBACK_WAIT_SECONDS, availability_timer, refresh_availability,
};

mod availability_watcher;
pub use availability_watcher::{ensure_availability_watcher, watch_availability};
