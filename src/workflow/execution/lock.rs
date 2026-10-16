use super::*;

pub(crate) struct Lock {
    _lease: crate::leases::Lease,
}
impl Lock {
    pub(crate) fn acquire(dir: &Path) -> Result<Self, String> {
        let lock = Self {
            _lease: crate::leases::Lease::acquire(&dir.join("coordinator.lock"))?,
        };
        if let Some((root, expected)) = REVIEWED_REVISION.get()
            && (&dir.canonicalize().map_err(|e| e.to_string())? != root
                || &revision(&load(dir)?)? != expected)
        {
            return Err("run changed since review; refresh before acting".into());
        }
        Ok(lock)
    }
}

pub(crate) fn validate_permissions(plan: &Plan) -> Result<(), String> {
    let mut reachable: std::collections::HashSet<&str> =
        plan.tasks.iter().map(|task| task.agent.as_str()).collect();
    loop {
        let previous = reachable.len();
        for profile in &plan.agents {
            if reachable.contains(profile.id.as_str())
                && let Some(fallback) = &profile.fallback
            {
                reachable.insert(fallback.agent.as_str());
            }
        }
        if previous == reachable.len() {
            break;
        }
    }
    for agent in &plan.agents {
        if !reachable.contains(agent.id.as_str()) {
            continue;
        }
        if !matches!(
            agent.permission_mode.as_str(),
            "external_sandbox"
                | "read_only"
                | "native_sandbox"
                | "native_callback"
                | "full_access"
                | "operator_auto_approve"
        ) {
            return Err(format!(
                "profile {} needs a verified permission mode; unrestricted CLI execution is unsupported; save the agent list once so its permission is derived",
                agent.id
            ));
        }
        if agent.permission_mode == "native_callback" {
            crate::native_guard::validate(agent)?;
        }
        // DD-131: full access is the operator's default, but only through the exact
        // full-access invocation of the CLI the profile names.
        let full_access = agent.permission_mode == "full_access";
        if full_access
            && crate::adapters::full_access_args(agent.parser).as_ref() != Some(&agent.args)
        {
            return Err(format!(
                "profile {} is full_access but does not use the full-access invocation of its CLI",
                agent.id
            ));
        }
        if !full_access
            && agent
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
            && !(crate::executable::program_name(&agent.executable) == "codex"
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
