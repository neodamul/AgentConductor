use crate::protocol::Provider;

#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    Stdin,
    File,
}

pub struct Invocation {
    pub executable: String,
    pub args: Vec<String>,
    pub input: InputMode,
}

/// Builds provider argv only. Permission capabilities must be checked before execution.
/// In particular, a resumed session may inherit different permissions from a new session.
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
