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

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskReport {
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
    pub timed_out: bool,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
    pub stdout_hash: String,
    pub stderr_hash: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Report {
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
    fs::write(wiki.join("index.md"),"# Run knowledge\n\n- [Progress log](log.md)\n- [Local lessons](lessons.md)\n\nSQLite is authoritative; these pages are local progress summaries.\n").map_err(|e|e.to_string())?;
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
struct Lock(PathBuf);
impl Lock {
    fn acquire(dir: &Path) -> Result<Self, String> {
        let path = dir.join("coordinator.lock");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("run already has a coordinator lock: {e}"))?;
        writeln!(file, "{}", std::process::id()).map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn validate_permissions(plan: &Plan) -> Result<(), String> {
    for agent in &plan.agents {
        if !matches!(
            agent.permission_mode.as_str(),
            "external_sandbox" | "read_only" | "native_sandbox"
        ) {
            return Err(format!(
                "profile {} needs a verified permission mode; unrestricted CLI execution is unsupported",
                agent.id
            ));
        }
        if agent
            .args
            .iter()
            .any(|a| a.starts_with("--dangerously") || a == "--yolo" || a == "--auto")
        {
            return Err("permission bypass flags are unsupported".into());
        }
        if agent.parser != crate::protocol::Provider::Custom
            && agent.args.iter().any(|a| {
                matches!(
                    a.as_str(),
                    "--resume" | "--session" | "--session-id" | "--continue" | "--last"
                )
            })
        {
            return Err("native profiles must start independent sessions; the coordinator owns resume selection".into());
        }
        if agent.permission_mode == "native_sandbox"
            && !(agent.executable.rsplit('/').next() == Some("codex")
                && agent.args
                    == crate::adapters::invocation(
                        crate::protocol::Provider::Codex,
                        None,
                        "{task_file}",
                    )?
                    .args)
        {
            return Err("native_sandbox requires the verified Codex initial workspace-write invocation; other CLIs need an external sandbox".into());
        }
    }
    Ok(())
}

async fn worker(
    task: Task,
    agent: AgentProfile,
    mut report: TaskReport,
    worktree: bool,
    timeout: u64,
) -> TaskReport {
    report.error = None;
    report.restriction = None;
    let stop_file = report.artifacts.parent().unwrap().join("stop.request");
    let mut stopped = false;
    let result = async {
        let prompt = format!("{}\n\nTask: {}\nOwned paths: {}\nDo not change Git refs/index, deploy, push, transmit externally, or access secrets.\nCheck existing partial changes before continuing. Report evidence and stop when complete.\n", task.prompt, task.id, task.paths.join(", "));
        let prompt_file = report.artifacts.join("prompt.txt");
        fs::write(&prompt_file, &prompt).map_err(|e|e.to_string())?;
        let args = crate::adapters::session_args(&agent, task.model.as_deref(), report.session_id.as_deref(), prompt_file.to_str().ok_or("non-UTF8 prompt path")?)?;
        let process = process::run_until(ProcessSpec {executable:agent.executable.clone().into(),args,cwd:report.cwd.clone(),
            input:if agent.input_mode==InputMode::Stdin {prompt.into_bytes()} else {vec![]},log_dir:report.artifacts.join("process"),
            timeout:Duration::from_secs(timeout),signal_grace:Duration::from_secs(5),environment:agent.environment.clone()},Some(stop_file.clone())).await?;
        stopped=process.stopped;
        let mut file = fs::File::open(&process.stdout).map_err(|e|e.to_string())?;
        let mut decoder = Decoder::new(agent.parser,1024*1024);
        let mut buffer = [0;8192];
        let mut events = fs::File::create(report.artifacts.join("events.jsonl")).map_err(|e|e.to_string())?;
        let mut terminal = false;
        let mut progress = false;
        let mut provider_error = None;
        let mut parse_error = None;
        loop {
            let n = file.read(&mut buffer).map_err(|e|e.to_string())?;
            let batch = match if n==0 {decoder.finish()} else {decoder.push(&buffer[..n])} {
                Ok(batch)=>batch, Err(e)=>{parse_error=Some(e); break;}
            };
            for event in batch {
                if let Some(id)=&event.session_id {report.session_id=Some(id.clone());}
                if event.kind==EventKind::Error {
                    let message=event.payload.to_string();
                    report.restriction=availability::detect(&event.payload,&message,(now()/1000) as i64);
                    provider_error=Some(message);
                }
                terminal |= event.kind==EventKind::Result;
                progress |= event.kind==EventKind::Progress;
                serde_json::to_writer(&mut events,&event).map_err(|e|e.to_string())?;
                events.write_all(b"\n").map_err(|e|e.to_string())?;
            }
            if n==0 {break;}
        }
        let failed=process.exit_code!=Some(0)||process.timed_out||process.stopped||provider_error.is_some();
        if failed {
            if report.restriction.is_none() && !process.timed_out && !process.stopped {
                let mut message=String::new();
                fs::File::open(&process.stderr).map_err(|e|e.to_string())?.take(1024*1024).read_to_string(&mut message).map_err(|e|e.to_string())?;
                report.restriction=availability::detect(&serde_json::Value::Null,&message,(now()/1000) as i64);
            }
            return Err(provider_error.unwrap_or_else(||format!("agent exit={:?}, timed_out={}; inspect retained logs",process.exit_code,process.timed_out)));
        }
        if let Some(error)=parse_error {return Err(error);}
        if !terminal && !(agent.parser==crate::protocol::Provider::OpenCode && progress) {return Err("missing successful provider result".into());}
        if worktree {
            let git=Repository::open(&report.cwd)?;
            if git.head()?!=report.base {return Err("agent changed Git HEAD".into());}
            report.snapshot=Some(git.snapshot(&report.base,&task.paths,&report.artifacts.join("candidate.index"))?);
        }
        Ok::<_,String>(())
    }.await;
    report.finished_ms = Some(now());
    match result {
        Ok(()) => report.status = "completed".into(),
        Err(e) => {
            report.status = if stopped || stop_file.exists() {
                "stopped"
            } else if report.restriction.is_some() {
                "waiting_availability"
            } else {
                "failed"
            }
            .into();
            report.error = Some(e);
            report.retry_not_before = (now() / 1000) as i64 + 60;
        }
    }
    report
}

pub async fn run(
    plan: Plan,
    repo_path: &Path,
    run_dir: &Path,
    timeout: u64,
) -> Result<Report, String> {
    plan.validate()?;
    validate_permissions(&plan)?;
    if timeout == 0 {
        return Err("timeout must be positive".into());
    }
    let git = Repository::open(repo_path)?;
    git.clean()?;
    let base = git.head()?;
    let branch = git.branch()?;
    fs::create_dir(run_dir).map_err(|e| format!("create new run directory: {e}"))?;
    let dir = run_dir.canonicalize().map_err(|e| e.to_string())?;
    if dir.starts_with(&git.root) {
        return Err("run directory must be outside the repository".into());
    }
    let _lock = Lock::acquire(&dir)?;
    let worktree = plan.workspace_mode == WorkspaceMode::Worktree;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let mut report = Report {
        schema_version: 1,
        state: "running".into(),
        repo: git.root.clone(),
        base: base.clone(),
        branch,
        plan: plan.clone(),
        tasks: vec![],
        candidate: None,
        tests: vec![],
        diff_hash: None,
        error: None,
        verification_cwd: git.root.clone(),
        verification_attempt: 0,
    };
    for (i, task) in plan.tasks.iter().enumerate() {
        let artifacts = dir.join(format!("task-{i}"));
        fs::create_dir(&artifacts).map_err(|e| e.to_string())?;
        let cwd = if worktree {
            let cwd = dir.join(format!("workspace-{i}"));
            git.worktree(&cwd, &base)?;
            cwd
        } else {
            git.root.clone()
        };
        report.tasks.push(TaskReport {
            attempt: 0,
            restriction: None,
            retry_not_before: 0,
            automatic_retries: 0,
            id: task.id.clone(),
            status: "pending".into(),
            cwd,
            started_ms: None,
            finished_ms: None,
            session_id: None,
            error: None,
            base: base.clone(),
            snapshot: None,
            artifacts,
        });
    }
    save(&mut store, &dir, &report)?;
    drive(report, &dir, &mut store, timeout).await
}

async fn drive(
    mut report: Report,
    dir: &Path,
    store: &mut Store,
    timeout: u64,
) -> Result<Report, String> {
    let plan = report.plan.clone();
    let git = Repository::open(&report.repo)?;
    let base = report.base.clone();
    let worktree = plan.workspace_mode == WorkspaceMode::Worktree;
    report.state = "running".into();
    let mut jobs = tokio::task::JoinSet::new();
    loop {
        if dir.join("stop.request").exists() {
            for task in &mut report.tasks {
                if task.status == "pending" {
                    task.status = "stopped".into();
                }
            }
        }
        for i in 0..plan.tasks.len() {
            if report.tasks[i].status != "pending" {
                continue;
            }
            let deps: Vec<_> = plan.tasks[i]
                .depends_on
                .iter()
                .map(|id| report.tasks.iter().find(|t| &t.id == id).unwrap())
                .collect();
            if deps
                .iter()
                .any(|t| matches!(t.status.as_str(), "failed" | "blocked"))
            {
                report.tasks[i].status = "blocked".into();
                continue;
            }
            if deps.iter().any(|t| t.status != "completed") {
                continue;
            }
            let profile = plan
                .agents
                .iter()
                .find(|a| a.id == plan.tasks[i].agent)
                .unwrap();
            let active = report
                .tasks
                .iter()
                .enumerate()
                .filter(|(j, t)| t.status == "running" && plan.tasks[*j].agent == profile.id)
                .count();
            if active >= profile.max_sessions {
                continue;
            }
            if report.tasks.iter().enumerate().any(|(j, t)| {
                t.status == "waiting_availability"
                    && plan
                        .agents
                        .iter()
                        .find(|a| a.id == plan.tasks[j].agent)
                        .unwrap()
                        .quota_key()
                        == profile.quota_key()
            }) {
                continue;
            }
            if report.tasks[i].session_id.as_ref().is_some_and(|id| {
                report.tasks.iter().enumerate().any(|(j, t)| {
                    j != i
                        && t.status == "running"
                        && t.session_id.as_ref() == Some(id)
                        && plan.tasks[j].agent == profile.id
                })
            }) {
                continue;
            }
            if worktree && report.tasks[i].attempt == 0 {
                let workspace = Repository::open(&report.tasks[i].cwd)?;
                for dep in &deps {
                    if let Some(snapshot) = &dep.snapshot {
                        workspace.command(&["merge", "--no-edit", &snapshot.commit])?;
                    }
                }
                report.tasks[i].base = workspace.head()?;
            }
            if report.tasks[i].attempt > 0 {
                let artifacts =
                    dir.join(format!("task-{i}-attempt-{}", report.tasks[i].attempt + 1));
                fs::create_dir(&artifacts).map_err(|e| e.to_string())?;
                report.tasks[i].artifacts = artifacts;
            }
            report.tasks[i].attempt += 1;
            report.tasks[i].status = "running".into();
            report.tasks[i].started_ms = Some(now());
            let agent = plan
                .agents
                .iter()
                .find(|a| a.id == plan.tasks[i].agent)
                .unwrap()
                .clone();
            let task = plan.tasks[i].clone();
            let task_report = report.tasks[i].clone();
            jobs.spawn(
                async move { (i, worker(task, agent, task_report, worktree, timeout).await) },
            );
        }
        save(store, dir, &report)?;
        match jobs.join_next().await {
            Some(result) => {
                let (i, task) = result.map_err(|e| e.to_string())?;
                report.tasks[i] = task;
            }
            None => break,
        }
    }
    if dir.join("stop.request").exists() {
        report.state = "stopped".into();
        store.record("stop",&serde_json::json!({"request":fs::read_to_string(dir.join("stop.request")).map_err(|e|e.to_string())?}),"Human stop completed; owned processes exited and attempts retained")?;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
        save(store, dir, &report)?;
        return Ok(report);
    }
    if report.tasks.iter().any(|t| t.status != "completed") {
        report.state = if report
            .tasks
            .iter()
            .any(|t| t.status == "waiting_availability")
        {
            "waiting_availability"
        } else {
            "failed"
        }
        .into();
        save(store, dir, &report)?;
        return Ok(report);
    }
    report.verification_attempt += 1;
    let verification = async {
        if git.head()? != base || git.branch()? != report.branch {
            return Err("target Git state changed during execution".into());
        }
        let candidate = if worktree {
            let patches = report
                .tasks
                .iter()
                .map(|t| {
                    Repository::open(&t.cwd)?.diff(&t.base, &t.snapshot.as_ref().unwrap().commit)
                })
                .collect::<Result<Vec<_>, String>>()?;
            git.combine(&base, &patches, &dir.join("integration.index"))?
        } else {
            if !git
                .command(&["diff", "--cached", "--name-only"])?
                .is_empty()
            {
                return Err("agent modified the shared Git index".into());
            }
            git.snapshot(&base, &scopes(&plan), &dir.join("integration.index"))?
        };
        let diff = git.diff(&base, &candidate.commit)?;
        report.diff_hash = Some(digest(&diff));
        fs::write(dir.join("diff.patch"), diff).map_err(|e| e.to_string())?;
        report.candidate = Some(candidate.clone());
        if worktree {
            let cwd = dir.join(format!("integration-{}", report.verification_attempt));
            git.worktree(&cwd, &candidate.commit)?;
            report.verification_cwd = cwd;
        }
        for (i, task) in plan.tasks.iter().enumerate() {
            let process = process::run_until(
                ProcessSpec {
                    executable: task.test_command[0].clone().into(),
                    args: task.test_command[1..].to_vec(),
                    cwd: report.verification_cwd.clone(),
                    input: vec![],
                    log_dir: dir.join(format!("test-{i}-{}", report.verification_attempt)),
                    timeout: Duration::from_secs(timeout),
                    signal_grace: Duration::from_secs(5),
                    environment: Default::default(),
                },
                Some(dir.join("stop.request")),
            )
            .await?;
            let evidence = TestReport {
                task: task.id.clone(),
                exit_code: process.exit_code,
                timed_out: process.timed_out,
                stdout_hash: file_digest(&process.stdout)?,
                stderr_hash: file_digest(&process.stderr)?,
                stdout: process.stdout,
                stderr: process.stderr,
            };
            let passed = evidence.exit_code == Some(0) && !evidence.timed_out;
            report.tests.push(evidence);
            if !passed {
                return Err(format!("integration test failed: {}", task.id));
            }
        }
        let verification_git = Repository::open(&report.verification_cwd)?;
        let verify_base = if worktree { &candidate.commit } else { &base };
        if verification_git
            .snapshot(verify_base, &scopes(&plan), &dir.join("verification.index"))?
            .tree
            != candidate.tree
        {
            return Err("test commands modified candidate files".into());
        }
        Ok::<_, String>(())
    }
    .await;
    match verification {
        Ok(()) => report.state = "waiting_approval".into(),
        Err(e) => {
            report.state = if dir.join("stop.request").exists() {
                "stopped"
            } else {
                "failed"
            }
            .into();
            report.error = Some(e);
        }
    }
    if dir.join("stop.request").exists() {
        report.state = "stopped".into();
        store.record("stop", &serde_json::json!({"request":fs::read_to_string(dir.join("stop.request")).map_err(|e|e.to_string())?}), "Human stopped integration verification")?;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
    }
    save(store, dir, &report)?;
    Ok(report)
}

fn binding(dir: &Path, report: &Report) -> Result<String, String> {
    let git = Repository::open(&report.repo)?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err("target branch changed since verification".into());
    }
    let candidate = report.candidate.as_ref().ok_or("no verified candidate")?;
    if report.plan.workspace_mode == WorkspaceMode::Shared
        && !git
            .command(&["diff", "--cached", "--name-only"])?
            .is_empty()
    {
        return Err("shared Git index changed since verification".into());
    }
    if digest(&git.diff(&report.base, &candidate.commit)?)
        != report.diff_hash.clone().ok_or("missing diff hash")?
    {
        return Err("candidate Git diff differs from verified evidence".into());
    }
    if git.text(&["rev-parse", &format!("{}^{{tree}}", candidate.commit)])? != candidate.tree {
        return Err("candidate tree identity mismatch".into());
    }
    if file_digest(&dir.join("diff.patch"))?
        != report.diff_hash.clone().ok_or("diff evidence missing")?
    {
        return Err("diff evidence changed".into());
    }
    for test in &report.tests {
        if test.exit_code != Some(0)
            || test.timed_out
            || file_digest(&test.stdout)? != test.stdout_hash
            || file_digest(&test.stderr)? != test.stderr_hash
        {
            return Err("test evidence changed or failed".into());
        }
    }
    if report.tests.len() != report.plan.tasks.len() {
        return Err("integration tests incomplete".into());
    }
    let verification = Repository::open(&report.verification_cwd)?;
    if report.plan.workspace_mode == WorkspaceMode::Worktree {
        git.clean()?;
    }
    let base = if report.plan.workspace_mode == WorkspaceMode::Worktree {
        &candidate.commit
    } else {
        &report.base
    };
    if verification
        .snapshot(base, &scopes(&report.plan), &dir.join("approval.index"))?
        .tree
        != candidate.tree
    {
        return Err("candidate files changed since verification".into());
    }
    let value = serde_json::json!({"repo":report.repo,"base":report.base,"branch":report.branch,"candidate":candidate,"plan":report.plan,"diff_hash":report.diff_hash,"tests":report.tests});
    Ok(digest(
        &serde_json::to_vec(&value).map_err(|e| e.to_string())?,
    ))
}

pub fn approve(dir: &Path, candidate: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if report.state != "waiting_approval" {
        return Err("run is not waiting for approval".into());
    }
    if report.candidate.as_ref().map(|c| c.commit.as_str()) != Some(candidate) {
        return Err("approval requires the exact candidate SHA".into());
    }
    let fingerprint = binding(dir, &report)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "approval",
        &serde_json::json!({"fingerprint":fingerprint,"candidate":candidate}),
        "Human approved exact integration candidate",
    )?;
    report.state = "approved".into();
    save(&mut store, dir, &report)
}
pub fn reject(dir: &Path, reason: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if !matches!(report.state.as_str(), "waiting_approval" | "approved") {
        return Err("run has no pending approval".into());
    }
    report.state = "rejected".into();
    report.error = Some(reason.into());
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Human rejected candidate",
    )?;
    save(&mut store, dir, &report)
}
pub fn merge(dir: &Path) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if report.state != "approved" {
        return Err("human approval is required before merge".into());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let approval = store.document("approval")?;
    let fingerprint = binding(dir, &report)?;
    if approval["fingerprint"] != fingerprint {
        return Err("approval no longer matches candidate and evidence".into());
    }
    let git = Repository::open(&report.repo)?;
    let candidate = &report.candidate.as_ref().unwrap().commit;
    match report.plan.workspace_mode {
        WorkspaceMode::Shared => git.merge_shared(&report.branch, &report.base, candidate)?,
        WorkspaceMode::Worktree => {
            git.command(&["merge", "--ff-only", candidate])?;
        }
    }
    report.state = "completed".into();
    save(&mut store, dir, &report)
}

