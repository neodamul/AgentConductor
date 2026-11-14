use super::*;

pub(super) fn public_report(report: &Report) -> Result<serde_json::Value, String> {
    let mut value = serde_json::to_value(report).map_err(|e| e.to_string())?;
    value["direct_workspace"] = serde_json::json!(
        !report.repo.join(".git").exists()
            && crate::folder::external_metadata(&report.repo)?.is_some()
    );
    if let Some(agents) = value
        .pointer_mut("/plan/agents")
        .and_then(serde_json::Value::as_array_mut)
    {
        for agent in agents {
            agent
                .as_object_mut()
                .ok_or(crate::ui::messages::text("native.invalid_profile"))?
                .remove("environment");
        }
    }
    if let Some(folder) = value
        .get_mut("folder")
        .and_then(serde_json::Value::as_object_mut)
    {
        folder.remove("files");
    }
    // DD-113: why the orchestrator's integration verification failed, readable in the room.
    if let Some(tests) = value.get_mut("tests").and_then(|t| t.as_array_mut()) {
        for (test, source) in tests.iter_mut().zip(&report.tests) {
            test["passed"] = serde_json::json!(report.test_passed(source));
            if !report.test_passed(source) {
                let read = |path| tail(path, 2048).unwrap_or_default();
                let output =
                    failure_output::failure_output(&read(&source.stderr), &read(&source.stdout));
                test["output"] = serde_json::json!(output);
            }
        }
    }
    Ok(value)
}
