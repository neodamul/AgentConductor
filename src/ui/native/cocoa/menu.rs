//! Native menu labels use the same message IDs as the web interface.
use super::{Id, class, sel, send0, send1, string};

fn menu_item(title: &str, key: &str, action: &str, target: Id) -> Result<Id, String> {
    let item = send0(class("NSMenuItem"), "new");
    send1(item, "setTitle:", string(title)?);
    send1(item, "setKeyEquivalent:", string(key)?);
    send1(item, "setAction:", sel(action));
    send1(item, "setTarget:", target);
    Ok(item)
}
pub(super) fn install(app: Id, window: Id) -> Result<(), String> {
    let main = send0(class("NSMenu"), "new");
    for (title, items) in [
        (
            "AgentConductor",
            vec![(
                crate::ui::messages::text("native.quit_agentconductor"),
                "q",
                "performClose:",
                window,
            )],
        ),
        (
            crate::ui::messages::text("native.edit"),
            vec![
                (
                    crate::ui::messages::text("native.undo"),
                    "z",
                    "undo:",
                    std::ptr::null_mut(),
                ),
                (
                    crate::ui::messages::text("native.cut"),
                    "x",
                    "cut:",
                    std::ptr::null_mut(),
                ),
                (
                    crate::ui::messages::text("native.copy"),
                    "c",
                    "copy:",
                    std::ptr::null_mut(),
                ),
                (
                    crate::ui::messages::text("native.paste"),
                    "v",
                    "paste:",
                    std::ptr::null_mut(),
                ),
                (
                    crate::ui::messages::text("native.select_all"),
                    "a",
                    "selectAll:",
                    std::ptr::null_mut(),
                ),
            ],
        ),
        (
            crate::ui::messages::text("native.window"),
            vec![(
                crate::ui::messages::text("native.close_window"),
                "w",
                "performClose:",
                window,
            )],
        ),
    ] {
        let parent = send0(class("NSMenuItem"), "new");
        send1(parent, "setTitle:", string(title)?);
        let submenu = send0(class("NSMenu"), "new");
        send1(submenu, "setTitle:", string(title)?);
        for (title, key, action, target) in items {
            let item = menu_item(title, key, action, target)?;
            send1(submenu, "addItem:", item);
            send0(item, "release");
        }
        send1(parent, "setSubmenu:", submenu);
        send1(main, "addItem:", parent);
        send0(submenu, "release");
        send0(parent, "release");
    }
    send1(app, "setMainMenu:", main);
    send0(main, "release");
    Ok(())
}
