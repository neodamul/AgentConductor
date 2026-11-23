use std::path::Path;

mod inline;

pub(crate) use inline::chat_link;
use inline::{escape, inline};

fn cells(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cell = String::new();
    let mut chars = line.trim().trim_matches('|').chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'|') {
            chars.next();
            cell.push('|');
        } else if ch == '|' {
            result.push(cell.trim().to_owned());
            cell.clear();
        } else {
            cell.push(ch);
        }
    }
    result.push(cell.trim().to_owned());
    result
}

fn list_item(line: &str, chat: bool) -> Option<(&'static str, &str)> {
    let line = line.trim_start();
    line.strip_prefix("- ")
        .map(|text| ("ul", text))
        .or_else(|| {
            if !chat {
                return None;
            }
            if let Some(text) = line.strip_prefix("* ").or_else(|| line.strip_prefix("+ ")) {
                return Some(("ul", text));
            }
            let (number, text) = line.split_once(". ").or_else(|| line.split_once(") "))?;
            (!number.is_empty() && number.len() <= 9 && number.bytes().all(|b| b.is_ascii_digit()))
                .then_some(("ol", text))
        })
}

fn table_line(line: &str, chat: bool) -> bool {
    line.starts_with('|') || (chat && line.contains('|'))
}

fn render_fragment(text: &str, directory: Option<&Path>) -> Result<String, String> {
    let chat = directory.is_none();
    let lines: Vec<_> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with("```") {
            let mut code = Vec::new();
            while i < lines.len() && !lines[i].starts_with("```") {
                code.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            out.push(format!(
                "<pre><code>{}</code></pre>",
                escape(&code.join("\n"))
            ));
        } else if let Some((prefix, heading)) = line.split_once(' ')
            && !prefix.is_empty()
            && prefix.len() <= 6
            && prefix.chars().all(|c| c == '#')
        {
            out.push(format!(
                "<h{}>{}</h{}>",
                prefix.len(),
                inline(heading, directory)?,
                prefix.len()
            ));
        } else if table_line(line, chat)
            && i < lines.len()
            && cells(lines[i]).iter().all(|c| {
                let separator = c.trim_matches(':');
                !separator.is_empty() && separator.chars().all(|x| x == '-')
            })
        {
            out.push(format!(
                "<div class=\"manual-table\"><table><thead><tr>{}</tr></thead><tbody>",
                cells(line)
                    .iter()
                    .map(|c| inline(c, directory).map(|v| format!("<th>{v}</th>")))
                    .collect::<Result::<Vec<_>, _>>()?
                    .join("")
            ));
            i += 1;
            while i < lines.len() && table_line(lines[i], chat) {
                out.push(format!(
                    "<tr>{}</tr>",
                    cells(lines[i])
                        .iter()
                        .map(|c| inline(c, directory).map(|v| format!("<td>{v}</td>")))
                        .collect::<Result::<Vec<_>, _>>()?
                        .join("")
                ));
                i += 1;
            }
            out.push("</tbody></table></div>".into());
        } else if let Some((tag, first)) = list_item(line, chat) {
            out.push(format!("<{tag}>"));
            let mut item_line = first;
            loop {
                let mut item = if chat {
                    item_line
                } else {
                    item_line.trim_start_matches("- ")
                }
                .to_owned();
                while i < lines.len()
                    && lines[i].starts_with("  ")
                    && list_item(lines[i], chat).is_none()
                {
                    item.push(' ');
                    item.push_str(lines[i].trim());
                    i += 1;
                }
                out.push(format!("<li>{}</li>", inline(&item, directory)?));
                let Some((next_tag, text)) = lines.get(i).and_then(|line| list_item(line, chat))
                else {
                    break;
                };
                if next_tag != tag {
                    break;
                }
                item_line = text;
                i += 1;
            }
            out.push(format!("</{tag}>"));
        } else if chat && let Some(first) = line.strip_prefix('>') {
            let mut quote = vec![first.strip_prefix(' ').unwrap_or(first)];
            while let Some(text) = lines.get(i).and_then(|line| line.strip_prefix('>')) {
                quote.push(text.strip_prefix(' ').unwrap_or(text));
                i += 1;
            }
            out.push(format!(
                "<blockquote><p>{}</p></blockquote>",
                inline(&quote.join("\n"), directory)?
            ));
        } else {
            let mut paragraph = vec![line];
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].starts_with('#')
                && !lines[i].starts_with('|')
                && !lines[i].starts_with("```")
                && list_item(lines[i], chat).is_none()
                && !(chat && (lines[i].starts_with('>') || table_line(lines[i], true)))
            {
                paragraph.push(lines[i]);
                i += 1;
            }
            out.push(format!(
                "<p>{}</p>",
                inline(&paragraph.join(if chat { "\n" } else { " " }), directory)?
            ));
        }
    }
    Ok(out.join("\n"))
}

pub fn render_chat(text: &str) -> String {
    render_fragment(text, None).unwrap_or_else(|_| escape(text))
}

pub fn render(text: &str, directory: &Path) -> Result<String, String> {
    let language = if text
        .lines()
        .next()
        .is_some_and(|line| line.contains("사용"))
    {
        "ko"
    } else {
        "en"
    };
    let style = "body{font:14px/1.65 system-ui,sans-serif;max-width:1000px;margin:24px auto;padding:0 20px;overflow-wrap:anywhere}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:8px;text-align:left;vertical-align:top}th{background:#eee}pre{background:#f5f5f5;padding:12px;white-space:pre-wrap}img{max-width:100%;height:auto}.manual-table{overflow:auto}";
    Ok(format!(
        "<!doctype html>\n<html lang=\"{language}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>AgentConductor User Manual</title><style>{style}</style></head><body>\n{}\n</body></html>\n",
        render_fragment(text, Some(directory))?
    ))
}
