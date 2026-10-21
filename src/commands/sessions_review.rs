use super::*;

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [command, dir] if command == "tool-requests" => Some((|| {
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
        })()),
        [command, dir, id] if command == "tool-approve" || command == "tool-reject" => {
            Some((|| {
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
            })())
        }
        [command, dir, id, seconds] if command == "tool-approve" => Some((|| {
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
        })()),
        [command, dir, id] if command == "tool-refresh" => Some((|| {
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
        })()),
        [command, dir, task] if command == "tool-context" => Some((|| {
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
        })()),
        [command, dir, coordinator] if command == "review" || command == "review-apply" => {
            Some((|| {
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
            })())
        }
        [command, dir, proposal] if command == "apply-review" => Some((|| {
            workflow::apply_review(Path::new(dir), proposal)?;
            print_report(&workflow::load(Path::new(dir))?)
        })()),
        [command, dir, task, feedback] if command == "revise" => Some((|| {
            workflow::revise(Path::new(dir), task, feedback)?;
            print_report(&workflow::load(Path::new(dir))?)
        })()),
        [command, dir] if command == "recover" => {
            Some((|| print_report(&workflow::recover(Path::new(dir))?))())
        }
        [command, dir, reason] if command == "stop" => Some((|| {
            workflow::stop(Path::new(dir), reason)?;
            println!("Stop request retained; inspect status until the run is stopped.");
            Ok(())
        })()),
        [command, dir] if command == "resume" => Some((|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let report = runtime.block_on(workflow::resume(Path::new(dir), 1800))?;
            print_report(&runtime.block_on(workflow::repair(Path::new(dir), 1800, report))?)
        })()),
        [command, profiles_path, agent_id, dir] if command == "models-discover" => Some((|| {
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
        })(
        )),
        [command, profiles_path, agent_id, catalog_path] if command == "catalog-import" => {
            Some((|| {
                let mut profiles: Vec<AgentProfile> = serde_json::from_slice(
                    &std::fs::read(profiles_path).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let catalog: agentconductor::plan::ModelCatalog = serde_json::from_slice(
                    &std::fs::read(catalog_path).map_err(|e| e.to_string())?,
                )
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
            })())
        }
        [
            command,
            profiles_path,
            coordinator,
            repo,
            dir,
            goal,
            rest @ ..,
        ] if command == "plan" && rest.len() <= 9 => {
            Some((|| {
                // DD-55/DD-60: optional role rules file and `--from <plan.md>`.
                // DD-65: `--rules <plan-rules.md>` may follow as well.
                let (
                    mut roles_path,
                    mut document,
                    mut rules_path,
                    mut phases_root,
                    mut context_path,
                ) = (None, None, None, None, None);
                let mut args = rest.iter();
                while let Some(arg) = args.next() {
                    match arg.as_str() {
                        "--from" => document = args.next(),
                        "--context" => context_path = args.next(),
                        "--rules" => rules_path = args.next(),
                        "--phases" => phases_root = args.next(),
                        _ if roles_path.is_none() && document.is_none() => roles_path = Some(arg),
                        _ => return Err(
                            "usage: plan … [roles.json] [--from <plan.md>] [--rules <rules.md>]"
                                .into(),
                        ),
                    }
                }
                let rules = match rules_path {
                    Some(path) => Some(
                        std::fs::read_to_string(path)
                            .map_err(|e| format!("plan rules unavailable: {e}"))?,
                    ),
                    None => None,
                };
                let roles: agentconductor::roles::RoleRules = match roles_path {
                    Some(path) => {
                        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                            .map_err(|e| format!("invalid role rules: {e}"))?
                    }
                    None => Default::default(),
                };
                let reference = match document {
                    Some(path) => {
                        let bytes = std::fs::read(path)
                            .map_err(|e| format!("plan document unavailable: {e}"))?;
                        if bytes.len() > 64 * 1024 {
                            return Err("plan document exceeds 64 KiB".into());
                        }
                        Some(
                            String::from_utf8(bytes)
                                .map_err(|_| "plan document is not UTF-8 text")?,
                        )
                    }
                    None => None,
                };
                let context = match context_path {
                    Some(path) => {
                        let bytes = agentconductor::mcp::load(Path::new(path))?;
                        if bytes.len() > 64 * 1024 {
                            return Err("source context exceeds 64 KiB".into());
                        }
                        Some(String::from_utf8(bytes).map_err(|_| "source context is not UTF-8")?)
                    }
                    None => None,
                };
                let profiles: Vec<AgentProfile> = serde_json::from_slice(
                    &std::fs::read(profiles_path).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| e.to_string())?;
                let plan = runtime.block_on(agentconductor::planner::generate_from(
                    goal,
                    &profiles,
                    coordinator,
                    Path::new(repo),
                    Path::new(dir),
                    &agentconductor::planner::PlanInputs {
                        roles: &roles,
                        reference: reference.as_deref(),
                        context: context.as_deref(),
                        rules: rules.as_deref(),
                        phases_root: phases_root.map(Path::new),
                    },
                ))?;
                println!(
                    "{}",
                    serde_json::json!({"plan":Path::new(dir).join("plan.json"),"plan_digest":plan_digest(&plan)?})
                );
                Ok(())
            })())
        }
        [command, dir] if command == "scope-failure" => {
            Some(workflow::scope_failure(Path::new(dir)).map(|v| println!("{v}")))
        }
        [command, dir, task, digest, paths] | [command, dir, task, digest, paths, _]
            if command == "scope-recovery" =>
        {
            Some((|| {
                let test = match args.get(5) {
                    Some(text) => Some(
                        serde_json::from_str::<Vec<String>>(text)
                            .map_err(|_| "invalid test command JSON")?,
                    ),
                    None => None,
                };
                let paths: Vec<String> =
                    serde_json::from_str(paths).map_err(|_| "invalid paths JSON")?;
                workflow::recovery_apply(Path::new(dir), task, &paths, test, digest)
            })())
        }
        [command, dir, task, timestamp] if command == "schedule" => {
            Some(workflow::schedule(Path::new(dir), task, timestamp))
        }
        [command, dir, task, model] if command == "session-model" => {
            Some(workflow::set_model(Path::new(dir), task, model))
        }
        [command, dir, task, agent] if command == "reassign" => {
            Some(workflow::reassign(Path::new(dir), task, agent, None))
        }
        [command, dir, task, agent, model] if command == "reassign" => {
            Some(workflow::reassign(Path::new(dir), task, agent, Some(model)))
        }
        [command, dir] if command == "sessions" => Some((|| {
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
        })()),
        _ => None,
    }
}
