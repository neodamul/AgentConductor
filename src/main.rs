use agentconductor::plan::Plan;
use agentconductor::protocol::{Decoder, Provider};
use std::io::{Read, Write};

fn main() -> std::process::ExitCode {
    match execute() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("agentconductor: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => Err("expected a command; use --help".into()),
        [flag] if flag == "--help" || flag == "-h" => {
            println!(
                "AgentConductor\n\nUsage:\n  agentconductor validate <plan.json>\n  agentconductor normalize <claude|codex|opencode|custom> <stdout.jsonl>\n\nValidates plans or normalizes stored events without launching agents.\nWorkspace modes: shared (default), worktree (optional)."
            );
            Ok(())
        }
        [command, path] if command == "validate" => {
            let bytes = std::fs::read(path).map_err(|e| format!("read plan: {e}"))?;
            let plan: Plan =
                serde_json::from_slice(&bytes).map_err(|e| format!("parse plan: {e}"))?;
            plan.validate()?;
            println!(
                "{}",
                serde_json::json!({
                    "valid": true, "tasks": plan.tasks.len(), "workspace_mode": plan.workspace_mode
                })
            );
            Ok(())
        }
        [command, provider, path] if command == "normalize" => {
            let provider = match provider.as_str() {
                "claude" => Provider::Claude,
                "codex" => Provider::Codex,
                "opencode" => Provider::OpenCode,
                "custom" => Provider::Custom,
                _ => return Err("unknown provider".into()),
            };
            let mut file = std::fs::File::open(path).map_err(|e| format!("open events: {e}"))?;
            let mut decoder = Decoder::new(provider, 1024 * 1024);
            let mut buffer = [0u8; 8192];
            let mut out = std::io::stdout().lock();
            loop {
                let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
                let events = if count == 0 {
                    decoder.finish()?
                } else {
                    decoder.push(&buffer[..count])?
                };
                for event in events {
                    serde_json::to_writer(&mut out, &event).map_err(|e| e.to_string())?;
                    out.write_all(b"\n").map_err(|e| e.to_string())?;
                }
                if count == 0 {
                    break;
                }
            }
            Ok(())
        }
        _ => Err("invalid command or arguments; use --help".into()),
    }
}
