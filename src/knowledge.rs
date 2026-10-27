use crate::{
    guard::{Disposition, Policy},
    state::Store,
    workflow::digest,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::Path,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub task: String,
    pub context: String,
    pub failure: String,
    pub cause: String,
    pub response: String,
    pub evidence: String,
    pub confidence: f64,
    pub revalidate: String,
    pub sources: Vec<Source>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    id: String,
    accepted_ms: u64,
    lesson: Lesson,
}

fn assets(dir: &Path) -> Result<Vec<Asset>, String> {
    let store = Store::read_only_snapshot(dir.join("state.sqlite"))?;
    let list: Vec<Asset> = serde_json::from_value(
        store
            .optional_document("knowledge")?
            .unwrap_or_else(|| serde_json::json!([])),
    )
    .map_err(|e| e.to_string())?;
    if list.len() > 100
        || list.iter().any(|a| {
            a.accepted_ms == 0
                || !serde_json::to_vec(&a.lesson)
                    .is_ok_and(|v| v.len() <= 16 * 1024 && digest(&v) == a.id)
        })
    {
        return Err("invalid authoritative lesson asset".into());
    }
    Ok(list)
}

struct Fingerprints {
    remaining: u64,
    cache: BTreeMap<String, String>,
}
impl Fingerprints {
    fn new() -> Self {
        Self {
            remaining: 8 * 1024 * 1024,
            cache: BTreeMap::new(),
        }
    }
    fn hash(&mut self, repo: &Path, path: &str) -> Result<String, String> {
        let policy = Policy::new(repo, &["knowledge-source".into()])?;
        if policy.classify("Read", &serde_json::json!({"file_path":path}))?
            != Disposition::Automatic
        {
            return Err("sensitive knowledge sources are unsupported".into());
        }
        if let Some(hash) = self.cache.get(path) {
            return Ok(hash.clone());
        }
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let file = options.open(repo.join(path)).map_err(|e| e.to_string())?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 1024 * 1024 || metadata.len() > self.remaining {
            return Err("knowledge source exceeds regular-file fingerprint budget".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1 {
                return Err("knowledge source alias refused".into());
            }
        }
        let mut bytes = Vec::new();
        file.take(metadata.len() + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 != metadata.len() {
            return Err("knowledge source changed during fingerprinting".into());
        }
        self.remaining -= metadata.len();
        let hash = digest(&bytes);
        self.cache.insert(path.into(), hash.clone());
        Ok(hash)
    }
    fn fresh(&mut self, repo: &Path, lesson: &Lesson) -> bool {
        lesson
            .sources
            .iter()
            .all(|s| self.hash(repo, &s.path).is_ok_and(|h| h == s.sha256))
    }
}

pub(crate) fn accept(
    dir: &Path,
    repo: &Path,
    lesson: &Lesson,
    expected: &str,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(lesson).map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024
        || digest(&bytes) != expected
        || lesson.sources.is_empty()
        || lesson.sources.len() > 8
        || !lesson.confidence.is_finite()
        || !(0.0..=1.0).contains(&lesson.confidence)
        || [
            &lesson.task,
            &lesson.context,
            &lesson.failure,
            &lesson.cause,
            &lesson.response,
            &lesson.evidence,
            &lesson.revalidate,
        ]
        .iter()
        .any(|s| s.trim().is_empty() || s.len() > 4096)
    {
        return Err(
            "lesson requires exact digest, bounded typed fields, confidence and sources".into(),
        );
    }
    if !Fingerprints::new().fresh(repo, lesson) {
        return Err("lesson sources are stale, unsupported or inaccessible".into());
    }
    let mut list = assets(dir)?;
    if !list.iter().any(|a| a.id == expected) {
        if list.len() >= 100 {
            return Err("reviewed lesson limit is 100 per run".into());
        }
        list.push(Asset {
            id: expected.into(),
            accepted_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| "knowledge timestamp overflow")?,
            lesson: lesson.clone(),
        });
        Store::open(dir.join("state.sqlite"))?.record(
            "knowledge",
            &serde_json::to_value(&list).map_err(|e| e.to_string())?,
            &format!(
                "Reviewed lesson {expected} for task {}: {}. Response: {}. Revalidate: {}.",
                lesson.task, lesson.context, lesson.response, lesson.revalidate
            ),
        )?;
    }
    materialize(dir, &list)
        .map_err(|e| format!("Reviewed lesson saved in SQLite; wiki materialization failed: {e}"))
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("pending");
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(&temporary).map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    fs::rename(&temporary, path).map_err(|e| e.to_string())
}
fn materialize(dir: &Path, list: &[Asset]) -> Result<(), String> {
    for path in [dir.join("knowledge"), dir.join("knowledge/wiki")] {
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Err("knowledge mirror directory alias refused".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&path).map_err(|e| e.to_string())?
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let wiki = dir.join("knowledge/wiki");
    let mut index = String::from(
        "# Reviewed runtime lessons\n\nStatus must be revalidated against source hashes before reuse.\n\n",
    );
    let log_path = wiki.join("log.md");
    safe_mirror(&log_path)?;
    let mut log = if log_path.exists() {
        fs::read_to_string(&log_path).map_err(|e| e.to_string())?
    } else {
        String::from("# Runtime knowledge log\n\n")
    };
    for asset in list {
        let l = &asset.lesson;
        let path = wiki.join(format!("{}.md", asset.id));
        safe_mirror(&path)?;
        let page = format!(
            "# Reviewed lesson {}\n\nAccepted at Unix UTC milliseconds: {}\nTask: {}\nConfidence: {}\n\n## Context\n\n{}\n\n## Failure\n\n{}\n\n## Cause\n\n{}\n\n## Response\n\n{}\n\n## Evidence\n\n{}\n\n## Revalidation trigger\n\n{}\n\n## Sources\n\n{}\n",
            asset.id,
            asset.accepted_ms,
            l.task,
            l.confidence,
            l.context,
            l.failure,
            l.cause,
            l.response,
            l.evidence,
            l.revalidate,
            l.sources
                .iter()
                .map(|s| format!("- `{}`: `{}`", s.path, s.sha256))
                .collect::<Vec<_>>()
                .join("\n")
        );
        atomic(&path, page.as_bytes())?;
        index.push_str(&format!("- [{}]({}.md)\n", asset.id, asset.id));
        if !log.contains(&format!("Lesson {} accepted", asset.id)) {
            log.push_str(&format!(
                "- Lesson {} accepted for task {} after exact digest/source review.\n",
                asset.id, l.task
            ));
        }
    }
    safe_mirror(&wiki.join("index.md"))?;
    atomic(&wiki.join("index.md"), index.as_bytes())?;
    atomic(&log_path, log.as_bytes())
}

fn safe_mirror(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.len() > 64 * 1024 {
                return Err("invalid knowledge mirror file".into());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.nlink() != 1 {
                    return Err("knowledge mirror alias refused".into());
                }
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn status(dir: &Path, repo: &Path) -> Result<Vec<serde_json::Value>, String> {
    let mut hashes = Fingerprints::new();
    Ok(assets(dir)?.into_iter().map(|a|serde_json::json!({"id":a.id,"task":a.lesson.task,"status":if hashes.fresh(repo,&a.lesson){"fresh"}else{"stale"},"lesson":a.lesson})).collect())
}
pub fn context(dir: &Path, repo: &Path, task: &str) -> Result<String, String> {
    let mut hashes = Fingerprints::new();
    let mut text = String::new();
    for asset in assets(dir)? {
        if asset.lesson.task != task || !hashes.fresh(repo, &asset.lesson) {
            continue;
        }
        let note = serde_json::to_string(&asset).map_err(|e| e.to_string())?;
        if text.len() + note.len() + 1 > 8 * 1024 {
            continue;
        }
        text.push_str(&note);
        text.push('\n');
    }
    Ok(text)
}
