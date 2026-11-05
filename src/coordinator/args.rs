use crate::{adapters, plan::AgentProfile, protocol::Provider};

pub fn read_only_args(profile: &AgentProfile, task_file: &str) -> Result<Vec<String>, String> {
    // GitHub #326: an operator who declares how the CLI runs read-only is trusted for that CLI; the
    // normal arguments are never accepted as the read-only ones.
    if let Some(declared) = &profile.read_only_args {
        if declared.is_empty() || *declared == profile.args {
            return Err(crate::ui::messages::render(
                "native.coordinator_bad_read_only_args",
                &[("id", &profile.id)],
            ));
        }
        let mut readonly = profile.clone();
        readonly.args = declared.clone();
        readonly.read_only_args = None;
        return adapters::session_args(
            &readonly,
            profile.coordinator_model.as_deref(),
            None,
            task_file,
        );
    }
    // DD-131: a full-access agent still coordinates read-only, through its CLI's verified
    // sandboxed template restricted below.
    if profile.permission_mode == "full_access"
        && adapters::full_access_args(profile.parser).as_ref() == Some(&profile.args)
    {
        let mut restricted = profile.clone();
        restricted.args = adapters::invocation(profile.parser, None, "{task_file}")?.args;
        restricted.permission_mode = match profile.parser {
            Provider::Codex => "native_sandbox",
            _ => "native_callback",
        }
        .into();
        return read_only_args(&restricted, task_file);
    }
    let mut args = adapters::session_args(
        profile,
        profile.coordinator_model.as_deref(),
        None,
        task_file,
    )?;
    if profile.permission_mode == "native_sandbox"
        && profile.parser == Provider::Codex
        && crate::executable::program_name(&profile.executable) == "codex"
    {
        if adapters::session_args(profile, None, None, task_file)?
            != adapters::invocation(Provider::Codex, None, task_file)?.args
        {
            return Err("coordinator requires the verified native Codex template".into());
        }
        args[4] = "read-only".into();
    } else if profile.permission_mode == "native_callback" {
        // Reuse the pinned guarded Claude template, exposing only read-only tools.
        crate::native_guard::validate(profile)?;
        args.extend(
            [
                "--tools",
                "Read,Grep,Glob",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--no-chrome",
                "--disable-slash-commands",
            ]
            .map(String::from),
        );
    } else if profile.permission_mode != "read_only" {
        return Err(crate::ui::messages::render(
            "native.coordinator_not_read_only",
            &[("id", &profile.id), ("mode", &profile.permission_mode)],
        ));
    }
    Ok(args)
}

/// Planning runs in a disposable snapshot, so a coordinator whose read-only behavior
/// cannot be verified may plan; verified templates keep their own restrictions.
pub fn planning_args(profile: &AgentProfile, task_file: &str) -> Result<Vec<String>, String> {
    read_only_args(profile, task_file).or_else(|_| {
        adapters::session_args(
            profile,
            profile.coordinator_model.as_deref(),
            None,
            task_file,
        )
    })
}
