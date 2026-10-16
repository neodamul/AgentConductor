use serde_json::Value;

pub(in crate::native_ui_probe::room::task_activity) fn tool_line(
    name: &Value,
    input: &Value,
) -> String {
    let argument = [
        "command",
        "file_path",
        "filePath",
        "path",
        "pattern",
        "url",
        "query",
    ]
    .iter()
    .find_map(|key| input[*key].as_str())
    .unwrap_or_default();
    format!(
        "● {}({})",
        name.as_str().unwrap_or("tool"),
        argument
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect::<String>()
    )
}

pub(super) fn suffix(value: &Value) -> String {
    value
        .as_str()
        .filter(|value| !value.is_empty())
        .map(|value| format!("\n{}", clip(value, 4000)))
        .unwrap_or_default()
}

pub(super) fn result_text(value: &Value) -> String {
    if let Some(value) = value.as_str() {
        return value.into();
    }
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn clip_lines_in_language(value: &Value, maximum: usize, language: &str) -> String {
    let text = result_text(value);
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = lines
        .iter()
        .take(maximum)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    if lines.len() > maximum {
        output.push_str(&crate::ui::messages::interpolate(
            crate::ui::messages::in_language(language, "room.more_lines"),
            &[("value1", &(lines.len() - maximum).to_string())],
        ));
    }
    clip(&output, 4000)
}

pub(super) fn clip(value: &str, maximum: usize) -> String {
    let mut chars = value.chars();
    let output = chars.by_ref().take(maximum).collect::<String>();
    if chars.next().is_some() {
        format!("{output}…")
    } else {
        output
    }
}
