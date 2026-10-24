# Current local verification

At issue #9 branch head `3ae08e2db6cbf0c02c74ac390fdbc9e3218ae889`, `cargo test --locked --offline` completed with 963 passed, 14 ignored, 0 failed across 321 test binaries in 245.25 seconds. This is host/default-feature test evidence only. It adds no packaged UI workflow acceptance: the count remains 20/25, and shipped/default Rust UI remains 0/25.

The v0.1.37 arm64 package bundle has a valid `Info.plist`; `Contents/MacOS/agentconductor` exists, is executable, and is an arm64 Mach-O. A prior LaunchServices attempt returned `kLSNoExecutableErr`, and a temporary uniquely identified app copy aborted before exposing a UI window. The exact launch/window-binding failure remains unresolved; no package UI acceptance is claimed from those attempts.

The user reports the Windows UTM VM does not boot and has no display following the approved reset. Keep VM power, reset, share, and configuration controls untouched unless the user explicitly authorizes a specific recovery step. Windows UI/runtime, screen-reader/Korean IME, resource, final license, and stable release gates remain open.

Evidence: `docs/evidence/issue9-full-suite-3ae08e2-20261024.json`.

## macOS native launch failure

Two separate package bundle IDs (the temporary Workflow20 bundle and the standard `dev.agentconductor.desktop` bundle) produced the same host crash: SIGABRT in `___RegisterApplication_block_invoke` during AppKit `NSApplication` registration, with LaunchServices `_LSCopyLocalDatabase` on a background thread. The application does not reach the eframe window callback. Static package checks pass (valid plist, executable path/name, mode 0755, arm64 Mach-O). The OS-level failure cause is still unknown; no product code or LaunchServices database was changed. See `docs/evidence/issue9-macos-ui-launch-crash-20261024.json`.
