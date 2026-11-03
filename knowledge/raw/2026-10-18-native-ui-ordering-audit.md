# Native UI ordering audit — 2026-10-18

Scope: issue #9; R-102 → BG-57 → SF-89 → UC-92 → FR-87 → QAS-98 → DD-110, with R-50/DD-58 proposal preservation.

## Red evidence

- `cargo test --offline --locked --test native_ui_agent_order` failed with `missing reorder action claude: Move up` before the native ordering controls existed.
- The reviewed-plan regression in `tests/model_catalogs.rs` failed with proposal phase `Null` instead of `ready` after only reversing the profile array.

## Green evidence

- Four ordering tests cover real egui pointer drag/arrow events, English/Korean disabled boundaries within a 210px rail, edit/pending locks, stored order after reopen, retained coordinator/role/fallback/private environment, invalid moves, and stale-revision refusal.
- The execution view sorts a clone by stable agent ID before comparison. The operator's stored array order is untouched. A reviewed proposal keeps its digest through reorder/reopen; changing argv still invalidates it.
- Six affected persistence/consent targets passed 24 tests after the comparison change: `model_catalogs`, `native_ui_agent_order`, `profile_persistence`, `workspace_share`, `startup_run_binding`, and `native_ui_launch`.
- The default all-target baseline before that comparison change passed 611 tests, with 0 failures and 14 ignored tests across 214 targets.
- Eight feature-gated local fixture suites plus the current `source_layout` target passed 21 tests with no failures/ignored tests: cooperative worker/handoff, MCP imports/bindings/large sources/workspace/automatic sources, and Claude error-protocol fixtures. These use local Rust child processes, not authenticated cloud calls.

## Native-window follow-up

The optimized `12d130d` executable built successfully. Real macOS inspection exposed three visual defects that the widget tests had missed: tab frames extended beyond the content column, input borders had the correct color but zero width, and the edit glyph was missing from the platform font. New regressions reproduced the first two and rejected glyph-dependent icons. The fixes account for frame margins/gaps and text wrapping, set a real 1px inactive stroke, and paint edit/remove icons with vector strokes.

The initial drag label passed synthetic pointer tests but did not move cards in the real window. After replacing it with egui's dedicated drag source, a real drag moved gamma from third to first; reading `startup.json` and restarting the app confirmed gamma/alpha/beta with coordinator beta retained. This follow-up ran against the current debug binary in a disposable HOME/workspace; no provider CLI was invoked. Twenty focused UI/layout tests and all-target/all-feature Clippy pass. Test coverage must include actual rendered stroke widths and container bounds, not only palette values or accessible-name presence.

The optimized follow-up binary also builds successfully (17 MiB); `--help` exits 0. SHA-256: `356a982ad3ed19bb075d642206d2862caa0e14b4077db8b28352e32e253eb131`. The disposable app process, bundle, and workspace were removed after validation. This is a local build, not a published release.

## Remaining acceptance

These results do not establish equivalence with the shipped WebView app. Registration's immediate-save flow, complete native-window interactions, current Rust screenshots, IME/accessibility/resource checks, and exact macOS/Windows package acceptance remain open. Do not publish or close issue #9 on this evidence alone.

Graphify refreshed the local code graph to 6,247 nodes and 10,839 edges without an LLM call. CocoIndex could not start because the installed environment lacks `mcp.server.fastmcp`.
