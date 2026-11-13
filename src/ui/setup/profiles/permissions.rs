use super::*;

pub(crate) fn derive_permission(profile: &mut AgentProfile) {
    // DD-131: the standard Claude and Codex invocations run with full access by default;
    // a profile still on the earlier sandboxed template is raised to it.
    let program = crate::executable::program_name(&profile.executable);
    let known = matches!(
        (profile.parser, program.as_str()),
        (crate::protocol::Provider::Claude, "claude") | (crate::protocol::Provider::Codex, "codex")
    );
    if known
        && profile.parser == crate::protocol::Provider::Claude
        && profile.permission_mode == "native_callback"
    {
        let Ok(template) = crate::adapters::invocation(profile.parser, None, "{task_file}") else {
            return;
        };
        let pinned = template.args;
        if profile.args == pinned
            || crate::adapters::full_access_args(profile.parser).as_ref() == Some(&profile.args)
        {
            profile.args = pinned;
        }
        return;
    }
    if known && let Some(full) = crate::adapters::full_access_args(profile.parser) {
        let sandboxed = crate::adapters::invocation(profile.parser, None, "{task_file}")
            .is_ok_and(|template| template.args == profile.args);
        if sandboxed {
            profile.args = full.clone();
        }
        if profile.args == full {
            profile.permission_mode = "full_access".into();
            return;
        }
    } else if crate::adapters::full_access_args(profile.parser).as_ref() == Some(&profile.args) {
        // Another executable never inherits full access: it falls back to the sandboxed
        // template and its permission is derived as for any other CLI.
        if let Ok(template) = crate::adapters::invocation(profile.parser, None, "{task_file}") {
            profile.args = template.args;
        }
    }
    let codex = crate::executable::program_name(&profile.executable) == "codex"
        && crate::adapters::invocation(crate::protocol::Provider::Codex, None, "{task_file}")
            .is_ok_and(|template| template.args == profile.args);
    profile.permission_mode = if codex {
        "native_sandbox"
    } else if crate::native_guard::validate(profile).is_ok() {
        "native_callback"
    } else if matches!(
        profile.permission_mode.as_str(),
        "read_only" | "external_sandbox"
    ) {
        return;
    } else {
        // DD-141 amendment (user, 2026-10-14): any other CLI runs as approving its own tools,
        // without an operator checkbox, in line with full access by default.
        "operator_auto_approve"
    }
    .into();
}
