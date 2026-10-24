use super::{Pending, Probe, room};
use serde_json::Value;

pub(super) fn read_reply(probe: &mut Probe, value: Value) {
    match probe.pending {
        Some(Pending::Evidence) => {
            let result = probe
                .room_view
                .as_mut()
                .ok_or_else(|| "Agent Room is unavailable".to_owned())
                .and_then(|view| room::evidence_reply(view, probe.snapshot.as_ref(), value));
            match result {
                Ok(()) => {
                    probe.pending = None;
                    probe.error = None;
                }
                Err(error) => {
                    probe.pending = None;
                    probe.error = Some(error);
                }
            }
        }
        Some(Pending::ScopeFailure) => probe.apply_scope_failure(value),
        Some(Pending::ScopeRecoveryPreview) => probe.apply_scope_preview(value),
        _ => {
            probe.pending = None;
            probe.error = Some("Unexpected recovery response".into());
        }
    }
}
