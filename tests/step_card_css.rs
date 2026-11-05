//! #543/#544/#545: layout rules of the step cards, check rows and the Codex logo are contract-tested
//! at source level; the rendered result is covered by design_parity.
const STARTUP: &str = include_str!("../src/ui/startup.css");
const ROOM: &str = include_str!("../src/ui/room.css");
const MOCK: &str = include_str!("../docs/design/approved-goal-ui/preview.css");

#[test]
fn only_the_step_card_body_scrolls_and_its_header_and_footer_stay() {
    assert!(STARTUP.contains(".startup-main{display:flex;flex-direction:column;overflow:hidden}"));
    assert!(STARTUP.contains(
        ".startup-main>.step-card:not([hidden]){flex:1 1 auto;min-height:0;overflow:auto}"
    ));
    assert!(
        STARTUP.contains(".step-card>.section-title{position:sticky;top:0")
            || STARTUP.contains(".step-card .section-title{position:sticky;top:0")
    );
    assert!(STARTUP.contains("position:sticky;bottom:0"));
}

#[test]
fn a_check_box_never_grows_and_pushes_its_label_to_the_right() {
    assert!(STARTUP.contains(".check-row input{width:auto;margin:0;flex:none}"));
}

#[test]
fn the_codex_logo_has_no_white_tile_in_the_app_and_the_mock() {
    assert!(ROOM.contains(".app-icon>svg.brand-logo>path[fill=\"#fff\"]{display:none}"));
    assert!(MOCK.contains(".agent-brand-logo>path[fill=\"#fff\"]{display:none}"));
}
