//! GitHub #323: why an integration test failed, from both of its streams.
const LINES: usize = 8;

/// The last `LINES` lines of a stream, saying how many earlier lines were left out.
fn last_lines(text: &str) -> String {
    let all: Vec<&str> = text.trim_end().lines().collect();
    if all.len() <= LINES {
        return all.join("\n");
    }
    let skipped = all.len() - LINES;
    format!("… {skipped} earlier lines\n{}", all[skipped..].join("\n"))
}

/// Each stream gets its own budget so a warning on one cannot push the failure of the other out.
/// A single non-empty stream is shown as it is; with both, they are labeled and stdout comes last.
pub fn failure_output(stderr: &str, stdout: &str) -> String {
    let streams: Vec<(&str, String)> = [("stderr", stderr), ("stdout", stdout)]
        .into_iter()
        .filter(|(_, text)| !text.trim().is_empty())
        .map(|(name, text)| (name, last_lines(text)))
        .collect();
    match streams.as_slice() {
        [(_, only)] => only.clone(),
        _ => streams
            .iter()
            .map(|(name, text)| format!("{name}:\n{text}"))
            .collect::<Vec<_>>()
            .join("\n\n"),
    }
}
