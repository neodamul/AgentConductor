use super::*;

pub(super) fn dispatch(
    controller: &Controller,
    request: Request,
) -> Result<serde_json::Value, String> {
    match request {
        Request::Briefing {} => unreachable!("briefing returned before polling mutations"),
        Request::Snapshot {
            icons,
            quota_refresh_last,
        } => controller.snapshot(icons, &quota_refresh_last),
        Request::Evidence {} => controller.evidence(),
        Request::Task { index } => controller.task(index),
        Request::Coordinator {} => controller.coordinator(),
        Request::ScopeFailure {} => workflow::scope_failure(&controller.dir),
        Request::ScopeRecoveryPreview { task, paths, test } => {
            controller.recovery_preview(task, paths, test)
        }
        Request::Action { revision, action } => {
            controller.action(ActionRequest { revision, action })
        }
    }
}