fn editable(report: &Report) -> Result<(), String> {
    if report.state == "completed" || report.tasks.iter().any(|t| t.status == "running") {
        return Err("stop the active coordinator before editing; merged runs are immutable".into());
    }
    Ok(())
}

fn task_index(report: &Report, id: &str) -> Result<usize, String> {
    report
        .tasks
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| "unknown task/session ID".into())
}

fn invalidate(report: &mut Report, index: usize, store: &mut Store) -> Result<(), String> {
    report.candidate = None;
    report.diff_hash = None;
    report.tests.clear();
    report.error = None;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Session edit revoked prior integration approval",
    )?;
    report.tasks[index].status = "pending".into();
    report.tasks[index].snapshot = None;
    let mut changed = vec![report.tasks[index].id.clone()];
    loop {
        let mut added = false;
        for (i, task) in report.plan.tasks.iter().enumerate() {
            if !changed.contains(&task.id) && task.depends_on.iter().any(|id| changed.contains(id))
            {
                changed.push(task.id.clone());
                report.tasks[i].status = "pending".into();
                report.tasks[i].snapshot = None;
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    report.state = "ready".into();
    Ok(())
}

/// Scheduling a session also applies to waiting sessions in its account quota group.
pub fn schedule(dir: &Path, id: &str, timestamp: &str) -> Result<(), String> {
    let time = availability::parse_time(timestamp)?;
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    if report.tasks[index].status != "waiting_availability" {
        return Err("session has no temporary restriction".into());
    }
    let profile = report
        .plan
        .agents
        .iter()
        .find(|a| a.id == report.plan.tasks[index].agent)
        .unwrap();
    let group = profile.quota_key().to_string();
    for (i, task) in report.tasks.iter_mut().enumerate() {
        let agent = report
            .plan
            .agents
            .iter()
            .find(|a| a.id == report.plan.tasks[i].agent)
            .unwrap();
        if agent.quota_key() == group
            && task.status == "waiting_availability"
            && let Some(restriction) = &mut task.restriction
        {
            restriction.available_at = Some(time);
            restriction.source = "manual_schedule".into();
            task.retry_not_before = time;
            task.automatic_retries = 0;
        }
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record("schedule",&serde_json::json!({"task":id,"quota_group":group,"available_at":time,"original_timestamp":timestamp}),"Human scheduled restricted account sessions")?;
    save(&mut store, dir, &report)
}

pub fn set_model(dir: &Path, id: &str, model: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    report
        .plan
        .agents
        .iter()
        .find(|a| a.id == report.plan.tasks[index].agent)
        .unwrap()
        .validate_model(model)?;
    report.plan.tasks[index].model = Some(model.into());
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let previous_status = report.tasks[index].status.clone();
    invalidate(&mut report, index, &mut store)?;
    if previous_status == "waiting_availability" {
        report.tasks[index].status = previous_status;
        report.state = "waiting_availability".into();
    }
    store.record(
        "session_model",
        &serde_json::json!({"task":id,"model":model}),
        "Human changed session model; work must be verified again",
    )?;
    save(&mut store, dir, &report)
}

pub fn reassign(dir: &Path, id: &str, agent: &str, model: Option<&str>) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let index = task_index(&report, id)?;
    let target = report
        .plan
        .agents
        .iter()
        .find(|a| a.id == agent)
        .ok_or("unknown target agent")?;
    if let Some(model) = model {
        target.validate_model(model)?;
    }
    let old_agent = report.plan.tasks[index].agent.clone();
    let task = &report.tasks[index];
    let git = Repository::open(&task.cwd)?;
    let allowed = if report.plan.workspace_mode == WorkspaceMode::Shared {
        scopes(&report.plan)
    } else {
        report.plan.tasks[index].paths.clone()
    };
    git.check_scope(&git.changed(&task.base)?, &allowed)?;
    let mut args = vec!["diff", "--binary", task.base.as_str(), "--"];
    args.extend(report.plan.tasks[index].paths.iter().map(String::as_str));
    let diff = git.command(&args)?;
    let handoff = dir.join(format!("handoff-{index}-{}.md", task.attempt));
    let text = format!(
        "# Role handoff\n\nGoal: {}\nTask: {}\nFrom: {}\nTo: {}\nPrevious provider session: {:?}\nOwned paths: {}\nPrior evidence: {}\n\nInspect existing owned files, including untracked files, before continuing. Preserve completed work.\n\n```diff\n{}\n```\n",
        report.plan.goal,
        id,
        old_agent,
        agent,
        task.session_id,
        report.plan.tasks[index].paths.join(", "),
        task.artifacts.display(),
        String::from_utf8_lossy(&diff[..diff.len().min(64 * 1024)])
    );
    fs::write(&handoff, &text).map_err(|e| e.to_string())?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record("handoff",&serde_json::json!({"task":id,"from":old_agent,"to":agent,"previous_session":task.session_id,"evidence":task.artifacts,"handoff":handoff}),"Human reassigned a role; target uses a new provider session")?;
    report.plan.tasks[index].agent = agent.into();
    report.plan.tasks[index].model = model.map(String::from);
    // Bounded facts only; no free-form cross-agent conversation or foreign native ID.
    report.plan.tasks[index].prompt.push_str("\n\n");
    report.plan.tasks[index].prompt.push_str(&text);
    report.tasks[index].session_id = None;
    report.tasks[index].restriction = None;
    invalidate(&mut report, index, &mut store)?;
    save(&mut store, dir, &report)
}

pub async fn resume_due(dir: &Path, current_time: i64, timeout: u64) -> Result<Report, String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if matches!(
        report.state.as_str(),
        "completed" | "waiting_approval" | "approved" | "rejected" | "stopped"
    ) {
        return Ok(report);
    }
    editable(&report)?;
    report.plan.validate()?;
    validate_permissions(&report.plan)?;
    let git = Repository::open(&report.repo)?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err("target Git state changed before resume".into());
    }
    for task in &mut report.tasks {
        if task.status == "waiting_availability"
            && task
                .restriction
                .as_ref()
                .is_some_and(|r| r.available_at.is_some_and(|t| t <= current_time))
            && task.retry_not_before <= current_time
            && task.automatic_retries < 3
        {
            task.status = "pending".into();
            task.restriction = None;
            task.automatic_retries += 1;
        }
    }
    if !report.tasks.iter().any(|t| t.status == "pending") {
        return Ok(report);
    }
    for task in &mut report.tasks {
        if task.status == "blocked" {
            task.status = "pending".into();
        }
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "resume",
        &serde_json::json!({"at":current_time}),
        "Coordinator claimed due sessions under exclusive run lock",
    )?;
    report.tests.clear();
    report.candidate = None;
    report.diff_hash = None;
    report.error = None;
    drive(report, dir, &mut store, timeout).await
}

