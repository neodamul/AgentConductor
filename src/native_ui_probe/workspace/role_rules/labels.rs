pub(super) fn compact_label(value: &str) -> String {
    let mut chars = value.chars();
    let compact = chars.by_ref().take(8).collect::<String>();
    if chars.next().is_some() {
        format!("{compact}…")
    } else {
        compact
    }
}

pub(super) fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

pub(super) fn alternate_word(language: &str) -> &'static str {
    if language == "ko" {
        "대체"
    } else {
        "alternate"
    }
}

pub(super) fn role_name(language: &str, role: &'static str) -> &'static str {
    if language != "ko" {
        return role;
    }
    crate::ui::messages::in_language(
        language,
        match role {
            "verification" => "startup.verification",
            "development" => "startup.development",
            "documentation" => "startup.documentation",
            "knowledge" => "startup.knowledge",
            "testing" => "startup.testing",
            _ => return role,
        },
    )
}
