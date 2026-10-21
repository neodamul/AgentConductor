use super::*;
use agentconductor::cooperative::route;

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [command, port, packet] if command == "memory-publish" => Some((|| {
            let client = memory::Client::from_environment(
                port.parse().map_err(|_| "invalid local memory port")?,
            )?;
            println!(
                "{}",
                serde_json::to_string(&client.publish(&checkpoint_packet(packet)?)?)
                    .map_err(|e| e.to_string())?
            );
            Ok(())
        })()),
        [command, port, id] if command == "memory-read" => Some((|| {
            let client = memory::Client::from_environment(
                port.parse().map_err(|_| "invalid local memory port")?,
            )?;
            println!(
                "{}",
                serde_json::to_string(&client.read(id)?).map_err(|e| e.to_string())?
            );
            Ok(())
        })()),
        [command, dir, port] if command == "memory-discover" => Some((|| {
            let path = Path::new(dir).join("state.sqlite");
            let report = workflow::load(Path::new(dir))?;
            let client = memory::Client::from_environment(
                port.parse().map_err(|_| "invalid local memory port")?,
            )?;
            let mut store = Store::open(&path)?;
            let mut tasks = serde_json::Map::new();
            for spec in &report.plan.tasks {
                let cooperative = report.plan.agents.iter().any(|agent| {
                    agent.id == spec.agent
                        && agent.cooperative == agentconductor::plan::CooperativeMode::CheckpointV1
                });
                if !cooperative {
                    continue;
                }
                // Tasks without a current owned session have no address to discover yet.
                let result = match agentconductor::cooperative::worker_address(&mut store, &spec.id)
                {
                    Ok(address) => memory::checkpoints::discover(
                        &mut store,
                        &client,
                        &address,
                        checkpoint_time()?,
                    )?,
                    Err(error) => serde_json::json!({"unavailable":error}),
                };
                tasks.insert(spec.id.clone(), result);
            }
            println!("{}", serde_json::Value::Object(tasks));
            Ok(())
        })()),
        [command, dir, port, id] if command == "memory-import" => Some((|| {
            let path = Path::new(dir).join("state.sqlite");
            Store::read_only_snapshot(&path)?.document("workflow")?;
            let client = memory::Client::from_environment(
                port.parse().map_err(|_| "invalid local memory port")?,
            )?;
            let mut store = Store::open(&path)?;
            println!(
                "{}",
                memory::checkpoints::import(&mut store, &client, id, checkpoint_time()?)?
            );
            Ok(())
        })()),
        [command, dir, packet]
            if [
                "checkpoint-issue",
                "checkpoint-poll",
                "checkpoint-ack",
                "checkpoint-report",
            ]
            .contains(&command.as_str()) =>
        {
            Some((|| {
                let path = Path::new(dir).join("state.sqlite");
                // Verify an existing workflow before opening a writable ledger; no new run is created.
                Store::read_only_snapshot(&path)?.document("workflow")?;
                let mut store = Store::open(&path)?;
                let now = checkpoint_time()?;
                let value = match command.as_str() {
                    "checkpoint-issue" => {
                        let directive = agentconductor::cooperative::issue(
                            &mut store,
                            checkpoint_packet(packet)?,
                            now,
                        )?;
                        // DD-100: the workspace (a run's parent with startup.json) holds the preference.
                        let run = Path::new(dir);
                        let workspace = run
                            .parent()
                            .filter(|p| p.join("startup.json").exists())
                            .unwrap_or(run);
                        let preference = route::load_preference(workspace);
                        route::deliver(&mut store, &directive, &preference, workspace)?;
                        serde_json::to_value(directive)
                    }
                    "checkpoint-poll" => serde_json::to_value(agentconductor::cooperative::poll(
                        &mut store,
                        &checkpoint_packet(packet)?,
                        now,
                    )?),
                    "checkpoint-ack" => {
                        serde_json::to_value(agentconductor::cooperative::acknowledge(
                            &mut store,
                            checkpoint_packet(packet)?,
                            now,
                        )?)
                    }
                    "checkpoint-report" => {
                        serde_json::to_value(agentconductor::cooperative::report_progress(
                            &mut store,
                            checkpoint_packet(packet)?,
                            now,
                        )?)
                    }
                    _ => unreachable!(),
                }
                .map_err(|e| e.to_string())?;
                println!("{value}");
                Ok(())
            })())
        }
        [command, root] if command == "ui-new" => Some((|| {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let room = agentconductor::ui::Room::startup(Path::new(root), &exe)?;
            // A missing home directory only disables the recent list (DD-47).
            if let Ok(path) = Path::new(root).canonicalize() {
                let _ = agentconductor::ui::workspaces::remember(&path);
            }
            agentconductor::ui::native::open(
                std::sync::Arc::new(room),
                agentconductor::ui::native::Surface::Workspace,
            )
        })()),
        [command, dir] if command == "ui" => Some((|| {
            let executable = std::env::current_exe().map_err(|e| e.to_string())?;
            let room =
                std::sync::Arc::new(agentconductor::ui::Room::open(Path::new(dir), &executable)?);
            println!(
                "Agent Room opened for {dir}. Closing the window keeps active coordinators running."
            );
            agentconductor::ui::native::open(room, agentconductor::ui::native::Surface::AgentRoom)
        })()),
        [prefix, dir, revision, command, rest @ ..] if prefix == "ui-action" => Some((|| {
            if ![
                "approve",
                "reject",
                "merge",
                "revise",
                "reassign",
                "session-model",
                "schedule",
                "tool-approve",
                "tool-reject",
                "tool-refresh",
                "ui-resume",
                "resume-due",
                "recover",
                "quota-refresh",
                "scope-recovery",
            ]
            .contains(&command.as_str())
            {
                return Err("unsupported broker action".into());
            }
            workflow::expect_revision(Path::new(dir), revision)?;
            let mut forwarded = vec![command.clone(), dir.clone()];
            forwarded.extend_from_slice(rest);
            execute_args(&forwarded)
        })()),
        [command, dir] if command == "briefing" => Some((|| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();
            println!(
                "{}",
                agentconductor::briefing::collect(
                    Path::new(dir),
                    u64::try_from(now).map_err(|_| "briefing clock overflow")?
                )?
            );
            Ok(())
        })()),
        [command, dir, agent] if command == "quota-refresh" => Some((|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            runtime.block_on(workflow::refresh_quota(Path::new(dir), agent))?;
            let report = workflow::load(Path::new(dir))?;
            let profile = report
                .plan
                .agents
                .iter()
                .find(|p| p.id == *agent)
                .ok_or("unknown agent")?;
            let store = agentconductor::state::Store::open(Path::new(dir).join("state.sqlite"))?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs() as i64;
            println!("{}", agentconductor::quota::stored(&store, profile, now)?);
            Ok(())
        })()),
        [command, dir, baseline] if command == "ui-resume" => Some((|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let report = runtime.block_on(workflow::resume_reviewed_stop(
                Path::new(dir),
                1800,
                baseline,
            ))?;
            print_report(&runtime.block_on(workflow::repair(Path::new(dir), 1800, report))?)
        })()),
        [command, dir] if command == "knowledge-status" => Some((|| {
            let report = workflow::load(Path::new(dir))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&agentconductor::knowledge::status(
                    Path::new(dir),
                    if report.state == "completed" {
                        report
                            .folder
                            .as_ref()
                            .map_or(report.repo.as_path(), |f| f.original.as_path())
                    } else {
                        report.repo.as_path()
                    }
                )?)
                .map_err(|e| e.to_string())?
            );
            Ok(())
        })()),
        [command, dir, path, expected] if command == "knowledge-accept" => Some((|| {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)
                .map_err(|e| e.to_string())?
                .take(16 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 16 * 1024 {
                return Err("lesson exceeds 16 KiB".into());
            }
            let lesson: agentconductor::knowledge::Lesson =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            workflow::knowledge_accept(Path::new(dir), &lesson, expected)?;
            println!("Reviewed lesson accepted; source hashes will be revalidated before reuse.");
            Ok(())
        })()),
        [command, path] if command == "knowledge-digest" => Some((|| {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)
                .map_err(|e| e.to_string())?
                .take(16 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 16 * 1024 {
                return Err("lesson exceeds 16 KiB".into());
            }
            let lesson: agentconductor::knowledge::Lesson =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            println!(
                "{}",
                workflow::digest(&serde_json::to_vec(&lesson).map_err(|e| e.to_string())?)
            );
            Ok(())
        })()),
        [command, dir, task] if command == "progress" => Some((|| {
            let report = workflow::load(Path::new(dir))?;
            let task_report = report
                .tasks
                .iter()
                .find(|t| t.id == *task)
                .ok_or("unknown task")?;
            let store = agentconductor::state::Store::open(Path::new(dir).join("state.sqlite"))?;
            println!("{}",serde_json::to_string_pretty(&serde_json::json!({"task":task,"attempt":task_report.attempt,"status":task_report.status,"events":store.task_events(task,task_report.attempt as u64)?})).map_err(|e|e.to_string())?);
            Ok(())
        })()),
        _ => None,
    }
}