/// Durable request; the active coordinator consumes and audits it after draining its jobs.
pub fn stop(dir: &Path, reason: &str) -> Result<(), String> {
    let report = load(dir)?;
    if reason.trim().is_empty()
        || !matches!(
            report.state.as_str(),
            "running" | "ready" | "waiting_availability" | "stopped"
        )
    {
        return Err("stop needs a reason and an active, ready, or restricted run".into());
    }
    let path = dir.join("stop.request");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            serde_json::to_writer(
                &mut file,
                &serde_json::json!({"reason":reason,"requested_at_ms":now()}),
            )
            .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.to_string()),
    }
    if let Ok(_lock) = Lock::acquire(dir) {
        let mut report = load(dir)?;
        if report.tasks.iter().any(|t| t.status == "running") {
            return Err("unknown process ownership; request retained for explicit recovery".into());
        }
        report.state = "stopped".into();
        let mut store = Store::open(dir.join("state.sqlite"))?;
        store.record(
            "stop",
            &serde_json::json!({"reason":reason}),
            "Human stopped an idle coordinator",
        )?;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
        save(&mut store, dir, &report)?;
    }
    Ok(())
}

pub async fn resume(dir: &Path, timeout: u64) -> Result<Report, String> {
    if timeout == 0 {
        return Err("timeout must be positive".into());
    }
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if report.state != "stopped" {
        return Err("explicit resume requires a confirmed stopped run".into());
    }
    editable(&report)?;
    report.plan.validate()?;
    validate_permissions(&report.plan)?;
    let git = Repository::open(&report.repo)?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err("target Git state changed before resume".into());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Explicit resume revoked prior integration approval",
    )?;
    store.record(
        "resume",
        &serde_json::json!({"requested_at_ms":now()}),
        "Human resumed confirmed stopped attempts",
    )?;
    for task in &mut report.tasks {
        if task.status == "stopped" || task.status == "blocked" {
            task.status = "pending".into();
        }
    }
    report.tests.clear();
    report.candidate = None;
    report.diff_hash = None;
    report.error = None;
    let marker = dir.join("stop.request");
    if marker.exists() {
        fs::remove_file(marker).map_err(|e| e.to_string())?;
    }
    drive(report, dir, &mut store, timeout).await
}
