use super::Probe;

pub(super) fn editable(probe: &Probe) -> bool {
    (probe.pending.is_none()
        || matches!(
            probe.pending,
            Some(
                crate::native_ui_probe::Pending::Metadata
                    | crate::native_ui_probe::Pending::MetadataRefresh
            )
        ))
        && probe
            .snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot["job"]["state"] != "running")
}

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/workspace_permissions.rs"]
mod tests;
