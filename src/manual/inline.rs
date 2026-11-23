use std::path::Path;

pub(super) fn escape(value: &str) -> String {
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

pub(super) fn inline(value: &str, directory: Option<&Path>) -> Result<String, String> {
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
        } else if directory.is_none()
            && let Some(emphasis) = rest.strip_prefix('*')
            && let Some(end) = emphasis.find('*').filter(|end| *end > 0)
        {
            out.push_str("<em>");
            out.push_str(&escape(&emphasis[..end]));
            out.push_str("</em>");
            rest = &emphasis[end + 1..];
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
                if image && let Some(directory) = directory {
                    let root = directory.canonicalize().map_err(|e| e.to_string())?;
                    let assets = root
                        .join("assets")
                        .canonicalize()
                        .map_err(|e| e.to_string())?;
                    let path = root.join(url).canonicalize().map_err(|e| e.to_string())?;
                    if !path.starts_with(&assets)
                        || !path
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|s| s.eq_ignore_ascii_case("png"))
                    {
                        return Err(format!("manual image must be a local PNG asset: {url}"));
                    }
                    let data = std::fs::read(path).map_err(|e| e.to_string())?;
                    out.push_str("<img alt=\"");
                    out.push_str(&quoted(label));
                    out.push_str("\" src=\"data:image/png;base64,");
                    out.push_str(&base64(&data));
                    out.push_str("\">");
                } else if !image
                    && if directory.is_some() {
                        safe_link(url)
                    } else {
                        chat_link(url)
                    }
                {
                    let target = if directory.is_some()
                        && matches!(url, "user-manual.md" | "user-manual.ko.md")
                    {
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

pub(crate) fn chat_link(url: &str) -> bool {
    url.len() <= 8192
        && !url
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace() || ch == '\\')
        && url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .and_then(|rest| rest.split(['/', '?', '#']).next())
            .is_some_and(|host| !host.is_empty())
}

fn safe_link(url: &str) -> bool {
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
