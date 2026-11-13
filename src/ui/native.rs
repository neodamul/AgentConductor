use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Workspace,
    AgentRoom,
}

/// R-118: ask the page to confirm closing while a run is active.
#[cfg(any(target_os = "macos", windows))]
const CLOSE_REQUESTED: &str = "window.__acCloseRequested?window.__acCloseRequested():window.__acRequest(JSON.stringify({kind:'close_window',stop:false}))";
#[cfg(target_os = "macos")]
mod cocoa;
/// Windows uses the system WebView2 runtime and the same request contract as WKWebView.
#[cfg(windows)]
mod webview2;

#[cfg(target_os = "macos")]
pub fn open(room: Arc<super::Room>, _: Surface) -> Result<(), String> {
    cocoa::open(room)
}
#[cfg(windows)]
pub fn open(room: Arc<super::Room>, _: Surface) -> Result<(), String> {
    webview2::open(room)
}
#[cfg(not(any(target_os = "macos", windows)))]
pub fn open(_: Arc<super::Room>, _: Surface) -> Result<(), String> {
    Err(crate::ui::messages::text("native.native_agent_room_requires_macos_or_windows_use").into())
}
