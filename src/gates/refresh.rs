use super::*;

impl Ledger {
    /// Replaces an exact pending/approved request in one SQLite transaction.
    /// The task, session, workspace, plan and tool input stay fixed; the caller
    /// must advance the approval epoch and re-verify the live workspace first.
    pub fn refresh(
        &mut self,
        id: &str,
        context: &Context,
        request: &ToolRequest,
        now: i64,
    ) -> Result<String, String> {
        if now < 0 {
            return Err("invalid request time".into());
        }
        let fresh_payload = payload(context, request)?;
        let fresh_id = digest(fresh_payload.as_bytes());
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let (old_payload, status): (String, String) = tx
            .query_row(
                "SELECT payload,status FROM tool_requests WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| error.to_string())?;
        if digest(old_payload.as_bytes()) != id {
            return Err("tool request content changed".into());
        }
        let (old_context, old_request): (Context, ToolRequest) =
            serde_json::from_str(&old_payload).map_err(|error| error.to_string())?;
        if !matches!(status.as_str(), "pending" | "approved")
            || old_request != *request
            || context.task != old_context.task
            || context.session != old_context.session
            || context.workspace != old_context.workspace
            || context.plan_sha256 != old_context.plan_sha256
            || context.epoch <= old_context.epoch
            || fresh_id == id
        {
            return Err(
                "refresh must replace the current request with a newer bound context".into(),
            );
        }
        let changed = tx
            .execute(
                "UPDATE tool_requests SET status='superseded' WHERE id=?1 AND status IN ('pending','approved')",
                [id],
            )
            .map_err(|error| error.to_string())?;
        if changed != 1 {
            return Err("only pending or unconsumed approved requests can be refreshed".into());
        }
        tx.execute(
            "INSERT INTO tool_requests(id,payload,status,created_at) VALUES(?1,?2,'pending',?3)",
            params![fresh_id, fresh_payload, now],
        )
        .map_err(|error| error.to_string())?;
        record_transaction(
            &tx,
            "tool_superseded",
            &serde_json::json!({"id":id,"replacement":fresh_id}),
            "Explicit refresh revoked the old request; replacement remains pending",
        )?;
        record_transaction(
            &tx,
            "tool_request",
            &serde_json::json!({"id":fresh_id,"task":context.task,"session":context.session}),
            "Tool approval request refreshed; no permission was granted",
        )?;
        tx.commit().map_err(|error| error.to_string())?;
        Ok(fresh_id)
    }
}
