# Windows viewport resizing

Verified: 2026-11-12. Status: implementation pending native Windows acceptance.

## Context and observation

[Issue #853](https://github.com/neodamul/AgentConductor/issues/853) reports that
resizing or maximizing the internally distributed v0.2.1 window leaves its content
at the previous dimensions. The user reports normal behavior before v0.2.0.

## Design and evidence

The Windows shell relied on Wry's automatic parent resize handling. The same
pattern exists in v0.1.61, so the introducing revision is not established.
The explicit fallback synchronizes the WebView bounds on native resize and DPI
change events using physical client pixels. Zero-size minimized bounds are ignored.
There is no polling, new thread, dependency, or change to close/IPC approval handling.

The Windows UI Automation acceptance compares document size changes with window
size changes for shrinking, growing, maximizing and restoring. Physical dimensions
avoid applying DPI scaling twice. Local macOS formatting and close behavior checks
pass; macOS skipping the Windows test is not Windows verification. Revalidate on
Windows/WebView2 updates, multi-monitor DPI changes, and any desktop shell change.

Sources: [observation](../raw/windows-resize-853.json),
[Windows shell](../../src/ui/native/webview2.rs),
[acceptance test](../../tests/windows_desktop.rs),
[direct native transport](direct-native-transport.md).
