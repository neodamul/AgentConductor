pub fn pending_message(kind: &str) -> &'static str {
    match kind {
        "folder" => "Choosing project folder…",
        "prepare" => "Starting plan generation…",
        "metadata" => "Queueing agent metadata refresh…",
        "metadata_refresh" => "Refreshing agent metadata…",
        "start" => "Starting agent execution…",
        "plan_refresh" => "Refreshing plan progress…",
        "workspace" => "Updating workspaces…",
        _ => "Saving workspace…",
    }
}
