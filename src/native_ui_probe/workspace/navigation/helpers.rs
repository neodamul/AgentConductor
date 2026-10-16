pub(super) fn display_path(path: &str) -> String {
    crate::git::git_path(std::path::Path::new(path).as_os_str())
        .to_string_lossy()
        .into_owned()
}

pub(super) fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

pub(super) fn message(language: &str, id: &'static str, params: &[(&str, &str)]) -> String {
    crate::ui::messages::interpolate(text(language, id), params)
}
