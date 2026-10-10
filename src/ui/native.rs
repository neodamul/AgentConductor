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
    Err("Native Agent Room requires macOS or Windows; use the CLI on this platform".into())
}
