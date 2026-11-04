//! Standalone Room document with the same trusted assets and policy as Workspace.
pub(super) fn room() -> Result<String, String> {
    let css = super::CSS;
    let i18n = super::I18N;
    let js = super::JS;
    let brand_script = include_str!("brand_avatar.js");
    let nonce = crate::native_guard::session_id()?;
    let policy = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'nonce-{nonce}'; style-src 'nonce-{nonce}'; connect-src 'none'; img-src data:; base-uri 'none'; form-action 'none'\">"
    );
    Ok(super::language::embed(super::HTML)
        .replace(
            "<body>",
            &format!("<body>{}", crate::icons::brand_templates()),
        )
        .replace(
            "<link rel=\"stylesheet\" href=\"/ui.css\"><script src=\"/ui.js\" defer></script>",
            &format!("{policy}<style nonce=\"{nonce}\">{css}</style>"),
        )
        .replace(
            "</body>",
            &format!("<script nonce=\"{nonce}\">{i18n}\n{brand_script}\n{js}</script></body>"),
        )
        .replace("{{APP_VERSION}}", &crate::update::current()))
}
