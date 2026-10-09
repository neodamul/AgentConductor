use rusqlite::{Connection, params};
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

pub struct Store(Connection);

impl Store {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let db = Connection::open(path).map_err(|e| e.to_string())?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS runs(id TEXT PRIMARY KEY, status TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS events(run_id TEXT NOT NULL REFERENCES runs(id), seq INTEGER NOT NULL,
                kind TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(run_id,seq));").map_err(|e| e.to_string())?;
        Ok(Self(db))
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
}
