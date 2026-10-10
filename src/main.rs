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
    match execute() {
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

fn execute_args(args: &[String]) -> Result<(), String> {
    match args {
        [command, port, packet] if command == "memory-publish" => {
            let client = memory::Client::from_environment(
                port.parse().map_err(|_| "invalid local memory port")?,
            )?;
            println!(
                "{}",
                serde_json::to_string(&client.publish(&checkpoint_packet(packet)?)?)
                    .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        [command, port, id] if command == "memory-read" => {
            let client = memory::Client::from_environment(
                port.parse().map_err(|_| "invalid local memory port")?,
            )?;
            println!(
                "{}",
                serde_json::to_string(&client.read(id)?).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        [command, dir, port, id] if command == "memory-import" => {
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
        }
        [command, dir, packet]
            if [
                "checkpoint-issue",
                "checkpoint-poll",
                "checkpoint-ack",
                "checkpoint-report",
            ]
            .contains(&command.as_str()) =>
        {
            let path = Path::new(dir).join("state.sqlite");
            // Verify an existing workflow before opening a writable ledger; no new run is created.
            Store::read_only_snapshot(&path)?.document("workflow")?;
            let mut store = Store::open(&path)?;
            let now = checkpoint_time()?;
            let value = match command.as_str() {
                "checkpoint-issue" => serde_json::to_value(agentconductor::cooperative::issue(
                    &mut store,
                    checkpoint_packet(packet)?,
                    now,
                )?),
                "checkpoint-poll" => serde_json::to_value(agentconductor::cooperative::poll(
                    &mut store,
                    &checkpoint_packet(packet)?,
                    now,
                )?),
                "checkpoint-ack" => serde_json::to_value(agentconductor::cooperative::acknowledge(
                    &mut store,
                    checkpoint_packet(packet)?,
                    now,
                )?),
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
        }
        [command, root] if command == "ui-new" => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            agentconductor::ui::native::open(std::sync::Arc::new(
                agentconductor::ui::Room::startup(Path::new(root), &exe)?,
            ))
        }
        [command, dir] if command == "ui" => {
            let executable = std::env::current_exe().map_err(|e| e.to_string())?;
            let room =
                std::sync::Arc::new(agentconductor::ui::Room::open(Path::new(dir), &executable)?);
            println!(
                "Agent Room opened for {dir}. Closing the window keeps active coordinators running."
            );
            agentconductor::ui::native::open(room)
        }
        [prefix, dir, revision, command, rest @ ..] if prefix == "ui-action" => {
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
            ]
            .contains(&command.as_str())
            {
                return Err("unsupported broker action".into());
            }
            workflow::expect_revision(Path::new(dir), revision)?;
            let mut forwarded = vec![command.clone(), dir.clone()];
            forwarded.extend_from_slice(rest);
            execute_args(&forwarded)
        }
        [command, dir] if command == "briefing" => {
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
        }
        [command, dir, agent] if command == "quota-refresh" => {
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
        }
        [command, dir, baseline] if command == "ui-resume" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            print_report(&runtime.block_on(workflow::resume_reviewed_stop(
                Path::new(dir),
                1800,
                baseline,
            ))?)
        }
        [command, dir] if command == "knowledge-status" => {
            let report = workflow::load(Path::new(dir))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&agentconductor::knowledge::status(
                    Path::new(dir),
                    &report.repo
                )?)
                .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        [command, dir, path, expected] if command == "knowledge-accept" => {
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
        }
        [command, path] if command == "knowledge-digest" => {
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
        }
        [command, dir, task] if command == "progress" => {
            let report = workflow::load(Path::new(dir))?;
            let task_report = report
                .tasks
                .iter()
                .find(|t| t.id == *task)
                .ok_or("unknown task")?;
            let store = agentconductor::state::Store::open(Path::new(dir).join("state.sqlite"))?;
            println!("{}",serde_json::to_string_pretty(&serde_json::json!({"task":task,"attempt":task_report.attempt,"status":task_report.status,"events":store.task_events(task,task_report.attempt as u64)?})).map_err(|e|e.to_string())?);
            Ok(())
        }
        [command, dir] if command == "tool-requests" => {
            workflow::load(Path::new(dir))?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &agentconductor::gates::Ledger::open(&Path::new(dir).join("state.sqlite"))?
                        .requests()?
                )
                .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        [command, dir, id] if command == "tool-approve" || command == "tool-reject" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            runtime.block_on(workflow::tool_decide(
                Path::new(dir),
                id,
                command == "tool-approve",
                300,
            ))?;
            print_report(&workflow::load(Path::new(dir))?)
        }
        [command, dir, id, seconds] if command == "tool-approve" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            runtime.block_on(workflow::tool_decide(
                Path::new(dir),
                id,
                true,
                seconds.parse().map_err(|_| "invalid approval lifetime")?,
            ))?;
            print_report(&workflow::load(Path::new(dir))?)
        }
        [command, dir, id] if command == "tool-refresh" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::json!({"request_sha256":runtime.block_on(workflow::tool_refresh(Path::new(dir),id))?})
            );
            Ok(())
        }
        [command, dir, task] if command == "tool-context" => {
            let mut bytes = vec![];
            std::io::stdin()
                .take(64 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 64 * 1024 {
                return Err("tool-context request exceeds 64 KiB".into());
            }
            let request: agentconductor::gates::ToolRequest =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::to_string(&workflow::tool_context(Path::new(dir), task, &request)?)
                    .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        [command, dir, coordinator] if command == "review" || command == "review-apply" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let proposal = runtime.block_on(workflow::review(
                Path::new(dir),
                coordinator,
                command == "review-apply",
            ))?;
            println!(
                "{}",
                serde_json::json!({"proposal_sha256":workflow::digest(&serde_json::to_vec(&proposal).map_err(|e|e.to_string())?),"proposal":proposal,"round":workflow::load(Path::new(dir))?.review_round})
            );
            Ok(())
        }
        [command, dir, proposal] if command == "apply-review" => {
            workflow::apply_review(Path::new(dir), proposal)?;
            print_report(&workflow::load(Path::new(dir))?)
        }
        [command, dir, task, feedback] if command == "revise" => {
            workflow::revise(Path::new(dir), task, feedback)?;
            print_report(&workflow::load(Path::new(dir))?)
        }
        [command, dir] if command == "recover" => print_report(&workflow::recover(Path::new(dir))?),
        [command, dir, reason] if command == "stop" => {
            workflow::stop(Path::new(dir), reason)?;
            println!("Stop request retained; inspect status until the run is stopped.");
            Ok(())
        }
        [command, dir] if command == "resume" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            print_report(&runtime.block_on(workflow::resume(Path::new(dir), 1800))?)
        }
        [command, profiles_path, agent_id, dir] if command == "models-discover" => {
            let mut profiles: Vec<AgentProfile> =
                serde_json::from_slice(&std::fs::read(profiles_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let agent = profiles
                .iter_mut()
                .find(|a| a.id == *agent_id)
                .ok_or("unknown agent")?;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let catalog =
                runtime.block_on(agentconductor::models::discover(agent, Path::new(dir)))?;
            println!(
                "{}",
                serde_json::to_string(&catalog).map_err(|e| e.to_string())?
            );
            agent.catalog = Some(catalog);
            write_profiles(profiles_path, &profiles)
        }
        [command, profiles_path, agent_id, catalog_path] if command == "catalog-import" => {
            let mut profiles: Vec<AgentProfile> =
                serde_json::from_slice(&std::fs::read(profiles_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let catalog: agentconductor::plan::ModelCatalog =
                serde_json::from_slice(&std::fs::read(catalog_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if catalog.models.is_empty()
                || catalog.queried_at <= 0
                || catalog.source.trim().is_empty()
            {
                return Err("catalog needs observed models and query provenance".into());
            }
            let agent = profiles
                .iter_mut()
                .find(|a| a.id == *agent_id)
                .ok_or("unknown agent")?;
            agent.catalog = Some(catalog);
            for model in &agent.catalog.as_ref().unwrap().models {
                if model.is_empty()
                    || model.starts_with('-')
                    || model.chars().any(char::is_whitespace)
                {
                    return Err("invalid model ID".into());
                }
            }
            write_profiles(profiles_path, &profiles)
        }
        [command, profiles_path, coordinator, repo, dir, goal] if command == "plan" => {
            let profiles: Vec<AgentProfile> =
                serde_json::from_slice(&std::fs::read(profiles_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let plan = runtime.block_on(agentconductor::planner::generate(
                goal,
                &profiles,
                coordinator,
                Path::new(repo),
                Path::new(dir),
            ))?;
            println!(
                "{}",
                serde_json::json!({"plan":Path::new(dir).join("plan.json"),"plan_digest":plan_digest(&plan)?})
            );
            Ok(())
        }
        [command, dir, task, timestamp] if command == "schedule" => {
            workflow::schedule(Path::new(dir), task, timestamp)
        }
        [command, dir, task, model] if command == "session-model" => {
            workflow::set_model(Path::new(dir), task, model)
        }
        [command, dir, task, agent] if command == "reassign" => {
            workflow::reassign(Path::new(dir), task, agent, None)
        }
        [command, dir, task, agent, model] if command == "reassign" => {
            workflow::reassign(Path::new(dir), task, agent, Some(model))
        }
        [command, dir] if command == "sessions" => {
            let report = workflow::load(Path::new(dir))?;
            for (i, session) in report.tasks.iter().enumerate() {
                let task = &report.plan.tasks[i];
                let agent = report
                    .plan
                    .agents
                    .iter()
                    .find(|a| a.id == task.agent)
                    .unwrap();
                println!(
                    "{}",
                    serde_json::json!({"logical_session":session.id,"agent":task.agent,"provider_session":session.session_id,"model":task.model,"state":session.status,"attempt":session.attempt,"restriction":session.restriction,"retry_not_before":session.retry_not_before,"catalog":agent.catalog})
                );
            }
            Ok(())
        }
        [command, dir] if command == "resume-due" || command == "watch" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            loop {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs() as i64;
                let report = runtime.block_on(workflow::resume_due(Path::new(dir), now, 1800))?;
                print_report(&report)?;
                if command == "resume-due" || report.state != "waiting_availability" {
                    return Ok(());
                }
                let next = report
                    .tasks
                    .iter()
                    .filter(|t| t.automatic_retries < 3)
                    .filter_map(|t| {
                        t.restriction
                            .as_ref()
                            .and_then(|r| r.available_at.map(|time| time.max(t.retry_not_before)))
                    })
                    .min();
                let Some(next) = next else {
                    return Err("reset time missing or automatic retry ceiling reached; inspect sessions and schedule/reassign manually".into());
                };
                std::thread::sleep(std::time::Duration::from_secs(
                    (next - now).clamp(1, 30) as u64
                ));
            }
        }
        [] => {
            let home = std::env::var_os("HOME")
                .ok_or("HOME is unavailable; use ui-new <workspace-dir>")?;
            let root = std::path::PathBuf::from(home).join(".agentconductor/desktop");
            execute_args(&["ui-new".into(), root.to_string_lossy().into()])
        }
        [flag] if flag == "--help" || flag == "-h" => {
            println!(
                "AgentConductor\n\nCommands:\n  ui-new <workspace-dir>\n  ui <run-dir>\n  profiles <profiles.json>\n  models-discover <profiles.json> <agent-id> <new-discovery-dir>\n  catalog-import <profiles.json> <agent-id> <observed-catalog.json>\n  plan <profiles.json> <coordinator-id> <repo> <new-plan-dir> <goal>\n  sessions <run-dir>\n  briefing <run-dir>\n  checkpoint-issue <run-dir> <issue.json>\n  checkpoint-poll <run-dir> <address.json>\n  checkpoint-ack <run-dir> <ack.json>\n  checkpoint-report <run-dir> <progress.json>\n  quota-refresh <run-dir> <agent-id>\n  schedule <run-dir> <task-id> <YYYY-MM-DDTHH:MM:SS+HH:MM>\n  reassign <run-dir> <task-id> <agent-id> [model]\n  session-model <run-dir> <task-id> <model>\n  stop <run-dir> <reason>\n  resume <run-dir>\n  recover <run-dir>\n  resume-due <run-dir>\n  watch <run-dir>\n  validate <plan.json>\n  run <plan.json> <repo> <new-run-dir> <approved-plan-digest>\n  status <run-dir>\n  logs <run-dir> <task-id>\n  progress <run-dir> <task-id>\n  diff <run-dir>\n  tool-requests <run-dir>\n  tool-refresh <run-dir> <exact-request-sha256>\n  tool-approve <run-dir> <exact-request-sha256> [lifetime-seconds]\n  tool-reject <run-dir> <exact-request-sha256>\n  approve <run-dir> <exact-candidate-sha>\n  reject <run-dir> <reason>\n  revise <run-dir> <task-id> <feedback>\n  review <run-dir> <coordinator-id>\n  review-apply <run-dir> <coordinator-id>\n  apply-review <run-dir> <exact-proposal-sha256>\n  merge <run-dir>\n  knowledge-digest <lesson.json>\n  knowledge-accept <run-dir> <lesson.json> <exact-digest>\n  knowledge-status <run-dir>\n  recall <run-dir> <query>\n  memory-sync <run-dir> <local-port>\n  memory-publish <local-port> <envelope.json>\n  memory-read <local-port> <memory-id>\n  memory-import <run-dir> <local-port> <memory-id>\n  normalize <claude|codex|opencode|custom> <stdout.jsonl>\n\nShared workspace is default; worktrees are optional. Run directories must be outside the repository.\nReview profiles, scopes and test commands before passing the validate command's plan digest.\nProfiles need an operator-verified sandbox/permission configuration before execution."
            );
            Ok(())
        }
        [command, path] if command == "validate" => {
            let plan = read_plan(path)?;
            println!(
                "{}",
                serde_json::json!({
                    "valid": true, "tasks": plan.tasks.len(), "workspace_mode": plan.workspace_mode,"plan_digest":plan_digest(&plan)?
                })
            );
            Ok(())
        }
        [command, path] if command == "profiles" => {
            let mut profiles = vec![];
            for (id, provider) in [
                ("claude", Provider::Claude),
                ("codex", Provider::Codex),
                ("opencode", Provider::OpenCode),
            ] {
                let invocation = adapters::invocation(provider, None, "{task_file}")?;
                profiles.push(AgentProfile {
                    id: id.into(),
                    executable: invocation.executable,
                    args: invocation.args,
                    parser: provider,
                    input_mode: invocation.input,
                    environment: Default::default(),
                    catalog: None,
                    max_sessions: 3,
                    quota_group: None,
                    fallback: None,
                    permission_mode: if id == "codex" {
                        "native_sandbox".into()
                    } else {
                        "unverified".into()
                    },
                });
            }
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|e| e.to_string())?;
            serde_json::to_writer_pretty(&mut file, &profiles).map_err(|e| e.to_string())?;
            println!(
                "Profile templates written; query/import model catalogs and verify permission capabilities before running."
            );
            Ok(())
        }
        [command, path, repo, dir, approval, base] if command == "startup-run" => {
            let plan = read_plan(path)?;
            if plan_digest(&plan)? != *approval {
                return Err("explicit approval of the exact plan digest is required".into());
            }
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let report = runtime.block_on(workflow::run_reviewed(
                plan,
                Path::new(repo),
                Path::new(dir),
                1800,
                base,
            ))?;
            print_report(&report)?;
            if matches!(report.state.as_str(), "failed" | "blocked") {
                return Err("workflow failed; inspect retained artifacts".into());
            }
            Ok(())
        }
        [command, path, repo, dir, approval] if command == "run" => {
            let plan = read_plan(path)?;
            if plan_digest(&plan)? != *approval {
                return Err("explicit approval of the exact plan digest is required".into());
            }
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let report =
                runtime.block_on(workflow::run(plan, Path::new(repo), Path::new(dir), 1800))?;
            print_report(&report)?;
            if report.state == "failed" {
                return Err("workflow failed; inspect status, logs, and retained artifacts".into());
            }
            Ok(())
        }
        [command, dir] if command == "status" || command == "approvals" => {
            print_report(&workflow::load(Path::new(dir))?)
        }
        [command, dir] if command == "diff" => {
            let _ = workflow::load(Path::new(dir))?;
            let bytes =
                std::fs::read(Path::new(dir).join("diff.patch")).map_err(|e| e.to_string())?;
            std::io::stdout()
                .write_all(&bytes)
                .map_err(|e| e.to_string())
        }
        [command, dir, task_id] if command == "logs" => {
            let report = workflow::load(Path::new(dir))?;
            let task = report
                .tasks
                .iter()
                .find(|t| t.id == *task_id)
                .ok_or("unknown task ID")?;
            let mut file = std::fs::File::open(task.artifacts.join("process/stdout.jsonl"))
                .map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut std::io::stdout()).map_err(|e| e.to_string())?;
            Ok(())
        }
        [command, dir, candidate] if command == "approve" => {
            workflow::approve(Path::new(dir), candidate)
        }
        [command, dir, reason] if command == "reject" => workflow::reject(Path::new(dir), reason),
        [command, dir] if command == "merge" => workflow::merge(Path::new(dir)),
        [command, dir, query] if command == "recall" => {
            let _ = workflow::load(Path::new(dir))?;
            let store = Store::open(Path::new(dir).join("state.sqlite"))?;
            println!(
                "{}",
                serde_json::to_string(&store.recall(query)?).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        [command, dir, port] if command == "memory-sync" => {
            let _ = workflow::load(Path::new(dir))?;
            let store = Store::open(Path::new(dir).join("state.sqlite"))?;
            let sent = memory::sync(&store, port.parse().map_err(|_| "invalid local port")?)?;
            println!("{{\"delivered\":{sent}}}");
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
