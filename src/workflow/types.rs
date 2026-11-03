use super::*;

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
    #[serde(default)]
    pub timeouts: u32,
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
    pub folder: Option<crate::folder::Source>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charter: Option<crate::workflow::guardrail::Charter>,
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
