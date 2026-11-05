use std::{path::PathBuf, time::Duration};

mod ownership;
pub use ownership::Ownership;
pub(crate) use ownership::record_owner;
mod supervision;
use supervision::run_with_control;

pub struct ProcessSpec {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub input: Vec<u8>,
    pub log_dir: PathBuf,
    pub timeout: Duration,
    pub signal_grace: Duration,
    pub environment: std::collections::BTreeMap<String, String>,
}

pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub termination_signal: Option<i32>,
    pub timed_out: bool,
    pub stopped: bool,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
}

#[derive(Default)]
pub struct ControlReply {
    pub messages: Vec<serde_json::Value>,
    pub close_input: bool,
}

pub type ControlHandler = Box<dyn FnMut(serde_json::Value) -> Result<ControlReply, String> + Send>;
pub type AsyncControlHandler = Box<
    dyn FnMut(
            serde_json::Value,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ControlReply, String>> + Send>,
        > + Send,
>;

pub async fn run(spec: ProcessSpec) -> Result<ProcessResult, String> {
    run_until(spec, None).await
}

pub async fn run_until(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
) -> Result<ProcessResult, String> {
    run_with_control(spec, stop_file, None, None, None).await
}

pub async fn run_controlled(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    mut handler: ControlHandler,
) -> Result<ProcessResult, String> {
    run_controlled_async(
        spec,
        stop_file,
        Box::new(move |message| {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(message)))
                    .map_err(|_| "control evaluator panicked".to_string())
                    .and_then(|result| result);
            Box::pin(async move { result })
        }),
    )
    .await
}

pub async fn run_controlled_async(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
) -> Result<ProcessResult, String> {
    run_controlled_async_within(spec, stop_file, handler, Duration::from_secs(30), None).await
}

/// Like `run_controlled_async`, but a single control evaluation may take `limit`,
/// e.g. while a paused tool call awaits a human decision. While `paused` is set the
/// process timeout does not run down, so a slow decision never times the task out.
pub async fn run_controlled_async_within(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
    limit: Duration,
    paused: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<ProcessResult, String> {
    run_with_control(spec, stop_file, Some((handler, limit)), paused, None).await
}

/// Like `run_controlled_async_within`, and lines sent on `outbound` are also written to the child's stdin,
/// in the order they were queued (the acio control channel, GitHub #208).
pub async fn run_controlled_outbound(
    spec: ProcessSpec,
    stop_file: Option<PathBuf>,
    handler: AsyncControlHandler,
    limit: Duration,
    outbound: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) -> Result<ProcessResult, String> {
    run_with_control(
        spec,
        stop_file,
        Some((handler, limit)),
        None,
        Some(outbound),
    )
    .await
}

mod control;
use control::*;
mod containment;
pub(crate) use containment::*;
#[cfg(windows)]
pub(crate) mod windows_job;
