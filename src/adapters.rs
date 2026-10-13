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
    if profile.parser == Provider::Custom {
        return Ok(args);
    }
    if let Some(model) = model {
        if args
            .iter()
            .any(|a| a == "--model" || a == "-m" || a.starts_with("--model="))
        {
            return Err("duplicate model selector in profile".into());
        }
        args.extend(["--model".into(), model.into()]);
    }
    if let Some(session) = session {
        if args.iter().any(|a| {
            matches!(
                a.as_str(),
                "--resume" | "--session" | "--continue" | "--last" | "-c"
            )
        }) {
            return Err("ambiguous session selector in profile".into());
        }
        match profile.parser {
            Provider::Claude => args.extend(["--resume".into(), session.into()]),
            Provider::OpenCode => args.extend(["--session".into(), session.into()]),
            // DD-131: full access is likewise an option of `exec`, before `resume`.
            Provider::Codex
                if full_access_args(Provider::Codex).as_ref() == Some(&profile.args) =>
            {
                args = vec![
                    "exec".into(),
                    "--dangerously-bypass-approvals-and-sandbox".into(),
                    "resume".into(),
                    session.into(),
                    "-".into(),
                    "--json".into(),
                ];
                if let Some(model) = model {
                    args.extend(["--model".into(), model.into()]);
                }
            }
            Provider::Codex => {
                let initial = invocation(Provider::Codex, None, task_file)?.args;
                if profile.args != initial {
                    return Err("Codex resume requires the verified initial argv template".into());
                }
                // Sandbox is an option of `exec`, preceding its `resume` subcommand.
                args = vec![
                    "exec".into(),
                    "--sandbox".into(),
                    "workspace-write".into(),
                    "resume".into(),
                    session.into(),
                    "-".into(),
                    "--json".into(),
                ];
                if let Some(model) = model {
                    args.extend(["--model".into(), model.into()]);
                }
            }
            Provider::Custom => unreachable!(),
        }
    }
    Ok(args)
}

/// Builds provider argv only. Permission capabilities must be checked before execution.
/// In particular, a resumed session may inherit different permissions from a new session.
/// DD-131 (operator decision 2026-10-13): agent CLIs run with full access by default —
/// Claude bypasses its permission prompts and loads the operator's own MCP servers (for
/// example Playwright for browser checks); Codex runs without approvals or sandbox.
pub fn full_access_args(provider: Provider) -> Option<Vec<String>> {
    let args: &[&str] = match provider {
        Provider::Claude => &[
            "-p",
            "--output-format",
            "stream-json",
            "--verbose",
            "--permission-mode",
            "bypassPermissions",
        ],
        Provider::Codex => &[
            "exec",
            "-",
            "--json",
            "--dangerously-bypass-approvals-and-sandbox",
        ],
        _ => return None,
    };
    Some(args.iter().map(|a| a.to_string()).collect())
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
