use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Prepared,
    Ready,
    Starting,
    Running,
    Stopping,
    Stopped,
    Completed,
    Failed,
}

pub struct Store(pub(crate) Connection);

impl Store {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let db = Connection::open(path).map_err(|e| e.to_string())?;
        db.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS runs(id TEXT PRIMARY KEY, status TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS events(run_id TEXT NOT NULL REFERENCES runs(id), seq INTEGER NOT NULL,
                kind TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(run_id,seq));
            CREATE TABLE IF NOT EXISTS documents(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS task_events(task TEXT NOT NULL, attempt INTEGER NOT NULL, seq INTEGER NOT NULL,
                value TEXT NOT NULL, PRIMARY KEY(task,attempt,seq));
            CREATE TABLE IF NOT EXISTS audit(seq INTEGER PRIMARY KEY AUTOINCREMENT, event TEXT NOT NULL, payload TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS memory_outbox(id INTEGER PRIMARY KEY, content TEXT NOT NULL, delivered INTEGER NOT NULL DEFAULT 0);
            CREATE VIRTUAL TABLE IF NOT EXISTS memory_search USING fts5(memory_id UNINDEXED, content);").map_err(|e| e.to_string())?;
        Ok(Self(db))
    }

    /// Opens an existing database without schema changes, writes, or run creation.
    /// All subsequent reads share one SQLite transaction snapshot.
    pub fn read_only_snapshot(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let db = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| e.to_string())?;
        db.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        db.execute_batch("BEGIN DEFERRED")
            .map_err(|e| e.to_string())?;
        Ok(Self(db))
    }

    pub fn tool_receipt(&self, id: &str) -> Result<Option<crate::gates::Receipt>, String> {
        let row: Option<(String, String, i64, Option<i64>)> = self
            .0
            .query_row(
                "SELECT payload,status,created_at,expires_at FROM tool_requests WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        row.map(|(value, status, created_at, expires_at)| {
            if crate::workflow::digest(value.as_bytes()) != id {
                return Err("tool request content changed".into());
            }
            let (context, request) = serde_json::from_str(&value).map_err(|e| e.to_string())?;
            Ok(crate::gates::Receipt {
                id: id.into(),
                context,
                request,
                status,
                created_at,
                expires_at,
            })
        })
        .transpose()
    }

