/// DD-94: screens that need the operator; the collector does not answer them.
pub fn claude_blocker(text: &str) -> Option<&'static str> {
    if text.contains("Let's get started.") || text.contains("Choose the text style") {
        return Some(
            "Claude CLI first-run setup is not finished on this computer; run claude once in a terminal, complete setup and sign in, then refresh",
        );
    }
    let text = text.to_ascii_lowercase();
    if [
        "not logged in",
        "please run /login",
        "select login method",
        "oauth token has expired",
    ]
    .iter()
    .any(|screen| text.contains(screen))
    {
        return Some(
            "Claude is not signed in or the login expired; run claude in a terminal, sign in with /login, then refresh",
        );
    }
    None
}
