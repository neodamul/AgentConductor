//! GitHub #349: the architecture documents stay connected. Every identifier (R, BG, SF, UC, FR,
//! NFR, C, QAS, DDR, DD) is defined once in its own document, every cited identifier is defined,
//! every requirement appears once in the README matrix, a `Status:` line never points to a
//! superseded entry, and every issue-driven unit of the implementation plan names what it traces.
//! The rules are stated in docs/README.md ("Identifier and consolidation rules").
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

type Id = (&'static str, u32);

const KINDS: [(&str, &str); 10] = [
    ("NFR", "04b-functional-nonfunctional-analysis.md"),
    ("QAS", "05-qas.md"),
    ("DDR", "05b-design-drivers.md"),
    ("DD", "06-design-decisions.md"),
    ("BG", "02-business-goals.md"),
    ("SF", "03-system-features.md"),
    ("UC", "04-usecases.md"),
    ("FR", "04b-functional-nonfunctional-analysis.md"),
    ("R", "01-requirements.md"),
    ("C", "04b-functional-nonfunctional-analysis.md"),
];

fn docs() -> BTreeMap<String, String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".md") && !name.starts_with("user-manual"))
        .map(|name| (name.clone(), fs::read_to_string(dir.join(&name)).unwrap()))
        .collect()
}

fn home(kind: &str) -> &'static str {
    KINDS.iter().find(|(k, _)| *k == kind).unwrap().1
}

fn digits(text: &str) -> (Option<u32>, usize) {
    let len = text.bytes().take_while(u8::is_ascii_digit).count();
    (text[..len].parse().ok(), len)
}

/// The identifier that starts exactly at `text`, and its length.
fn id_at(text: &str) -> Option<(Id, usize)> {
    KINDS.iter().find_map(|(kind, _)| {
        let rest = text.strip_prefix(kind)?.strip_prefix('-')?;
        match digits(rest) {
            (Some(n), len) if !rest[len..].starts_with(|c: char| c.is_alphanumeric()) => {
                Some(((*kind, n), kind.len() + 1 + len))
            }
            _ => None,
        }
    })
}

/// Every identifier cited in `line`, with ranges (`UC-44–48`, `R-55–R-59`) and slash lists
/// (`SF-04/06/07`) expanded.
fn cited(line: &str) -> Vec<Id> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let before = line[..i].chars().next_back();
        let boundary = !before.is_some_and(|c| c.is_alphanumeric() || c == '-');
        let Some(((kind, first), len)) = boundary.then(|| id_at(&line[i..])).flatten() else {
            i += line[i..].chars().next().unwrap().len_utf8();
            continue;
        };
        out.push((kind, first));
        i += len;
        let rest = &line[i..];
        if let Some(range) = rest.strip_prefix('–').or_else(|| rest.strip_prefix('-')) {
            let range = range
                .strip_prefix(kind)
                .and_then(|r| r.strip_prefix('-'))
                .unwrap_or(range);
            if let (Some(last), len) = digits(range)
                && last > first
                && last - first < 60
                && !range[len..]
                    .starts_with(['-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
            {
                out.extend((first + 1..=last).map(|n| (kind, n)));
                i = line.len() - range.len() + len;
            }
        }
        while let Some(more) = line[i..].strip_prefix('/') {
            match digits(more) {
                (Some(n), len) => {
                    out.push((kind, n));
                    i += 1 + len;
                }
                _ => break,
            }
        }
    }
    out
}

/// Identifiers a heading defines: the ones it starts with (`### FR-46 / NFR-21 …`).
fn heading_ids(line: &str) -> Vec<Id> {
    let mut rest = line.trim_start_matches('#').trim_start();
    rest = rest.strip_prefix("Driver ").unwrap_or(rest);
    let mut ids = Vec::new();
    while let Some((id, len)) = id_at(rest) {
        ids.push(id);
        let after = rest[len..].trim_start();
        match after.strip_prefix('/') {
            Some(next) => rest = next.trim_start(),
            None => break,
        }
    }
    ids
}

