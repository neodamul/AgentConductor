use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Workspace,
    AgentRoom,
}

#[cfg(all(feature = "native-ui-probe", target_os = "macos"))]
pub fn open(room: Arc<super::Room>, surface: Surface) -> Result<(), String> {
    let surface = match surface {
        Surface::Workspace => crate::native_ui_probe::Surface::Workspace,
        Surface::AgentRoom => crate::native_ui_probe::Surface::AgentRoom,
    };
    crate::native_ui_probe::open_window(room, surface)
}

#[cfg(all(feature = "native-ui-probe", windows))]
pub fn open(room: Arc<super::Room>, surface: Surface) -> Result<(), String> {
    let surface = match surface {
        Surface::Workspace => crate::native_ui_probe::Surface::Workspace,
        Surface::AgentRoom => crate::native_ui_probe::Surface::AgentRoom,
    };
    crate::native_ui_probe::open_window(room, surface)
}

#[cfg(all(not(feature = "native-ui-probe"), any(target_os = "macos", windows)))]
pub fn open(_: Arc<super::Room>, _: Surface) -> Result<(), String> {
    Err(crate::ui::messages::text("native.native_ui_disabled_in_this_build").into())
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn open(_: Arc<super::Room>, _: Surface) -> Result<(), String> {
    Err(crate::ui::messages::text("native.native_agent_room_requires_macos_or_windows_use").into())
}
