//! DD-120: a verification report that records a blocking issue fails integration
//! verification, like a failing test, so the coordinator never accepts a candidate its own
//! verifier rejected.

/// The blocking findings a Markdown report records: the items under a heading that names
/// blocking issues ("Blocking", "Blockers", "차단"; not "Non-blocking"/"비차단"), or else a
/// summary line counting more than zero blocking issues. None when nothing blocks.
pub fn blocking_findings(text: &str) -> Option<String> {
    let heading = |line: &str| {
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        (1..=6)
            .contains(&level)
            .then(|| trimmed[level..].trim().to_lowercase())
    };
    let blocking = |title: &str| {
        let negated = [
            "non-blocking",
            "non blocking",
            "nonblocking",
            "비차단",
            "unblocked",
        ]
        .iter()
        .any(|n| title.contains(n));
        !negated
            && ["blocking", "blocker", "차단"]
                .iter()
                .any(|k| title.contains(k))
    };
    let empty = |item: &str| {
        let item = item
            .trim()
            .trim_matches(['*', '_', '.', ' '])
            .to_lowercase();
        matches!(
            item.as_str(),
            "" | "none" | "n/a" | "없음" | "해당 없음" | "-"
        )
    };
    let mut items = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(title) = heading(line) {
            inside = blocking(&title);
            continue;
        }
        let trimmed = line.trim_start();
        let item = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| {
                let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
                (digits > 0)
                    .then(|| trimmed[digits..].strip_prefix(". "))
                    .flatten()
            });
        if inside && let Some(item) = item.filter(|i| !empty(i)) {
            items.push(item.trim().to_owned());
        }
    }
    if items.is_empty() {
        // "Blocking issues: 1" / "차단 이슈: 1건" without a findings section.
        let counted = text.lines().find(|line| {
            let lower = line.to_lowercase();
            let label = ["blocking issues:", "blockers:", "차단 이슈:"]
                .iter()
                .find_map(|l| lower.find(l).map(|at| at + l.len()));
            label.is_some_and(|at| {
                lower[at..]
                    .trim_start()
                    .trim_start_matches(['*', ' '])
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse::<u32>()
                    .is_ok_and(|n| n > 0)
            }) && !lower.contains("비차단")
        });
        return counted.map(|line| line.trim().to_owned());
    }
    let mut text = items.join("\n");
    if text.len() > 2048 {
        let mut end = 2048;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        text.push('…');
    }
    Some(text)
}
