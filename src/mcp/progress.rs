//! GitHub #903: per-document progress of a document preview, rewritten in place for the window to poll.
use super::Reference;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
const LOG_LIMIT: usize = 24;
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Progress {
    pub started_ms: u64,
    pub documents: Vec<Document>,
    #[serde(skip)]
    path: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub reference: String,
    /// waiting, selection, reading, summary_wait, summary, done, failed
    pub stage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub log: Vec<Entry>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub at_ms: u64,
    pub event: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub detail: String,
}
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}
impl Progress {
    /// Progress kept in memory only (library callers and tests).
    pub fn off() -> Self {
        Self::default()
    }
    /// Progress written to `path` after every change; write failures never stop the lookup.
    pub fn file(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            ..Self::default()
        }
    }
    pub(super) fn begin(&mut self, refs: &[Reference]) {
        self.started_ms = now();
        self.documents = refs
            .iter()
            .map(|r| Document {
                reference: r.key(),
                stage: "waiting".into(),
                title: None,
                started_ms: None,
                ended_ms: None,
                error: None,
                log: vec![],
            })
            .collect();
        self.write();
    }
    fn doc(&mut self, r: &Reference) -> Option<&mut Document> {
        let key = r.key();
        self.documents.iter_mut().find(|d| d.reference == key)
    }
    pub(super) fn stage(&mut self, r: &Reference, stage: &str) {
        if let Some(d) = self.doc(r) {
            d.stage = stage.into();
            d.started_ms.get_or_insert_with(now);
            if matches!(stage, "done" | "failed" | "summary_wait") {
                d.ended_ms = Some(now());
            } else {
                d.ended_ms = None;
            }
        }
        self.write();
    }
    pub(super) fn log(&mut self, r: &Reference, event: &str, detail: impl Into<String>) {
        if let Some(d) = self.doc(r) {
            d.started_ms.get_or_insert_with(now);
            d.log.push(Entry {
                at_ms: now(),
                event: event.into(),
                detail: detail.into(),
            });
            if d.log.len() > LOG_LIMIT {
                d.log.remove(0);
            }
        }
        self.write();
    }
    pub(super) fn title(&mut self, r: &Reference, title: Option<&str>) {
        if let (Some(d), Some(t)) = (self.doc(r), title) {
            d.title = Some(t.chars().take(200).collect());
        }
    }
    pub(super) fn fail(&mut self, r: &Reference, error: &str) {
        if let Some(d) = self.doc(r) {
            d.error = Some(error.chars().take(2048).collect());
        }
        self.log(r, "failed", "");
        self.stage(r, "failed");
    }
    fn write(&self) {
        let Some(path) = &self.path else { return };
        let Ok(data) = serde_json::to_vec(self) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = path.with_extension("tmp");
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        use std::io::Write;
        let ok = options
            .open(&tmp)
            .and_then(|mut f| f.write_all(&data))
            .is_ok();
        if ok {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}
