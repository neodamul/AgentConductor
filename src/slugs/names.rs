//! Validation and bounded reference names for Goal folders.
use crate::mcp::Reference;

pub(super) const MAX: usize = 48;
const MAX_WITH_REFS: usize = 96;
// Leave room for a translated or date-derived title and for a collision counter.
const MAX_SUFFIX: usize = MAX_WITH_REFS - 24;
// Keep previously registered development folders usable.
const LEGACY_WITH_REFS: usize = 220;
const REF_SEPARATOR: &str = "__refs_";

pub fn is_placeholder(slug: &str) -> bool {
    let (base, _) = split_references(slug);
    let Some(rest) = base.strip_prefix("goal_") else {
        return false;
    };
    let Some((date, suffix)) = rest.split_at_checked(8) else {
        return false;
    };
    date.bytes().all(|b| b.is_ascii_digit())
        && (suffix.is_empty()
            || suffix
                .strip_prefix('_')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())))
}

pub fn unique(base: &str, existing: &[String]) -> String {
    if !existing.iter().any(|s| s == base) {
        return base.into();
    }
    let (name, refs) = split_references(base);
    let limit = if refs.is_empty() {
        MAX
    } else {
        reference_name_limit(base).saturating_sub(refs.len())
    };
    (2..)
        .map(|n| {
            let suffix = format!("_{n}");
            format!(
                "{}{suffix}{refs}",
                &name[..name.len().min(limit.saturating_sub(suffix.len()))]
            )
        })
        .find(|candidate| !existing.contains(candidate))
        .expect("an unused suffix exists")
}

fn split_references(slug: &str) -> (&str, &str) {
    slug.find(REF_SEPARATOR)
        .map_or((slug, ""), |index| slug.split_at(index))
}

pub(crate) fn with_references(base: &str, refs: &[Reference]) -> String {
    if refs.is_empty() {
        return base.into();
    }
    let all: Vec<_> = refs.iter().collect();
    let full = reference_suffix(&all);
    if full.len() <= MAX_SUFFIX {
        return with_suffix(base, &full, MAX_WITH_REFS);
    }

    let keys = refs
        .iter()
        .map(Reference::key)
        .collect::<Vec<_>>()
        .join("\n");
    let digest = crate::workflow::digest(keys.as_bytes());
    let overflow = format!("more_{}", &digest[..10]);
    let mut kept = Vec::new();
    for entry in all {
        kept.push(entry);
        if reference_suffix(&kept).len() + 1 + overflow.len() > MAX_SUFFIX {
            kept.pop();
        }
    }
    let suffix = if kept.is_empty() {
        format!("{REF_SEPARATOR}{overflow}")
    } else {
        format!("{}_{overflow}", reference_suffix(&kept))
    };
    with_suffix(base, &suffix, MAX_WITH_REFS)
}

fn reference_suffix(refs: &[&Reference]) -> String {
    let mut parts = Vec::new();
    for kind in ["jira", "confluence"] {
        let mut ids = refs.iter().filter(|r| r.kind == kind).peekable();
        if ids.peek().is_some() {
            parts.push(kind.to_string());
            parts.extend(ids.map(|r| r.id.to_ascii_lowercase().replace('-', "_")));
        }
    }
    format!("{REF_SEPARATOR}{}", parts.join("_"))
}

fn with_suffix(base: &str, suffix: &str, limit: usize) -> String {
    let end = base.len().min(limit.saturating_sub(suffix.len()));
    format!("{}{suffix}", &base[..end])
}

pub(crate) fn renamed(old: &str, name: &str) -> String {
    let (_, suffix) = split_references(old);
    with_suffix(name, suffix, reference_name_limit(old))
}

fn reference_name_limit(slug: &str) -> usize {
    if slug.len() > MAX_WITH_REFS {
        LEGACY_WITH_REFS
    } else {
        MAX_WITH_REFS
    }
}

pub(crate) fn validate_base(slug: &str) -> Result<(), String> {
    if slug.len() <= MAX && !slug.contains(REF_SEPARATOR) {
        return validate(slug);
    }
    Err(crate::ui::messages::text("native.goal_slug_invalid_base").into())
}

pub fn validate(slug: &str) -> Result<(), String> {
    let (base, refs) = split_references(slug);
    let ok = !slug.is_empty()
        && (slug.len() <= MAX
            || (!base.is_empty()
                && refs.len() > REF_SEPARATOR.len()
                && slug.len() <= LEGACY_WITH_REFS))
        && slug.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    ok.then_some(())
        .ok_or_else(|| crate::ui::messages::text("native.goal_slug_invalid").into())
}
