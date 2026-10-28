use crate::agent_settings::{ArgsEdit, Resume, preset, settings_for};
use crate::protocol::Provider;

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
    // A custom profile is declarative: its own placeholders already say where model and session go.
    let Some(settings) = settings_for(profile.parser, profile.agent_settings.as_ref())? else {
        return Ok(args);
    };
    let flags = &settings.invocation.session;
    let fill = |template: &[String], key: &str, value: &str| -> Vec<String> {
        template.iter().map(|a| a.replace(key, value)).collect()
    };
    if let Some(model) = model {
        let flag = flags.model_args.first();
        if args.iter().any(|a| {
            flag.is_some_and(|f| a == f || a.starts_with(&format!("{f}=")))
                || flags.model_aliases.contains(a)
        }) {
            return Err("duplicate model selector in profile".into());
        }
        args.extend(fill(&flags.model_args, "{model}", model));
    }
    let Some(session) = session else {
        return Ok(args);
    };
    if args.iter().any(|a| flags.selectors.contains(a)) {
        return Err("ambiguous session selector in profile".into());
    }
    match &flags.resume {
        Some(Resume::Append { args: extra }) => args.extend(fill(extra, "{session_id}", session)),
        // The resume subcommand takes the place of the whole argv, so the profile must still be a verified template.
        Some(Resume::ReplaceArgv {
            argv,
            full_access_argv,
            ..
        }) => {
            let full = full_access_of(&settings);
            let initial = fill(&settings.invocation.args, "{task_file}", task_file);
            args = if full_access_argv.is_some() && full.as_ref() == Some(&profile.args) {
                fill(
                    full_access_argv.as_deref().unwrap_or_default(),
                    "{session_id}",
                    session,
                )
            } else if profile.args == initial {
                fill(argv, "{session_id}", session)
            } else {
                let label = settings.label.as_deref().unwrap_or("This agent");
                return Err(format!(
                    "{label} resume requires the verified initial argv template"
                ));
            };
            if let Some(model) = model {
                args.extend(fill(&flags.model_args, "{model}", model));
            }
        }
        None => return Err("this agent cannot resume a session".into()),
    }
    Ok(args)
}

fn full_access_of(settings: &crate::agent_settings::AgentSettings) -> Option<Vec<String>> {
    Some(match settings.invocation.full_access.as_ref()? {
        ArgsEdit::Append { args } => settings
            .invocation
            .args
            .iter()
            .chain(args)
            .cloned()
            .collect(),
        ArgsEdit::ReplaceArgv { argv } => argv.clone(),
    })
}

/// The argument list that gives a built-in agent full access (DD-131), from its preset.
pub fn full_access_args(provider: Provider) -> Option<Vec<String>> {
    full_access_of(preset(provider)?)
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
    let settings =
        preset(provider).ok_or("custom agents require an explicitly registered CLI profile")?;
    let plan = &settings.invocation;
    if plan.input == InputMode::File && (task_file.is_empty() || task_file.starts_with('-')) {
        return Err("invalid task file argument".into());
    }
    let fill = |template: &[String], key: &str, value: &str| -> Vec<String> {
        template.iter().map(|a| a.replace(key, value)).collect()
    };
    let mut args = fill(&plan.args, "{task_file}", task_file);
    if let Some(session) = session {
        match &plan.session.resume {
            Some(Resume::Append { args: extra }) => {
                args.extend(fill(extra, "{session_id}", session))
            }
            Some(Resume::ReplaceArgv {
                argv, bare_argv, ..
            }) => {
                args = fill(
                    bare_argv.as_deref().unwrap_or(argv),
                    "{session_id}",
                    session,
                );
            }
            None => {}
        }
    }
    Ok(Invocation {
        executable: plan.executable.clone().unwrap_or_default(),
        args,
        input: plan.input,
    })
}

/// DD-133: only app-bound ordinary folders bypass a CLI's Git-root prerequisite (the agent's `direct_folder` setting).
pub fn direct_folder_args(
    provider: Provider,
    mut args: Vec<String>,
) -> Result<Vec<String>, String> {
    let Some(folder) = preset(provider).and_then(|s| s.invocation.direct_folder.as_ref()) else {
        return Ok(args);
    };
    if !folder
        .skip_if_present
        .iter()
        .any(|flag| args.contains(flag))
    {
        let anchor = args
            .iter()
            .position(|arg| *arg == folder.after)
            .ok_or_else(|| {
                crate::ui::messages::text(
                    folder
                        .missing_anchor_message
                        .as_deref()
                        .unwrap_or("folder.codex_exec_required"),
                )
                .to_string()
            })?;
        for (offset, extra) in folder.args.iter().enumerate() {
            args.insert(anchor + 1 + offset, extra.clone());
        }
    }
    Ok(args)
}
