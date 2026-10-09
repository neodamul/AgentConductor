use crate::{
    state::{Store, record_transaction},
    workflow::digest,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    #[serde(default)]
    pub epoch: u64,
    pub task: String,
    pub session: String,
    pub workspace: String,
    pub plan_sha256: String,
    pub workspace_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolRequest {
    pub tool: String,
    pub input: serde_json::Value,
    pub reason: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Pending,
    Allowed,
    Denied,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub context: Context,
    pub request: ToolRequest,
    pub status: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

pub struct Ledger(Connection);

fn payload(context: &Context, request: &ToolRequest) -> Result<String, String> {
    if [
        &context.task,
        &context.session,
        &context.workspace,
        &context.plan_sha256,
        &context.workspace_sha256,
    ]
    .iter()
    .any(|s| s.trim().is_empty() || s.len() > 4096)
        || request.tool.trim().is_empty()
        || request.tool.len() > 256
        || request.reason.trim().is_empty()
        || request.reason.len() > 8192
        || !request.input.is_object()
    {
        return Err(
            "tool request needs a bounded explicit context, object input and reason".into(),
        );
    }
    let value = serde_json::to_string(&(context, request)).map_err(|e| e.to_string())?;
    if value.len() > 64 * 1024 {
        return Err("tool request exceeds 64 KiB".into());
    }
    Ok(value)
}

impl Ledger {
    pub fn receipt(&self, id: &str) -> Result<Option<Receipt>, String> {
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
            if digest(value.as_bytes()) != id {
                return Err("tool request content changed".into());
            }
            let (context, request) = serde_json::from_str(&value).map_err(|e| e.to_string())?;
            Ok(Receipt {
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
    pub fn supersede(&mut self, id: &str) -> Result<(), String> {
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let changed=tx.execute("UPDATE tool_requests SET status='superseded' WHERE id=?1 AND status IN ('pending','approved')",[id]).map_err(|e|e.to_string())?;
        if changed != 1 {
            return Err("only pending or unconsumed approved requests can be refreshed".into());
        }
        record_transaction(
            &tx,
            "tool_superseded",
            &serde_json::json!({"id":id}),
            "Explicit refresh revoked the old request; new permission remains pending",
        )?;
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn open(path: &Path) -> Result<Self, String> {
        drop(Store::open(path)?);
        let db = Connection::open(path).map_err(|e| e.to_string())?;
        db.busy_timeout(Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS tool_requests(
            id TEXT PRIMARY KEY,payload TEXT NOT NULL,status TEXT NOT NULL,
            created_at INTEGER NOT NULL,decided_at INTEGER,expires_at INTEGER);",
        )
        .map_err(|e| e.to_string())?;
        Ok(Self(db))
    }

    pub fn request(
        &mut self,
        context: &Context,
        request: &ToolRequest,
        now: i64,
    ) -> Result<String, String> {
        if now < 0 {
            return Err("invalid request time".into());
        }
        let value = payload(context, request)?;
        let id = digest(value.as_bytes());
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        tx.execute("INSERT OR IGNORE INTO tool_requests(id,payload,status,created_at) VALUES(?1,?2,'pending',?3)",params![id,value,now]).map_err(|e|e.to_string())?;
        record_transaction(
            &tx,
            "tool_request",
            &serde_json::json!({"id":id,"task":context.task,"session":context.session}),
            "Tool approval requested; tool input retained locally and no execution authorized",
        )?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn requests(&self) -> Result<Vec<Receipt>, String> {
        let mut statement=self.0.prepare("SELECT id,payload,status,created_at,expires_at FROM tool_requests ORDER BY created_at DESC,id LIMIT 1000").map_err(|e|e.to_string())?;
        let rows = statement
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|row| {
            let (id, value, status, created_at, expires_at) = row.map_err(|e| e.to_string())?;
            if digest(value.as_bytes()) != id {
                return Err("tool request content changed".into());
            }
            let (context, request) = serde_json::from_str(&value).map_err(|e| e.to_string())?;
            Ok(Receipt {
                id,
                context,
                request,
                status,
                created_at,
                expires_at,
            })
        })
        .collect()
    }

    pub fn decide(
        &mut self,
        id: &str,
        expected: &str,
        allow: bool,
        current: &Context,
        now: i64,
        expires: i64,
    ) -> Result<(), String> {
        if id != expected
            || (allow && (expires <= now || expires.checked_sub(now).is_none_or(|age| age > 900)))
        {
            return Err("tool approval requires the exact request SHA-256 and a lifetime of at most 900 seconds".into());
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let (value, status, created): (String, String, i64) = tx
            .query_row(
                "SELECT payload,status,created_at FROM tool_requests WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|e| e.to_string())?;
        let (original, _): (Context, ToolRequest) =
            serde_json::from_str(&value).map_err(|e| e.to_string())?;
        if original != *current
            || digest(value.as_bytes()) != expected
            || status != "pending"
            || now < created
        {
            return Err("tool request context, content or state changed; decision refused".into());
        }
        let status = if allow { "approved" } else { "denied" };
        tx.execute(
            "UPDATE tool_requests SET status=?2,decided_at=?3,expires_at=?4 WHERE id=?1",
            params![id, status, now, if allow { Some(expires) } else { None }],
        )
        .map_err(|e| e.to_string())?;
        record_transaction(
            &tx,
            "tool_decision",
            &serde_json::json!({"id":id,"status":status,"decided_at":now,"expires_at":if allow {Some(expires)}else{None}}),
            "Tool approval decision recorded; merge approval remains separate",
        )?;
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn consume(
        &mut self,
        id: &str,
        current: &Context,
        request: &ToolRequest,
        now: i64,
    ) -> Result<Decision, String> {
        let value = payload(current, request)?;
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let row = tx
            .query_row(
                "SELECT payload,status,decided_at,expires_at FROM tool_requests WHERE id=?1",
                [id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<i64>>(2)?,
                        r.get::<_, Option<i64>>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((stored, status, decided, expires)) = row else {
            return Ok(Decision::Denied);
        };
        if stored != value || digest(stored.as_bytes()) != id {
            return Ok(Decision::Denied);
        }
        if status == "pending" {
            return Ok(Decision::Pending);
        }
        if status != "approved"
            || !decided.is_some_and(|at| now >= at)
            || !expires.is_some_and(|at| now < at)
        {
            return Ok(Decision::Denied);
        }
        tx.execute(
            "UPDATE tool_requests SET status='consumed' WHERE id=?1 AND status='approved'",
            [id],
        )
        .map_err(|e| e.to_string())?;
        record_transaction(
            &tx,
            "tool_consumed",
            &serde_json::json!({"id":id,"consumed_at":now}),
            "Tool approval consumed exactly once before execution",
        )?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(Decision::Allowed)
    }
}