    pub fn create(&self, id: &str) -> Result<(), String> {
        self.0
            .execute(
                "INSERT INTO runs VALUES(?1,?2)",
                params![id, serde_json::to_string(&Status::Prepared).unwrap()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn status(&self, id: &str) -> Result<Status, String> {
        let value: String = self
            .0
            .query_row("SELECT status FROM runs WHERE id=?1", [id], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&value).map_err(|e| e.to_string())
    }

    pub fn transition(&mut self, id: &str, next: Status) -> Result<(), String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        let current: String = tx
            .query_row("SELECT status FROM runs WHERE id=?1", [id], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let current: Status = serde_json::from_str(&current).map_err(|e| e.to_string())?;
        use Status::*;
        if !matches!(
            (current, next),
            (Prepared, Ready)
                | (Ready, Starting)
                | (Starting, Running | Failed)
                | (Running, Stopping | Completed | Failed)
                | (Stopping, Stopped | Failed)
                | (Stopped, Ready)
        ) {
            return Err(format!("invalid transition: {current:?} -> {next:?}"));
        }
        let payload = serde_json::to_string(&next).map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE runs SET status=?2 WHERE id=?1",
            params![id, payload],
        )
        .map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO events SELECT ?1, COALESCE(MAX(seq),-1)+1, 'status', ?2 FROM events WHERE run_id=?1",
            params![id, payload]).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn append(&self, id: &str, seq: u64, kind: &str, payload: &str) -> Result<bool, String> {
        let seq = i64::try_from(seq).map_err(|_| "event sequence overflow".to_string())?;
        self.0
            .execute(
                "INSERT OR IGNORE INTO events VALUES(?1,?2,?3,?4)",
                params![id, seq, kind, payload],
            )
            .map(|n| n == 1)
            .map_err(|e| e.to_string())
    }

    pub fn events(&self, id: &str) -> Result<Vec<String>, String> {
        let mut statement = self
            .0
            .prepare("SELECT kind FROM events WHERE run_id=?1 ORDER BY seq")
            .map_err(|e| e.to_string())?;
        statement
            .query_map([id], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn document(&self, key: &str) -> Result<serde_json::Value, String> {
        let value: String = self
            .0
            .query_row("SELECT value FROM documents WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&value).map_err(|e| e.to_string())
    }

    pub fn optional_document(&self, key: &str) -> Result<Option<serde_json::Value>, String> {
        let value: Option<String> = self
            .0
            .query_row("SELECT value FROM documents WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        value
            .map(|v| serde_json::from_str(&v).map_err(|e| e.to_string()))
            .transpose()
    }

    pub fn append_task_events(
        &mut self,
        task: &str,
        attempt: u64,
        events: &[crate::protocol::Event],
    ) -> Result<(), String> {
        if task.is_empty() || task.len() > 256 || attempt == 0 {
            return Err("invalid task event identity".into());
        }
        let attempt = i64::try_from(attempt).map_err(|_| "attempt overflow")?;
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let mut last = None;
        for event in events {
            let seq = i64::try_from(event.seq).map_err(|_| "sequence overflow")?;
            let value = serde_json::to_string(event).map_err(|e| e.to_string())?;
            let prior: Option<String> = tx
                .query_row(
                    "SELECT value FROM task_events WHERE task=?1 AND attempt=?2 AND seq=?3",
                    params![task, attempt, seq],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            match prior {
                Some(prior) if prior != value => {
                    return Err("normalized task evidence changed".into());
                }
                Some(_) => {}
                None => {
                    tx.execute(
                        "INSERT INTO task_events VALUES(?1,?2,?3,?4)",
                        params![task, attempt, seq, value],
                    )
                    .map_err(|e| e.to_string())?;
                    last = Some(event);
                }
            }
        }
        if let Some(event) = last {
            let observed_at_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();
            tx.execute("INSERT INTO documents VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![format!("task_activity/{task}/{attempt}"),serde_json::to_string(&serde_json::json!({"seq":event.seq,"kind":event.kind,"observed_at_ms":observed_at_ms})).map_err(|e|e.to_string())?])
                .map_err(|e|e.to_string())?;
            record_transaction(
                &tx,
                "intermediate",
                &serde_json::json!({"task":task,"attempt":attempt,"seq":event.seq,"kind":event.kind}),
                &format!(
                    "Task {task}, attempt {attempt}: intermediate {:?} report at sequence {}",
                    event.kind, event.seq
                ),
            )?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    /// The newest `limit` audit payloads of one event kind, oldest first.
    pub fn audit(&self, event: &str, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let mut query = self
            .0
            .prepare("SELECT payload FROM audit WHERE event=?1 ORDER BY seq DESC LIMIT ?2")
            .map_err(|e| e.to_string())?;
        let rows = query
            .query_map(params![event, limit as i64], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            result.push(
                serde_json::from_str(&row.map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?,
            );
        }
        result.reverse();
        Ok(result)
    }

    pub fn task_events(&self, task: &str, attempt: u64) -> Result<Vec<serde_json::Value>, String> {
        let attempt = i64::try_from(attempt).map_err(|_| "attempt overflow")?;
        let mut query=self.0.prepare("SELECT value FROM task_events WHERE task=?1 AND attempt=?2 ORDER BY seq DESC LIMIT 1000").map_err(|e|e.to_string())?;
        let rows = query
            .query_map(params![task, attempt], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        let mut bytes = 0usize;
        for row in rows {
            let value = row.map_err(|e| e.to_string())?;
            bytes = bytes.saturating_add(value.len());
            if bytes > 1024 * 1024 && !result.is_empty() {
                break;
            }
            result.push(serde_json::from_str(&value).map_err(|e| e.to_string())?);
        }
        result.reverse();
        Ok(result)
    }

    pub fn record(
        &mut self,
        key: &str,
        value: &serde_json::Value,
        summary: &str,
    ) -> Result<(), String> {
        let tx = self.0.transaction().map_err(|e| e.to_string())?;
        record_transaction(&tx, key, value, summary)?;
        tx.commit().map_err(|e| e.to_string())
    }

    /// Append an immutable receipt once, with audit/outbox in the same transaction.
    pub fn record_once(
        &mut self,
        key: &str,
        value: &serde_json::Value,
        summary: &str,
    ) -> Result<bool, String> {
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let prior: Option<String> = tx
            .query_row("SELECT value FROM documents WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(prior) = prior {
            let prior: serde_json::Value =
                serde_json::from_str(&prior).map_err(|e| e.to_string())?;
            if prior != *value {
                return Err("immutable receipt content changed".into());
            }
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(false);
        }
        record_transaction(&tx, key, value, summary)?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(true)
    }

    /// Next 128 pending observations; remaining rows stay durable for a later sync.
    pub fn pending_memory(&self) -> Result<Vec<(i64, String)>, String> {
        let mut query = self
            .0
            .prepare("SELECT id,content FROM memory_outbox WHERE delivered=0 ORDER BY id LIMIT 128")
            .map_err(|e| e.to_string())?;
        query
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    pub fn memory_delivered(&self, id: i64) -> Result<(), String> {
        let changed = self
            .0
            .execute("UPDATE memory_outbox SET delivered=1 WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err("unknown memory outbox item".into());
        }
        Ok(())
    }
    pub fn recall(&self, query: &str) -> Result<Vec<String>, String> {
        let query = query
            .split_whitespace()
            .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        if query.is_empty() {
            return Ok(vec![]);
        }
        let mut statement=self.0.prepare("SELECT content FROM memory_search WHERE memory_search MATCH ?1 ORDER BY rank LIMIT 20").map_err(|e|e.to_string())?;
        statement
            .query_map([query], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}

pub(crate) fn record_transaction(
    tx: &rusqlite::Transaction<'_>,
    key: &str,
    value: &serde_json::Value,
    summary: &str,
) -> Result<(), String> {
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO documents VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, json],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO audit(event,payload) VALUES(?1,?2)",
        params![key, json],
    )
    .map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO memory_outbox(id,content) VALUES(?1,?2)",
        params![id, summary],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO memory_search(memory_id,content) VALUES(?1,?2)",
        params![id, summary],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
