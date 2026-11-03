use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Workspace,
    AgentRoom,
}

/// R-118: asks the page whether to close while a run works; a page without the dialog confirms at once.
#[cfg(all(any(target_os = "macos", windows), not(feature = "native-ui-probe")))]
const CLOSE_REQUESTED: &str = "window.__acCloseRequested?window.__acCloseRequested():window.__acRequest(JSON.stringify({kind:'close_window',stop:false}))";
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
