use crate::{
    agent_settings::{Resume, full_access_args as settings_full_access_args, preset, settings_for},
    protocol::Provider,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    #[default]
    Stdin,
    File,
}

pub struct Invocation {
    pub executable: String,
    pub args: Vec<String>,
    pub input: InputMode,
}

pub fn session_args(
    profile: &crate::plan::AgentProfile,
    model: Option<&str>,
    session: Option<&str>,
    task_file: &str,
) -> Result<Vec<String>, String> {
    session_args_with_options(profile, model, session, task_file, &Default::default())
}

pub fn session_args_with_options(
    profile: &crate::plan::AgentProfile,
    model: Option<&str>,
    session: Option<&str>,
    task_file: &str,
    overrides: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<String>, String> {
    let mut args = base_session_args(profile, model, session, task_file)?;
    let effective_model = overrides
        .get("model")
        .map(String::as_str)
        .or(model)
        .or_else(|| {
            profile
                .execution_options
                .iter()
                .find(|option| option.name == "model")
                .and_then(|option| option.default.as_deref())
        });
    if let Some(model) = effective_model
        && profile
            .execution_options
            .iter()
            .any(|option| option.name == "model")
    {
        profile.validate_model(model)?;
    }
    for option in &profile.execution_options {
        let value = match option.name.as_str() {
            "model" => effective_model,
            "resume" | "session" => session,
            name => overrides
                .get(name)
                .map(String::as_str)
                .or(option.default.as_deref()),
        };
        let Some(value) = value else { continue };
        if option.name == "model" {
            profile.validate_model(value)?;
        } else if option.name != "resume" && option.name != "session" {
            let allowed = option.allowed_values(effective_model).ok_or_else(|| {
                format!("option group {} is unavailable for this model", option.name)
            })?;
            if !allowed.iter().any(|allowed| allowed == value) {
                return Err(format!(
                    "invalid value for CLI option group {}",
                    option.name
                ));
            }
        }
        for template in &option.args {
            args.push(
                template
                    .replace("{value}", value)
                    .replace("{model}", effective_model.unwrap_or(value))
                    .replace("{session_id}", session.unwrap_or(value))
                    .replace("{task_file}", task_file),
            );
        }
    }
    Ok(args)
}

fn base_session_args(
    profile: &crate::plan::AgentProfile,
    model: Option<&str>,
    session: Option<&str>,
    task_file: &str,
) -> Result<Vec<String>, String> {
    if let Some(model) = model {
        profile.validate_model(model)?;
    }
    if let Some(id) = session
        && (id.is_empty()
            || id.starts_with('-')
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.')))
    {
        return Err("invalid explicit session ID".into());
    }
    let mut args: Vec<_> = profile
        .args
        .iter()
        .map(|a| {
            a.replace("{task_file}", task_file)
                .replace("{model}", model.unwrap_or(""))
                .replace("{session_id}", session.unwrap_or(""))
        })
        .collect();
    let Some(settings) = settings_for(profile.parser, profile.agent_settings.as_ref())? else {
        return Ok(args);
    };
    let flags = &settings.invocation.session;
    if let Some(model) = model {
        let flag = flags.model_args.first();
        if args.iter().any(|a| {
            flag.is_some_and(|f| a == f || a.starts_with(&format!("{f}=")))
                || flags.model_aliases.contains(a)
        }) {
            return Err("duplicate model selector in profile".into());
        }
        args.extend(
            flags
                .model_args
                .iter()
                .map(|arg| arg.replace("{model}", model)),
        );
    }
    let Some(session) = session else {
        return Ok(args);
    };
    if args.iter().any(|arg| flags.selectors.contains(arg)) {
        return Err("ambiguous session selector in profile".into());
    }
    match &flags.resume {
        Some(Resume::Append { args: extra }) => {
            args.extend(extra.iter().map(|arg| arg.replace("{session_id}", session)))
        }
        Some(Resume::ReplaceArgv {
            argv,
            full_access_argv,
            ..
        }) => {
            let full = settings_full_access_args(&settings);
            let initial = settings
                .invocation
                .args
                .iter()
                .map(|arg| arg.replace("{task_file}", task_file))
                .collect::<Vec<_>>();
            args = if full_access_argv.is_some() && full.as_ref() == Some(&profile.args) {
                full_access_argv
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(|arg| arg.replace("{session_id}", session))
                    .collect()
            } else if profile.args == initial {
                argv.iter()
                    .map(|arg| arg.replace("{session_id}", session))
                    .collect()
            } else {
                let label = settings.label.as_deref().unwrap_or("This agent");
                return Err(format!(
                    "{label} resume requires the verified initial argv template"
                ));
            };
            if let Some(model) = model {
                args.extend(
                    flags
                        .model_args
                        .iter()
                        .map(|arg| arg.replace("{model}", model)),
                );
            }
        }
        None => return Err("this agent cannot resume a session".into()),
    }
    Ok(args)
}

/// Builds provider argv only. Permission capabilities must be checked before execution.
/// In particular, a resumed session may inherit different permissions from a new session.
/// DD-131 (operator decision 2026-10-13): agent CLIs run with full access by default —
/// Claude bypasses its permission prompts and loads the operator's own MCP servers (for
/// example Playwright for browser checks); Codex runs without approvals or sandbox.
pub fn full_access_args(provider: Provider) -> Option<Vec<String>> {
    settings_full_access_args(preset(provider)?)
}

pub fn invocation(
    provider: Provider,
    session: Option<&str>,
    task_file: &str,
) -> Result<Invocation, String> {
    if session.is_some_and(|session| {
        session.is_empty()
            || session.starts_with('-')
            || !session
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
    }) {
        return Err("invalid explicit session ID".into());
    }
    let (executable, mut args, input) = match provider {
        Provider::Claude => (
            "claude",
            vec!["-p", "--output-format", "stream-json", "--verbose"],
            InputMode::Stdin,
        ),
        Provider::Codex if session.is_some() => (
            "codex",
            vec!["exec", "resume", session.unwrap(), "-", "--json"],
            InputMode::Stdin,
        ),
        Provider::Codex => (
            "codex",
            vec!["exec", "-", "--json", "--sandbox", "workspace-write"],
            InputMode::Stdin,
        ),
        Provider::OpenCode => {
            if task_file.is_empty() || task_file.starts_with('-') {
                return Err("invalid task file argument".into());
            }
            (
                "opencode",
                vec![
                    "run",
                    "--format",
                    "json",
                    "--file",
                    task_file,
                    "Follow the attached task scope and completion conditions",
                ],
                InputMode::File,
            )
        }
        Provider::Custom => {
            return Err("custom agents require an explicitly registered CLI profile".into());
        }
    };
    if let Some(session) = session {
        match provider {
            Provider::Claude => args.extend(["--resume", session]),
            Provider::OpenCode => args.extend(["--session", session]),
            _ => {}
        }
    }
    Ok(Invocation {
        executable: executable.into(),
        args: args.into_iter().map(String::from).collect(),
        input,
    })
}

/// DD-133: only app-bound ordinary folders bypass Codex's Git-root prerequisite.
pub fn direct_folder_args(
    provider: Provider,
    mut args: Vec<String>,
) -> Result<Vec<String>, String> {
    if provider == Provider::Codex && !args.iter().any(|arg| arg == "--skip-git-repo-check") {
        let exec = args
            .iter()
            .position(|arg| arg == "exec")
            .ok_or(crate::ui::messages::text("folder.codex_exec_required"))?;
        args.insert(exec + 1, "--skip-git-repo-check".into());
    }
    Ok(args)
}
