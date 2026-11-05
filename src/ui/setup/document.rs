//! Embed the locale catalogs and UI assets into the native document.
pub(crate) fn document() -> Result<String, String> {
    let nonce = crate::native_guard::session_id()?;
    let home = crate::icons::brand_templates()
        + include_str!("../startup.html")
        + include_str!("../mcp.html")
        + include_str!("../chat.html")
        + include_str!("../optimize.html")
        + include_str!("../about.html");
    let style = format!(
        "{}\n{}\n{}\n{}\n{}",
        super::super::CSS,
        include_str!("../startup.css"),
        include_str!("../mcp.css"),
        include_str!("../chat.css"),
        include_str!("../optimize.css")
    ) + include_str!("../anchored_select.css")
        + include_str!("../role_choices.css")
        + include_str!("../about.css")
        + include_str!("../titlebar.css");
    let helper_scripts = concat!(
        include_str!("../dialog_focus.js"),
        include_str!("../brand_avatar.js"),
        include_str!("../team_briefing.js"),
        include_str!("../about.js"),
        "\n",
        include_str!("../manual.js")
    );
    let script = include_str!("../role_choices.js").to_string()
        + &format!(
            "{}\nwindow.startRoom=function(){{ {} }};\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            super::super::I18N,
            super::super::JS,
            include_str!("../mcp.js"),
            include_str!("../chat.js"),
            include_str!("../optimize.js"),
            include_str!("../result_view.js"),
            include_str!("../goal_rows.js"),
            include_str!("../action_icons.js"),
            include_str!("../stepper.js"),
            include_str!("../rail.js"),
            include_str!("../agent_status.js"),
            include_str!("../quota_ring.js"),
            helper_scripts,
            include_str!("../startup.js")
        )
        + include_str!("../anchored_select.js");
    let policy = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'nonce-{nonce}'; style-src 'nonce-{nonce}'; connect-src 'none'; img-src data:; base-uri 'none'; form-action 'none'\">"
    );
    Ok(super::super::language::embed(super::super::HTML).replace("<link rel=\"stylesheet\" href=\"/ui.css\"><script src=\"/ui.js\" defer></script>",&format!("{policy}<style nonce=\"{nonce}\">{style}</style>"))
        .replace("<body>",&format!("<body><section id=\"startup-screen\">{home}</section><section id=\"run-screen\" hidden><button id=\"startup-home\" class=\"small-button room-home\"><span data-i18n=\"startup.workspace\"></span></button>"))
        .replace("</body>",&format!("</section><script nonce=\"{nonce}\">{script}</script></body>"))
        .replace("{{APP_VERSION}}",&crate::update::current()))
}
