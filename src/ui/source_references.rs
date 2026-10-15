use std::collections::HashSet;

pub(crate) fn suggestions(goal: &str) -> Vec<String> {
    let explicit: HashSet<_> = explicit_references(goal).into_iter().collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for reference in issue_references(goal)
        .into_iter()
        .chain(page_references(goal))
    {
        if !explicit.contains(&reference) && seen.insert(reference.clone()) {
            result.push(reference);
            if result.len() == 12 {
                break;
            }
        }
    }
    result
}

pub(crate) fn references(goal: &str) -> Vec<String> {
    let mut references = explicit_references(goal);
    let mut seen = HashSet::new();
    references.retain(|reference| seen.insert(reference.clone()));
    references
}

fn explicit_references(goal: &str) -> Vec<String> {
    let mut references = Vec::new();
    for (prefix, kind) in [("jira:", "jira"), ("confluence:", "confluence")] {
        for (index, _) in goal.match_indices(prefix) {
            let start = index + prefix.len();
            let end = goal[start..]
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .map_or(goal.len(), |offset| start + offset);
            let id = &goal[start..end];
            if (kind == "jira" && valid_jira(id))
                || (kind == "confluence"
                    && !id.is_empty()
                    && id.bytes().all(|c| c.is_ascii_digit()))
            {
                references.push((index, format!("{kind}:{id}")));
            }
        }
    }
    references.sort_by_key(|(index, _)| *index);
    references.into_iter().map(|(_, value)| value).collect()
}

fn issue_references(goal: &str) -> Vec<String> {
    let bytes = goal.as_bytes();
    let mut values = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_uppercase() && (index == 0 || !word(bytes[index - 1])) {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_uppercase()
                    || bytes[index].is_ascii_digit()
                    || bytes[index] == b'_')
            {
                index += 1;
            }
            if index < bytes.len() && bytes[index] == b'-' {
                let dash = index;
                index += 1;
                let digits = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                if digits < index && (index == bytes.len() || !word(bytes[index])) {
                    values.push(format!("jira:{}", &goal[start..index]));
                    continue;
                }
                index = dash + 1;
            }
            if index == start {
                index += 1;
            }
        } else {
            index += 1;
        }
    }
    values
}

fn page_references(goal: &str) -> Vec<String> {
    let lower = goal.to_ascii_lowercase();
    let mut values = Vec::new();
    let mut index = 0;
    while index < goal.len() {
        let rest = &lower[index..];
        let keyword_end = if rest.starts_with("참고") {
            Some(index + "참고".len())
        } else if index == 0 || !word(goal.as_bytes()[index - 1]) {
            ["references", "reference", "refs", "ref"]
                .iter()
                .find(|label| {
                    rest.starts_with(**label)
                        && rest
                            .as_bytes()
                            .get(label.len())
                            .is_none_or(|next| !word(*next))
                })
                .map(|label| index + label.len())
        } else {
            None
        };
        if let Some(end) = keyword_end
            && let Some(ids) = parenthesized_pages(goal, end)
        {
            values.extend(ids.into_iter().map(|id| format!("confluence:{id}")));
            index = end;
            continue;
        }
        index += goal[index..].chars().next().unwrap().len_utf8();
    }
    values
}

fn parenthesized_pages(goal: &str, mut index: usize) -> Option<Vec<&str>> {
    let bytes = goal.as_bytes();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    if bytes.get(index) == Some(&b':') {
        index += 1;
    } else if goal[index..].starts_with('：') {
        index += '：'.len_utf8();
    }
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    if bytes.get(index) != Some(&b'(') {
        return None;
    }
    let start = index + 1;
    let close = goal[start..].find(')')? + start;
    let body = &goal[start..close];
    if !body
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c.is_whitespace())
    {
        return None;
    }
    let ids: Vec<_> = body
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|id| !id.is_empty())
        .collect();
    (!ids.is_empty() && ids.iter().all(|id| valid_page(id))).then_some(ids)
}

fn valid_jira(id: &str) -> bool {
    let Some((project, number)) = id.split_once('-') else {
        return false;
    };
    !project.is_empty()
        && project.as_bytes()[0].is_ascii_uppercase()
        && project
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
        && !number.is_empty()
        && number.bytes().all(|c| c.is_ascii_digit())
}

fn valid_page(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|c| c.is_ascii_digit())
}

fn word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