/// Identifiers a paragraph or list item defines (`FR-47: …`, `- BG-07 (R-18): …`).
fn paragraph_ids(line: &str) -> Vec<Id> {
    let starts = std::iter::once(line.strip_prefix("- ").unwrap_or(line))
        .chain(line.match_indices(". ").map(|(at, _)| &line[at + 2..]));
    starts
        .filter_map(|text| {
            let (id, len) = id_at(text)?;
            let rest = &text[len..];
            let rest = match rest.strip_prefix(" (") {
                Some(inner) => &inner[inner.find(')')? + 1..],
                None => rest,
            };
            rest.starts_with(": ").then_some(id)
        })
        .collect()
}

struct Definitions {
    /// Headings and table rows: (document, line) per identifier.
    strong: BTreeMap<Id, BTreeSet<(String, usize)>>,
    /// Paragraphs outside the section their identifier heads: (document, line, is a list item).
    weak: BTreeMap<Id, Vec<(String, usize, bool)>>,
}

fn definitions(docs: &BTreeMap<String, String>) -> Definitions {
    let mut defs = Definitions {
        strong: BTreeMap::new(),
        weak: BTreeMap::new(),
    };
    let owned: BTreeSet<&str> = KINDS.iter().map(|(_, doc)| *doc).collect();
    for doc in owned {
        let mut section = Vec::new();
        for (n, line) in docs[doc].lines().enumerate() {
            let mine = |id: &Id| home(id.0) == doc;
            if line.starts_with('#') {
                section = heading_ids(line);
                for id in section.iter().filter(|id| mine(id)) {
                    defs.strong
                        .entry(*id)
                        .or_default()
                        .insert((doc.into(), n + 1));
                }
                continue;
            }
            if let Some(row) = line.strip_prefix("| ")
                && let Some((id, _)) = id_at(row).filter(|(id, _)| mine(id))
            {
                defs.strong
                    .entry(id)
                    .or_default()
                    .insert((doc.into(), n + 1));
            }
            for id in paragraph_ids(line) {
                if mine(&id) && !section.contains(&id) {
                    let item = line.starts_with("- ");
                    defs.weak
                        .entry(id)
                        .or_default()
                        .push((doc.into(), n + 1, item));
                }
            }
        }
    }
    defs
}

/// Lines of a document, without the README block that lists retired numbers.
fn cited_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut reserved = false;
    text.lines().enumerate().filter(move |(_, line)| {
        if line.starts_with("<!-- reserved identifiers") {
            reserved = true;
        } else if line.starts_with("<!-- end reserved identifiers") {
            reserved = false;
        }
        !reserved
    })
}

#[test]
fn identifiers_are_parsed_as_the_documents_write_them() {
    assert_eq!(
        cited("R-55–R-59 → UC-44–48, SF-04/06/07; FR-143/NFR-53 on 2026-10-14"),
        [
            ("R", 55),
            ("R", 56),
            ("R", 57),
            ("R", 58),
            ("R", 59),
            ("UC", 44),
            ("UC", 45),
            ("UC", 46),
            ("UC", 47),
            ("UC", 48),
            ("SF", 4),
            ("SF", 6),
            ("SF", 7),
            ("FR", 143),
            ("NFR", 53),
        ]
    );
    assert!(cited("DD-UI-01, QAS-12a, DDR-07").ends_with(&[("DDR", 7)]));
    assert_eq!(heading_ids("### Driver DDR-19 Custom CLI"), [("DDR", 19)]);
    assert_eq!(
        heading_ids("### FR-46 / NFR-21 — Render"),
        [("FR", 46), ("NFR", 21)]
    );
    assert_eq!(
        heading_ids("## DD-79 Messenger (decides part of DD-51)"),
        [("DD", 79)]
    );
    assert_eq!(
        paragraph_ids("- BG-07 (R-18): Continue. FR-48: resolve"),
        [("BG", 7), ("FR", 48)]
    );
}

