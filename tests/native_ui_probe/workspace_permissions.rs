use super::editable;
use crate::native_ui_probe::{Pending, Probe};
use serde_json::json;

#[test]
fn slow_quota_refresh_does_not_disable_workspace_editing() {
    let probe = Probe {
        snapshot: Some(json!({"job":{"state":"idle"}})),
        pending: Some(Pending::Metadata),
        ..Default::default()
    };

    assert!(editable(&probe));
}

#[test]
fn workspace_save_still_disables_editing() {
    let probe = Probe {
        snapshot: Some(json!({"job":{"state":"idle"}})),
        pending: Some(Pending::Save),
        ..Default::default()
    };

    assert!(!editable(&probe));
}
