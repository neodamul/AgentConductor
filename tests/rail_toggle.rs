//! GitHub #408 / behavior.md: a titlebar button hides and restores the whole left rail on both screens, and the
//! centre takes the space. The button sits before the Workspace name.
const START_HTML: &str = include_str!("../src/ui/startup.html");
const START_JS: &str = include_str!("../src/ui/startup.js");
const START_CSS: &str = include_str!("../src/ui/startup.css");
const ROOM_CSS: &str = include_str!("../src/ui/room.css");

#[test]
fn the_left_rail_toggle_precedes_the_workspace_name_and_hides_both_rails() {
    let toggle = START_HTML.find("id=\"start-rail-toggle\"").unwrap();
    assert!(toggle < START_HTML.find("id=\"start-workspace-menu\"").unwrap());
    assert!(START_HTML.contains("data-i18n-title=\"startup.rail_toggle\""));
    assert!(START_JS.contains("document.body.classList.toggle('rail-hidden')"));
    assert!(
        START_CSS
            .contains("body.rail-hidden .startup-shell{grid-template-columns:0 minmax(0,1fr)}")
    );
    assert!(
        ROOM_CSS.contains(
            "body.rail-hidden .workbench{grid-template-columns:0 minmax(390px,1fr) 306px}"
        )
    );
    assert!(ROOM_CSS.contains(
        "body.rail-hidden .workbench.insp-closed{grid-template-columns:0 minmax(390px,1fr) 0}"
    ));
}
