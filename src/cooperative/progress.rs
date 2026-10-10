use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub address: Address,
    pub sequence: u64,
    pub reported_at_ms: u64,
    pub phase: String,
    pub summary: String,
    pub next_step: String,
}
/// Stores a bounded observation. No report text grants permissions or task transitions.
pub fn report_progress(store: &mut Store, progress: Progress, now: u64) -> Result<bool, String> {
    report_progress_from(store, progress, now, "local checkpoint adapter")
}
pub(crate) fn report_progress_from(
    store: &mut Store,
    progress: Progress,
    now: u64,
    source: &str,
) -> Result<bool, String> {
    if !matches!(
        source,
        "local checkpoint adapter" | "agentmemory exact-ID adapter"
    ) {
        return Err("invalid checkpoint observation source".into());
    }
    validate_text(&progress.phase, 400)?;
    validate_text(&progress.summary, 1600)?;
    validate_text(&progress.next_step, 1600)?;
    if progress.sequence == 0 || progress.reported_at_ms == 0 || progress.reported_at_ms > now {
        return Err("invalid cooperative progress sequence or time".into());
    }
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    validate_address(&tx, &progress.address)?;
    let packet_key = format!(
        "cooperative_progress/{}/{}",
        progress.address.task, progress.address.attempt
    );
    let prior: Option<String> = tx
        .query_row(
            "SELECT value FROM documents WHERE key=?1",
            [&packet_key],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(prior) = prior {
        let prior: Progress = serde_json::from_str(&prior).map_err(|e| e.to_string())?;
        if prior == progress {
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(false);
        }
        if progress.sequence <= prior.sequence || progress.reported_at_ms < prior.reported_at_ms {
            return Err("cooperative report is older or conflicts with recorded evidence".into());
        }
    }
    let packet = serde_json::to_value(&progress).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO documents VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![
            packet_key,
            serde_json::to_string(&packet).map_err(|e| e.to_string())?
        ],
    )
    .map_err(|e| e.to_string())?;
    let a = &progress.address;
    let value = serde_json::json!({"agent":a.agent,"task":a.task,"attempt":a.attempt,"session_id":a.session_id,"sequence":progress.sequence,"reported_at_ms":progress.reported_at_ms,"received_at_ms":now,"phase":progress.phase,"summary":progress.summary,"next_step":progress.next_step,"source":source});
    let summary = serde_json::to_string(
        &serde_json::json!({"protocol":"agentconductor.progress.v1","progress":progress}),
    )
    .map_err(|e| e.to_string())?;
    record_transaction(
        &tx,
        &format!("cooperative_report/{}/{}", a.task, a.attempt),
        &value,
        &summary,
    )?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(true)
}
