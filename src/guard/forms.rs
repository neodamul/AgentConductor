//! Command spellings the textual check cannot place: the program name is assembled by the
//! shell, or the real command is inline code for an interpreter (GitHub #83).

/// True when the command must ask a human; `names` are its lower-cased program words.
pub(super) fn needs_human(command: &str, names: &[String]) -> bool {
    glued_quotes(command) || expands(command) || inline_code(names)
}

/// `c''url`, `"cu"rl`: a quote that starts or ends in the middle of a word.
fn glued_quotes(command: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':');
    let mut open = None;
    let mut previous = ' ';
    for c in command.chars() {
        match (open, c) {
            (None, '\'' | '"') => {
                if word(previous) {
                    return true;
                }
                open = Some(c);
            }
            (Some(q), c) if c == q => open = None,
            (None, c) if word(c) && matches!(previous, '\'' | '"') => return true,
            _ => {}
        }
        previous = c;
    }
    false
}

/// `${x}rl`, `$x`, `$(…)`: expansion decides what runs.
fn expands(command: &str) -> bool {
    let mut single = false;
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => single = !single,
            '$' if !single
                && chars
                    .peek()
                    .is_some_and(|n| n.is_alphabetic() || matches!(n, '_' | '{' | '(')) =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn inline_code(names: &[String]) -> bool {
    names.iter().enumerate().any(|(i, name)| {
        let rest = &names[i + 1..];
        let flag = |wanted: &[&str]| rest.iter().any(|w| wanted.contains(&w.as_str()));
        match name.as_str() {
            "sh" | "bash" | "zsh" | "dash" | "ksh" | "fish" => rest
                .iter()
                .any(|w| w.starts_with('-') && !w.starts_with("--") && w.contains('c')),
            "cmd" => flag(&["/c", "/k"]),
            "pwsh" | "powershell" => flag(&["-command", "-c"]),
            "node" | "nodejs" => flag(&["-e", "--eval", "-p", "--print"]),
            "perl" => flag(&["-e", "-E"]),
            "ruby" => flag(&["-e"]),
            "php" => flag(&["-r", "-s"]),
            "deno" => flag(&["eval"]),
            n if n.starts_with("python") || n == "py" => {
                flag(&["-c"])
                    || rest.windows(2).any(|p| {
                        p[0] == "-m"
                            && matches!(
                                p[1].as_str(),
                                "http.server" | "http" | "socketserver" | "simplehttpserver"
                            )
                    })
            }
            _ => false,
        }
    })
}
