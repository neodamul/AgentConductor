use super::*;

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [command, dir] if command == "resume-due" || command == "watch" => Some((|| {
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
        })()),
        [] => Some((|| {
            let root = agentconductor::desktop::workspace()?;
            execute_args(&["ui-new".into(), root.to_string_lossy().into()])
        })()),
        [flag] if flag == "--help" || flag == "-h" => Some({
            println!(
                "AgentConductor\n\nCommands:\n  ui-new <workspace-dir>\n  ui <run-dir>\n  profiles <profiles.json>\n  models-discover <profiles.json> <agent-id> <new-discovery-dir>\n  catalog-import <profiles.json> <agent-id> <observed-catalog.json>\n  plan <profiles.json> <coordinator-id> <repo> <new-plan-dir> <goal> [roles.json] [--from plan.md] [--rules rules.md] [--phases slug-dir]\n  sessions <run-dir>\n  briefing <run-dir>\n  checkpoint-issue <run-dir> <issue.json>\n  checkpoint-poll <run-dir> <address.json>\n  checkpoint-ack <run-dir> <ack.json>\n  checkpoint-report <run-dir> <progress.json>\n  quota-refresh <run-dir> <agent-id>\n  schedule <run-dir> <task-id> <YYYY-MM-DDTHH:MM:SS+HH:MM>\n  reassign <run-dir> <task-id> <agent-id> [model]\n  session-model <run-dir> <task-id> <model>\n  stop <run-dir> <reason>\n  resume <run-dir>\n  recover <run-dir>\n  resume-due <run-dir>\n  watch <run-dir>\n  validate <plan.json>\n  run <plan.json> <repo> <new-run-dir> <approved-plan-digest> [--phase slug-dir phase-id]\n  status <run-dir>\n  logs <run-dir> <task-id>\n  progress <run-dir> <task-id>\n  diff <run-dir>\n  tool-requests <run-dir>\n  tool-refresh <run-dir> <exact-request-sha256>\n  tool-approve <run-dir> <exact-request-sha256> [lifetime-seconds]\n  tool-reject <run-dir> <exact-request-sha256>\n  approve <run-dir> <exact-candidate-sha>\n  reject <run-dir> <reason>\n  revise <run-dir> <task-id> <feedback>\n  review <run-dir> <coordinator-id>\n  review-apply <run-dir> <coordinator-id>\n  apply-review <run-dir> <exact-proposal-sha256>\n  merge <run-dir>\n  knowledge-digest <lesson.json>\n  knowledge-accept <run-dir> <lesson.json> <exact-digest>\n  knowledge-status <run-dir>\n  recall <run-dir> <query>\n  memory-sync <run-dir> <local-port>\n  memory-publish <local-port> <envelope.json>\n  memory-read <local-port> <memory-id>\n  memory-import <run-dir> <local-port> <memory-id>\n  memory-discover <run-dir> <local-port>\n  normalize <claude|codex|opencode|custom> <stdout.jsonl>\n\nShared workspace is default; worktrees are optional. Run directories must be outside the repository.\nReview profiles, scopes and test commands before passing the validate command's plan digest.\nProfiles need an operator-verified sandbox/permission configuration before execution."
            );
            Ok(())
        }),
        [command, path] if command == "validate" => Some((|| {
            let plan = read_plan(path)?;
            println!(
                "{}",
                serde_json::json!({
                    "valid": true, "tasks": plan.tasks.len(), "workspace_mode": plan.workspace_mode,"plan_digest":plan_digest(&plan)?
                })
            );
            Ok(())
        })()),
        [command, path] if command == "profiles" => Some((|| {
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
                    cooperative: Default::default(),
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
        })()),
        [command, path, repo, dir, approval, base] if command == "startup-run" => Some((|| {
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
        })(
        )),
        [command, path, repo, dir, approval, rest @ ..]
            if command == "run"
                && (rest.is_empty() || (rest.len() == 3 && rest[0] == "--phase")) =>
        {
            Some((|| {
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
                // DD-62: `--phase <slug-dir> <phase-id>` ties the run to a slug phase.
                if let [_, root, phase] = rest {
                    workflow::link_phase(Path::new(dir), Path::new(root), phase)?;
                }
                print_report(&report)?;
                if report.state == "failed" {
                    return Err(
                        "workflow failed; inspect status, logs, and retained artifacts".into(),
                    );
                }
                Ok(())
            })())
        }
        [command, dir] if command == "status" || command == "approvals" => {
            Some((|| print_report(&workflow::load(Path::new(dir))?))())
        }
        [command, dir] if command == "diff" => Some((|| {
            let _ = workflow::load(Path::new(dir))?;
            let bytes =
                std::fs::read(Path::new(dir).join("diff.patch")).map_err(|e| e.to_string())?;
            std::io::stdout()
                .write_all(&bytes)
                .map_err(|e| e.to_string())
        })()),
        [command, dir, task_id] if command == "logs" => Some((|| {
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
        })()),
        [command, dir, candidate] if command == "approve" => {
            Some(workflow::approve(Path::new(dir), candidate))
        }
        [command, dir, reason] if command == "reject" => {
            Some(workflow::reject(Path::new(dir), reason))
        }
        [command, dir] if command == "merge" => Some(workflow::merge(Path::new(dir))),
        [command, dir, query] if command == "recall" => Some((|| {
            let _ = workflow::load(Path::new(dir))?;
            let store = Store::open(Path::new(dir).join("state.sqlite"))?;
            println!(
                "{}",
                serde_json::to_string(&store.recall(query)?).map_err(|e| e.to_string())?
            );
            Ok(())
        })()),
        [command, dir, port] if command == "memory-sync" => Some((|| {
            let _ = workflow::load(Path::new(dir))?;
            let store = Store::open(Path::new(dir).join("state.sqlite"))?;
            let sent = memory::sync(&store, port.parse().map_err(|_| "invalid local port")?)?;
            println!("{{\"delivered\":{sent}}}");
            Ok(())
        })()),
        [command, provider, path] if command == "normalize" => Some((|| {
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
        })()),
        _ => None,
    }
}
