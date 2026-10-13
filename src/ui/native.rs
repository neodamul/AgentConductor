#[cfg(target_os = "macos")]
mod cocoa;
/// Windows shell: the shared pages in the system WebView2 runtime with the same
/// promise-based request contract as WKWebView and no local server (DD-38).
#[cfg(windows)]
mod webview2;

#[cfg(target_os = "macos")]
pub fn open(room: std::sync::Arc<super::Room>) -> Result<(), String> {
    cocoa::open(room)
}
#[cfg(windows)]
pub fn open(room: std::sync::Arc<super::Room>) -> Result<(), String> {
    webview2::open(room)
}
#[cfg(not(any(target_os = "macos", windows)))]
pub fn open(_: std::sync::Arc<super::Room>) -> Result<(), String> {
    Err(crate::ui::messages::text("native.native_agent_room_requires_macos_or_windows_use").into())
}
