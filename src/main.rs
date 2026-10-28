#![cfg_attr(windows, windows_subsystem = "windows")]
use agentconductor::plan::{AgentProfile, Plan};
use agentconductor::protocol::{Decoder, Provider};
use agentconductor::{adapters, memory, state::Store, workflow};
use std::io::{Read, Write};
use std::path::Path;

fn read_plan(path: &str) -> Result<Plan, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read plan: {e}"))?;
    let plan: Plan = serde_json::from_slice(&bytes).map_err(|e| format!("parse plan: {e}"))?;
    plan.validate()?;
    Ok(plan)
}
fn plan_digest(plan: &Plan) -> Result<String, String> {
    Ok(workflow::digest(
        &serde_json::to_vec(plan).map_err(|e| e.to_string())?,
    ))
}
fn print_report(report: &workflow::Report) -> Result<(), String> {
    println!(
        "{}",
        serde_json::json!({"state":report.state,"base":report.base,"candidate":report.candidate,
        "tasks":report.tasks,"tests":report.tests,"error":report.error})
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let maintenance = args.as_slice() == ["manual", "render"]
        || args.first().map(String::as_str) == Some("bundle-macos")
        || args.first().map(String::as_str) == Some("archive-package")
        || args.first().map(String::as_str) == Some("validate-package")
        || args.first().map(String::as_str) == Some("smoke-package")
        || args.first().map(String::as_str) == Some("package-release")
        || (args.first().map(String::as_str) == Some("release")
            && args.get(1).map(String::as_str) == Some("notes"));
    // A detached watcher must retain its own logs, not attach to the caller console.
    if !maintenance && args.first().map(String::as_str) != Some("availability-watch") {
        agentconductor::subprocess::attach_parent_console();
    }
    let result = if maintenance {
        execute_args(&args)
    } else {
        execute()
    };
    // DD-79: let queued messenger notifications finish (bounded) before the process exits.
    if !maintenance {
        agentconductor::messenger::flush(std::time::Duration::from_secs(35));
    }
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("agentconductor: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn write_profiles(path: &str, profiles: &[AgentProfile]) -> Result<(), String> {
    let temporary = Path::new(path).with_extension(format!("{}.tmp", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut file, profiles).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    std::fs::rename(&temporary, path).map_err(|e| e.to_string())
}

fn execute() -> Result<(), String> {
    // DD-135: a process started by an update waits for the one it replaced.
    agentconductor::update::after_restart();
    let args: Vec<_> = std::env::args().skip(1).collect();
    execute_args(&args)
}

fn checkpoint_packet<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(16385)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16384 {
        return Err("checkpoint packet exceeds 16 KiB".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn checkpoint_time() -> Result<u64, String> {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis(),
    )
    .map_err(|_| "checkpoint clock overflow".into())
}

#[path = "commands/execution.rs"]
mod execution;
#[path = "commands/memory_ui.rs"]
mod memory_ui;
#[path = "commands/sessions_review.rs"]
mod sessions_review;

#[path = "commands/quota.rs"]
mod quota_commands;

#[path = "commands/chat.rs"]
mod chat_commands;
#[path = "commands/io.rs"]
mod io_commands;
#[path = "commands/manual.rs"]
mod manual_commands;
#[path = "commands/mcp.rs"]
mod mcp_commands;
#[path = "commands/optimize.rs"]
mod optimize_commands;

fn execute_args(args: &[String]) -> Result<(), String> {
    if let Some(result) = io_commands::execute(args) {
        return result;
    }
    if let Some(result) = manual_commands::execute(args) {
        return result;
    }
    if let Some(result) = mcp_commands::execute(args) {
        return result;
    }
    if let Some(result) = chat_commands::execute(args) {
        return result;
    }
    if let Some(result) = optimize_commands::execute(args) {
        return result;
    }
    if let Some(result) = quota_commands::execute(args) {
        return result;
    }
    if let Some(result) = memory_ui::execute(args) {
        return result;
    }
    if let Some(result) = sessions_review::execute(args) {
        return result;
    }
    if let Some(result) = execution::execute(args) {
        return result;
    }
    Err("invalid command or arguments; use --help".into())
}
