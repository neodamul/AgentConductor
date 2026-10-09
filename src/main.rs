use agentconductor::plan::Plan;

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
                "AgentConductor\n\nUsage: agentconductor validate <plan.json>\n\nValidates a three-task plan without launching agents.\nWorkspace modes: shared (default), worktree (optional)."
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
        _ => Err("invalid command or arguments; use --help".into()),
    }
}
