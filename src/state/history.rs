use super::*;

impl Store {
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
        let activity_key = format!("task_activity/{task}/{attempt}");
        let mut tool: serde_json::Value = tx
            .query_row(
                "SELECT value FROM documents WHERE key=?1",
                params![activity_key],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok())
            .map_or(serde_json::Value::Null, |v| v["tool"].clone());
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
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| e.to_string())?
                        .as_millis() as u64;
                    match crate::activity::tool_change(&event.payload) {
                        crate::activity::Change::Start(name) => {
                            tool = serde_json::json!({"name": name, "since_ms": now});
                        }
                        crate::activity::Change::End => tool = serde_json::Value::Null,
                        crate::activity::Change::None => {}
                    }
                }
            }
        }
        if let Some(event) = last {
            let observed_at_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();
            tx.execute("INSERT INTO documents VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![format!("task_activity/{task}/{attempt}"),serde_json::to_string(&serde_json::json!({"seq":event.seq,"kind":event.kind,"observed_at_ms":observed_at_ms,"tool":tool})).map_err(|e|e.to_string())?])
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

    /// The newest `limit` audit records with their summaries, oldest first, without the
    /// per-event progress records of agents.
    pub fn history(&self, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let mut query = self
            .0
            .prepare(
                "SELECT a.seq,a.event,a.payload,COALESCE(m.content,'') FROM audit a \
                 LEFT JOIN memory_outbox m ON m.id=a.seq WHERE a.event<>'intermediate' \
                 ORDER BY a.seq DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = query
            .query_map(params![limit as i64], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            let (seq, event, payload, summary) = row.map_err(|e| e.to_string())?;
            let payload: serde_json::Value =
                serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null);
            result.push(
                serde_json::json!({"seq":seq,"event":event,"summary":summary,"payload":payload}),
            );
        }
        result.reverse();
        Ok(result)
    }

    /// Audit records of the given event kinds with their summaries, oldest first. The kinds are
    /// fixed names from the program, never operator input.
    pub fn audit_of(&self, kinds: &[&str], limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let marks = vec!["?"; kinds.len()].join(",");
        let sql = format!(
            "SELECT a.seq,a.event,a.payload,COALESCE(m.content,'') FROM audit a \
             LEFT JOIN memory_outbox m ON m.id=a.seq WHERE a.event IN ({marks}) \
             ORDER BY a.seq DESC LIMIT {limit}"
        );
        let mut query = self.0.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = query
            .query_map(rusqlite::params_from_iter(kinds.iter()), |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            let (seq, event, payload, summary) = row.map_err(|e| e.to_string())?;
            let payload = serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null);
            result.push(
                serde_json::json!({"seq":seq,"event":event,"summary":summary,"payload":payload}),
            );
        }
        result.reverse();
        Ok(result)
    }

    pub fn count_events(&self, kinds: &[&str]) -> Result<usize, String> {
        let marks = vec!["?"; kinds.len()].join(",");
        self.0
            .query_row(
                &format!("SELECT COUNT(*) FROM audit WHERE event IN ({marks})"),
                rusqlite::params_from_iter(kinds.iter()),
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
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
}
