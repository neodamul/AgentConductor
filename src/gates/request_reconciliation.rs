use super::*;

impl Ledger {
    /// Atomically replaces same-task/session approvals whose workspace context changed.
    pub fn request_replacing_changed_context(
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
        let mut statement = tx
            .prepare("SELECT id,payload FROM tool_requests WHERE status IN ('pending','approved')")
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let rows = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        drop(statement);
        for (previous_id, previous_value) in rows {
            if digest(previous_value.as_bytes()) != previous_id {
                return Err("tool request content changed".into());
            }
            let (previous_context, previous_request): (Context, ToolRequest) =
                serde_json::from_str(&previous_value).map_err(|e| e.to_string())?;
            if previous_context.task == context.task
                && previous_context.session == context.session
                && previous_context != *context
                && previous_request == *request
            {
                tx.execute(
                    "UPDATE tool_requests SET status='superseded' WHERE id=?1 AND status IN ('pending','approved')",
                    [&previous_id],
                )
                .map_err(|e| e.to_string())?;
                record_transaction(
                    &tx,
                    "tool_superseded",
                    &serde_json::json!({"id":previous_id}),
                    "Workspace context changed; stale approval was revoked before a fresh request",
                )?;
            }
        }
        tx.execute(
            "INSERT OR IGNORE INTO tool_requests(id,payload,status,created_at) VALUES(?1,?2,'pending',?3)",
            params![id, value, now],
        )
        .map_err(|e| e.to_string())?;
        record_transaction(
            &tx,
            "tool_request",
            &serde_json::json!({"id":id,"task":context.task,"session":context.session}),
            "Tool approval requested; tool input retained locally and no execution authorized",
        )?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(id)
    }
}
