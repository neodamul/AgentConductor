//! Strip JSONC comments/trailing commas while preserving quoted strings.
pub(super) fn parse(text: &str) -> Result<serde_json::Value, String> {
    if text.len() > 48 * 1024 {
        return Err("MCP settings exceed 48 KiB".into());
    }
    let b = text.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let (mut i, mut string, mut escape) = (0, false, false);
    while i < b.len() {
        let c = b[i];
        if string {
            out.push(c);
            if escape {
                escape = false;
            } else if c == b'\\' {
                escape = true;
            } else if c == b'"' {
                string = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            string = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            out.push(b' ');
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < b.len() && &b[i..i + 2] != b"*/" {
                i += 1;
            }
            if i + 1 >= b.len() {
                return Err("unterminated JSONC comment".into());
            }
            i += 2;
            out.push(b' ');
            continue;
        }
        out.push(c);
        i += 1;
    }
    let b = out;
    let mut out = Vec::with_capacity(b.len());
    let (mut string, mut escape) = (false, false);
    for (i, &c) in b.iter().enumerate() {
        if !string && c == b',' {
            let next = b[i + 1..].iter().find(|v| !v.is_ascii_whitespace());
            if matches!(next, Some(b'}' | b']')) {
                continue;
            }
        }
        out.push(c);
        if string {
            if escape {
                escape = false;
            } else if c == b'\\' {
                escape = true;
            } else if c == b'"' {
                string = false;
            }
        } else if c == b'"' {
            string = true;
        }
    }
    serde_json::from_slice(&out).map_err(|_| "invalid MCP JSON/JSONC settings".into())
}
