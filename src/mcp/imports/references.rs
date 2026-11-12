//! Issue references found in a goal text.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    pub kind: String,
    pub id: String,
}
impl Reference {
    pub fn key(&self) -> String {
        format!("{}:{}", self.kind, self.id)
    }
}
pub fn references(goal: &str) -> Result<Vec<Reference>, String> {
    let mut refs = vec![];
    for token in goal.split(|c: char| !c.is_ascii_alphanumeric() && ![':', '-', '_'].contains(&c)) {
        let Some((kind, id)) = token.split_once(':') else {
            continue;
        };
        let kind = kind.to_ascii_lowercase();
        if !["jira", "confluence"].contains(&kind.as_str()) {
            continue;
        }
        let valid = if kind == "confluence" {
            !id.is_empty() && id.len() <= 32 && id.bytes().all(|c| c.is_ascii_digit())
        } else {
            id.rsplit_once('-').is_some_and(|(p, n)| {
                !p.is_empty()
                    && p.as_bytes()[0].is_ascii_uppercase()
                    && p.len() <= 32
                    && p.bytes()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
                    && !n.is_empty()
                    && n.bytes().all(|c| c.is_ascii_digit())
            })
        };
        if !valid {
            return Err("invalid jira:KEY-123 or confluence:PAGE_ID reference".into());
        }
        let r = Reference {
            kind,
            id: id.into(),
        };
        if !refs.contains(&r) {
            refs.push(r);
        }
    }
    if refs.len() > 12 {
        return Err("at most twelve goal references".into());
    }
    Ok(refs)
}
