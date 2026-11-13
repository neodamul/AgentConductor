//! Internal background worker, invoked only from the local workspace coordinator.
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    let [kind, path] = args else {
        return None;
    };
    if kind == "phase-auto-merge" {
        return Some((|| {
            let dir = std::path::Path::new(path);
            if agentconductor::workflow::automatic_merge(dir)? {
                return Ok(());
            }
            let review = agentconductor::workflow::load(dir).is_ok_and(|r| {
                r.review_round > 0
                    || r.review_error.is_some()
                    || r.latest_review
                        .as_ref()
                        .is_some_and(|v| v.verdict != agentconductor::coordinator::Verdict::Accept)
            });
            Err(agentconductor::ui::messages::text(if review {
                "native.automatic_merge_paused_review"
            } else {
                "native.automatic_merge_paused_gate"
            })
            .into())
        })());
    }
    if kind != "phase-replan" {
        return None;
    }
    Some((|| {
        let request: agentconductor::phase_replan::Request =
            serde_json::from_slice(&agentconductor::mcp::load(std::path::Path::new(path))?)
                .map_err(|e| e.to_string())?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        runtime.block_on(agentconductor::phase_replan::generate(request))
    })())
}
