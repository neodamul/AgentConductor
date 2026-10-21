//! GitHub #122: the Agent Room offers the recovery panel for out-of-scope failures.
const HTML: &str = include_str!("../src/ui/room.html");
const SCRIPT: &str = include_str!("../src/ui/room.js");

#[test]
fn the_room_embeds_the_recovery_button_dialog_and_wiring() {
    let body = &HTML[..HTML.find("</body>").unwrap()];
    for id in [
        "scope-review",
        "scope-dialog",
        "scope-task",
        "scope-paths",
        "scope-test",
        "scope-preview",
        "scope-approve",
        "scope-offending",
        "scope-tasks",
        "scope-stray",
        "scope-proposal",
    ] {
        assert!(
            body.contains(&format!("id=\"{id}\"")),
            "{id} is inside the page body"
        );
    }
    for call in [
        "kind: 'scope_failure'",
        "kind: 'scope_preview'",
        "kind: 'scope_recovery'",
        "scopeOffered(snapshot)",
    ] {
        assert!(SCRIPT.contains(call), "{call}");
    }
}

#[test]
fn every_id_the_recovery_script_uses_exists_in_the_page() {
    let start = SCRIPT.find("function scopeLines(").unwrap();
    let end = SCRIPT[start..].find("$('scope-task').onchange").unwrap() + start;
    for part in SCRIPT[start..end]
        .split("$('")
        .skip(1)
        .chain(SCRIPT[start..end].split("scopeListFill('").skip(1))
    {
        let id = part.split('\'').next().unwrap();
        assert!(
            HTML.contains(&format!("id=\"{id}\"")),
            "script uses missing id {id}"
        );
    }
}

#[test]
fn the_recovery_panel_script_previews_then_requires_the_reviewed_digest() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/scope_recovery_runtime.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn the_recovery_dialog_scrolls_on_short_windows() {
    assert!(
        include_str!("../src/ui/room.css").contains("#scope-dialog{max-height:90vh;overflow:auto}")
    );
}
