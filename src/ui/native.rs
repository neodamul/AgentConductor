use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Workspace,
    AgentRoom,
}

#[cfg(all(target_os = "macos", not(feature = "native-ui-probe")))]
mod cocoa;
/// Windows shell: the shared pages in the system WebView2 runtime with the same
/// promise-based request contract as WKWebView and no local server (DD-38).
#[cfg(all(windows, not(feature = "native-ui-probe")))]
mod webview2;

#[cfg(target_os = "macos")]
pub fn open(room: Arc<super::Room>, surface: Surface) -> Result<(), String> {
    #[cfg(feature = "native-ui-probe")]
    {
        let surface = match surface {
            Surface::Workspace => crate::native_ui_probe::Surface::Workspace,
            Surface::AgentRoom => crate::native_ui_probe::Surface::AgentRoom,
        };
        crate::native_ui_probe::open_window(room, surface)
    }
    #[cfg(not(feature = "native-ui-probe"))]
    {
        let _ = surface;
        cocoa::open(room)
    }
}
#[cfg(windows)]
pub fn open(room: Arc<super::Room>, surface: Surface) -> Result<(), String> {
    #[cfg(feature = "native-ui-probe")]
    {
        let surface = match surface {
            Surface::Workspace => crate::native_ui_probe::Surface::Workspace,
            Surface::AgentRoom => crate::native_ui_probe::Surface::AgentRoom,
        };
        crate::native_ui_probe::open_window(room, surface)
    }
    #[cfg(not(feature = "native-ui-probe"))]
    {
        let _ = surface;
        webview2::open(room)
    }
}
#[cfg(not(any(target_os = "macos", windows)))]
pub fn open(_: Arc<super::Room>, _: Surface) -> Result<(), String> {
    Err(crate::ui::messages::text("native.native_agent_room_requires_macos_or_windows_use").into())
}
