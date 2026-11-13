# Alternate Rust UI retirement

Issue #940, milestone 0.2.9. The user retired the postponed eframe UI, its dedicated tests and validation jobs. The supported desktop remains the WebView UI with the shared Rust backend and native macOS/Windows window/transport code.

Removed native_ui_probe source/binary, optional Cargo feature, rendering/launch/profile/role/task/tool/cache/workspace/pulse/provenance tests and CI feature builds. No replacement Rust UI test suite is added. Existing build, backend, native shell, WebView behavior, license, package and source-layout checks remain.

Removing eframe prunes 230 package-version identities from Cargo.lock (508 -> 278) without adding or upgrading retained dependencies. Only unused clipboard-win/libloading provenance texts are removed; WebView2 licenses and attribution for the supported app remain. SBOM/notices are generated from the remaining resolved graph.

Historical migration decisions remain archived and are marked superseded, rather than rewritten as if no migration had occurred. Future requirements must not reactivate the old feature or CI instructions.
