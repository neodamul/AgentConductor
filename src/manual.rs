use std::path::Path;

mod images;

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn quoted(value: &str) -> String {
    escape(value)
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: usize,
        text: String,
    },
    Paragraph(String),
    List(Vec<String>),
    Code(String),
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

pub fn blocks(text: &str) -> Vec<Block> {
    let lines: Vec<_> = text.lines().collect();
    let mut result = Vec::new();
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
            result.push(Block::Code(code.join("\n")));
        } else if let Some((prefix, heading)) = line.split_once(' ')
            && !prefix.is_empty()
            && prefix.len() <= 6
            && prefix.chars().all(|c| c == '#')
        {
            result.push(Block::Heading {
                level: prefix.len(),
                text: heading.to_owned(),
            });
        } else if line.starts_with('|')
            && i < lines.len()
            && cells(lines[i]).iter().all(|c| {
                let separator = c.trim_matches(':');
                !separator.is_empty() && separator.chars().all(|x| x == '-')
            })
        {
            let headers = cells(line);
            i += 1;
            let mut rows = Vec::new();
            while i < lines.len() && lines[i].starts_with('|') {
                rows.push(cells(lines[i]));
                i += 1;
            }
            result.push(Block::Table { headers, rows });
        } else if line.trim_start().starts_with("- ") {
            let mut items = Vec::new();
            let mut item_line = line;
            loop {
                let mut item = item_line.trim_start().trim_start_matches("- ").to_owned();
                while i < lines.len()
                    && lines[i].starts_with("  ")
                    && !lines[i].trim_start().starts_with("- ")
                {
                    item.push(' ');
                    item.push_str(lines[i].trim());
                    i += 1;
                }
                items.push(item);
                if i >= lines.len() || !lines[i].trim_start().starts_with("- ") {
                    break;
                }
                item_line = lines[i];
                i += 1;
            }
            result.push(Block::List(items));
        } else {
            let mut paragraph = vec![line];
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].starts_with('#')
                && !lines[i].starts_with('|')
                && !lines[i].starts_with("```")
                && !lines[i].trim_start().starts_with("- ")
            {
                paragraph.push(lines[i]);
                i += 1;
            }
            result.push(Block::Paragraph(paragraph.join(" ")));
        }
    }
    result
}

fn inline(value: &str, directory: &Path) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = value;
    while !rest.is_empty() {
        if let Some(code) = rest.strip_prefix('`')
            && let Some(end) = code.find('`')
        {
            out.push_str("<code>");
            out.push_str(&escape(&code[..end]));
            out.push_str("</code>");
            rest = &code[end + 1..];
        } else if let Some(strong) = rest.strip_prefix("**")
            && let Some(end) = strong.find("**")
        {
            out.push_str("<strong>");
            out.push_str(&escape(&strong[..end]));
            out.push_str("</strong>");
            rest = &strong[end + 2..];
        } else {
            let image = rest.starts_with("![");
            let bracket = if image {
                2
            } else if rest.starts_with('[') {
                1
            } else {
                0
            };
            if bracket > 0
                && let Some(label_end) = rest[bracket..].find("](")
                && let Some(url_end) = rest[bracket + label_end + 2..].find(')')
            {
                let label = &rest[bracket..bracket + label_end];
                let url = &rest[bracket + label_end + 2..bracket + label_end + 2 + url_end];
                if image {
                    out.push_str(&images::embed(label, url, directory)?);
                } else if safe_link(url) {
                    let target = if url == "user-manual.md" || url == "user-manual.ko.md" {
                        format!("{}.html", url.trim_end_matches(".md"))
                    } else {
                        url.to_owned()
                    };
                    out.push_str("<a target=\"_blank\" rel=\"noopener\" href=\"");
                    out.push_str(&quoted(&target));
                    out.push_str("\">");
                    out.push_str(&escape(label));
                    out.push_str("</a>");
                } else {
                    out.push_str(&escape(label));
                }
                rest = &rest[bracket + label_end + 2 + url_end + 1..];
            } else {
                let ch = rest.chars().next().expect("nonempty");
                out.push_str(&escape(&ch.to_string()));
                rest = &rest[ch.len_utf8()..];
            }
        }
    }
    Ok(out)
}

pub(crate) fn safe_link(url: &str) -> bool {
    !url.starts_with("//")
        && url
            .split_once(':')
            .map(|(scheme, _)| matches!(scheme, "https" | "http" | "mailto"))
            .unwrap_or(true)
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let bits = ((chunk[0] as u32) << 16)
            | ((*chunk.get(1).unwrap_or(&0) as u32) << 8)
            | (*chunk.get(2).unwrap_or(&0) as u32);
        for index in 0..4 {
            let byte = match (chunk.len(), index) {
                (1, 2..) | (2, 3) => b'=',
                _ => TABLE[((bits >> (18 - index * 6)) & 63) as usize],
            };
            out.push(byte as char);
        }
    }
    out
}

pub fn render(text: &str, directory: &Path) -> Result<String, String> {
    let mut out = Vec::new();
    for block in blocks(text) {
        match block {
            Block::Heading { level, text } => out.push(format!(
                "<h{level}>{}</h{level}>",
                inline(&text, directory)?
            )),
            Block::Paragraph(text) => out.push(format!("<p>{}</p>", inline(&text, directory)?)),
            Block::List(items) => {
                out.push("<ul>".into());
                for item in items {
                    out.push(format!("<li>{}</li>", inline(&item, directory)?));
                }
                out.push("</ul>".into());
            }
            Block::Code(code) => {
                out.push(format!("<pre><code>{}</code></pre>", escape(&code)));
            }
            Block::Table { headers, rows } => {
                out.push(format!(
                        "<div class=\"manual-table\"><table><thead><tr>{}</tr></thead><tbody>",
                        headers
                            .iter()
                            .map(|cell| inline(cell, directory)
                                .map(|value| format!("<th>{value}</th>")))
                            .collect::<Result::<Vec<_>, _>>()?
                            .join("")
                    ));
                for row in rows {
                    out.push(format!(
                        "<tr>{}</tr>",
                        row.iter()
                            .map(|cell| inline(cell, directory)
                                .map(|value| format!("<td>{value}</td>")))
                            .collect::<Result::<Vec<_>, _>>()?
                            .join("")
                    ));
                }
                out.push("</tbody></table></div>".into());
            }
        }
    }
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
        out.join("\n")
    ))
}