#[test]
fn every_identifier_is_defined_once_in_its_own_document() {
    let defs = definitions(&docs());
    let twice: Vec<_> = defs.strong.iter().filter(|(_, at)| at.len() > 1).collect();
    assert!(twice.is_empty(), "defined more than once: {twice:?}");
    // A paragraph that defines an identifier headed elsewhere gives it a second meaning;
    // list items under a table only elaborate their row.
    let stray: Vec<_> = defs
        .weak
        .iter()
        .filter(|(id, at)| defs.strong.contains_key(id) && at.iter().any(|(_, _, item)| !item))
        .collect();
    assert!(
        stray.is_empty(),
        "paragraph redefines a headed identifier: {stray:?}"
    );
}

#[test]
fn every_cited_identifier_is_defined() {
    let docs = docs();
    let defs = definitions(&docs);
    let mut missing = BTreeMap::<Id, Vec<String>>::new();
    for (doc, text) in &docs {
        for (n, line) in cited_lines(text) {
            for id in cited(line) {
                if !defs.strong.contains_key(&id) && !defs.weak.contains_key(&id) {
                    missing
                        .entry(id)
                        .or_default()
                        .push(format!("{doc}:{}", n + 1));
                }
            }
        }
    }
    assert!(missing.is_empty(), "cited but never defined: {missing:?}");
}

#[test]
fn every_requirement_is_in_the_readme_matrix_once() {
    let docs = docs();
    let defs = definitions(&docs);
    let rows: Vec<u32> = docs["README.md"]
        .lines()
        .filter_map(|line| id_at(line.strip_prefix("| ")?))
        .filter_map(|((kind, n), _)| (kind == "R").then_some(n))
        .collect();
    let defined: BTreeSet<u32> = defs
        .strong
        .keys()
        .chain(defs.weak.keys())
        .filter_map(|(kind, n)| (*kind == "R").then_some(*n))
        .collect();
    let listed: BTreeSet<u32> = rows.iter().copied().collect();
    assert_eq!(
        rows.len(),
        listed.len(),
        "a requirement has two matrix rows"
    );
    assert_eq!(
        listed, defined,
        "README matrix rows differ from the defined requirements"
    );
}

#[test]
fn a_status_never_points_to_a_superseded_entry() {
    let docs = docs();
    let mut superseded = BTreeSet::new();
    let mut targets = Vec::new();
    for doc in ["01-requirements.md", "04-usecases.md", "05-qas.md"] {
        let mut section = Vec::new();
        for line in docs[doc].lines() {
            if line.starts_with('#') {
                section = heading_ids(line);
            } else if let Some(status) = line.strip_prefix("Status") {
                if status.contains(": Superseded by") || status.contains(": Merged into") {
                    superseded.extend(section.iter().copied());
                }
                let named = status.split(':').nth(1).unwrap_or_default();
                targets.extend(cited(named).into_iter().map(|id| (id, line.to_owned())));
            }
        }
    }
    let dead: Vec<_> = targets
        .iter()
        .filter(|(id, _)| superseded.contains(id))
        .collect();
    assert!(
        dead.is_empty(),
        "status points to a superseded entry: {dead:?}"
    );
}

#[test]
fn every_issue_unit_of_the_plan_traces_to_the_architecture() {
    let plan = &docs()["07-implementation-plan.md"];
    let mut untraced = Vec::new();
    let mut current: Option<(&str, bool)> = None;
    for line in plan.lines().chain(std::iter::once("## end")) {
        if line.starts_with("## ") || line.starts_with("### ") {
            if let Some((title, false)) = current {
                untraced.push(title);
            }
            current = line
                .contains("GitHub #")
                .then_some((line, !cited(line).is_empty()));
        } else if let Some((_, traced)) = current.as_mut() {
            *traced |= !cited(line).is_empty();
        }
    }
    assert!(
        untraced.is_empty(),
        "plan units without a trace: {untraced:?}"
    );
}
