use super::Probe;

pub(super) fn editable(probe: &Probe) -> bool {
    probe.pending.is_none()
        && probe
            .snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot["job"]["state"] != "running")
}
