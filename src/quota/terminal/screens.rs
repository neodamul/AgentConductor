//! What the Claude terminal is showing: trust screen, blockers and the ready prompt.

pub(super) fn plain(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 27 {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            if bytes[i] == b'[' {
                i += 1;
                let first = i;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&b) {
                        // A terminal that draws a space as "cursor forward n" leaves no blank to read.
                        if b == b'C' {
                            let count = std::str::from_utf8(&bytes[first..i - 1])
                                .ok()
                                .and_then(|n| n.parse::<usize>().ok())
                                .unwrap_or(1);
                            out.extend(std::iter::repeat_n(b' ', count.min(200)));
                        }
                        break;
                    }
                }
            } else if bytes[i] == b']' {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 7 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == 27 && bytes.get(i + 1) == Some(&b'\\') {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            } else {
                while i < bytes.len() && (0x20..=0x2f).contains(&bytes[i]) {
                    i += 1;
                }
                i += 1;
            }
        } else {
            if bytes[i] >= 32 || bytes[i] == b'\n' {
                out.push(bytes[i]);
            }
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Claude's folder-trust screen. The usage session runs with no tools, hooks or setting
/// sources, so it is accepted with its default "Yes, proceed" (user, 2026-10-13).
pub fn claude_trust_prompt(text: &str) -> bool {
    text.contains("Permission Required:")
        || text.contains("Do you trust the files")
        || text.contains("Yes, I trust this folder")
        || yes_no(text)
}

/// Screen-reader mode asks "Enter y/n:" and repeats it until y or n is typed.
fn yes_no(text: &str) -> bool {
    text.contains("Enter y/n") || text.contains("Please answer y or n")
}

/// The keys that accept the trust screen: Enter takes the default "Yes, proceed", and the
/// screen-reader form needs an explicit y.
pub fn claude_trust_answer(text: &str) -> &'static [u8] {
    if yes_no(text) { b"y\r" } else { b"\r" }
}

/// The trust screen is answered at most this many times before the query gives up.
pub(crate) const TRUST_ANSWERS: u8 = 2;
pub(crate) const TRUST_REFUSED: &str =
    "Claude kept asking to trust the project folder; open claude there once and accept it";

/// DD-94: screens that need the operator; the collector does not answer them.
pub fn claude_blocker(text: &str) -> Option<&'static str> {
    if text.contains("Let's get started.") || text.contains("Choose the text style") {
        return Some(
            "Claude CLI first-run setup is not finished on this computer; run claude once in a terminal, complete setup and sign in, then refresh",
        );
    }
    if [
        "Not logged in",
        "Please run /login",
        "Select login method",
        "OAuth token has expired",
    ]
    .iter()
    .any(|screen| text.contains(screen))
    {
        return Some(SIGNED_OUT);
    }
    None
}

/// GitHub #317: a login that expired and one never made are told the same way.
const SIGNED_OUT: &str = "Claude is not signed in or the login expired; run claude in a terminal, sign in with /login, then refresh";

/// Version-sensitive screen-reader metadata; unrelated statistics never count as capacity.
pub fn claude_usage_ready(text: &str) -> bool {
    !claude_trust_prompt(text)
        && claude_blocker(text).is_none()
        && text.contains("Claude Code v")
        && text.lines().any(|line| {
            let line = line.trim();
            ["$", ">", "❯"].iter().any(|prompt| {
                line.strip_prefix(prompt)
                    .is_some_and(|tail| tail.is_empty() || tail.starts_with(' '))
            })
        })
}
