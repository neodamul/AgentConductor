use super::editable;
use crate::native_ui_probe::{Pending, Probe};
use serde_json::json;

#[test]
fn slow_quota_refresh_does_not_disable_workspace_editing() {
    let mut probe = Probe::default();
    probe.snapshot = Some(json!({"job":{"state":"idle"}}));
    probe.pending = Some(Pending::Metadata);

    assert!(editable(&probe));
}

#[test]
fn workspace_save_still_disables_editing() {
    let mut probe = Probe::default();
    probe.snapshot = Some(json!({"job":{"state":"idle"}}));
    probe.pending = Some(Pending::Save);

    assert!(!editable(&probe));
}
