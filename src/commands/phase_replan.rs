//! Internal background worker, invoked only from the local workspace coordinator.
pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    let [kind, path] = args else {
        return None;
    };
    if kind == "phase-auto-merge" {
        return Some(
            agentconductor::workflow::automatic_merge(std::path::Path::new(path)).map(|_| ()),
        );
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
