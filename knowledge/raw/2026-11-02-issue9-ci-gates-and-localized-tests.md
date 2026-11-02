# Raw source: Rust issue #9 CI gate and localization test fixes

- Date: 2026-11-02
- Repository: `neodamul/AgentConductor`
- Rust branch: `codex/issue-9-native-ui-parity`
- Main commit integrated: `54a9148fbe25a6eaad4f64a39efafcaefc6bbebd`
- Rust merge head: `ab41ab0dfaf557e53987bbbc11f4a45fb7a948d9`
- Feature gate fix: `16771cc6`
- Failed previous validation: `41423326877`
- Current exact-head validation at capture time: `41426728992` (in progress)

## Observations

1. Default-feature macOS CI failed because Cargo auto-discovered `tests/native_ui_update_blocker_notice.rs`, which imports the optional `native-ui-probe` feature. The test lacked a crate-level feature gate. `tests/native_ui_targets.rs` correctly checks all discovered native UI integration tests for this gate.
2. The crate-level `#![cfg(feature = "native-ui-probe")]` restores the intended default build boundary. Both default-feature `native_ui_targets` tests and the opt-in UI test passed locally.
3. Main PR #334 localized coordinator refusal messages. The Rust port uses its native message catalogs and must keep the web-only `src/ui/messages/en.json` and `ko.json` absent; those files also exceed the 300-line source/test bound.
4. `native_workspace_chat_compatibility` assumed that the active UI language was English. On a Korean-configured machine the product correctly returned `읽기 전용`, so the assertion failed. The test now checks the coordinator identifier and accepts the English or Korean read-only wording. Catalog-level tests separately assert both translations and placeholders.

## Local verification

- `cargo test --locked --offline --test native_ui_targets`: 2 passed.
- `cargo test --locked --offline --features native-ui-probe --test native_ui_targets --test native_ui_update_blocker_notice`: 3 passed.
- `cargo test --locked --offline --features test-fixtures,native-ui-probe --test coordinator --test coordinator_chat_check --test native_update_blocker_messages`: 13 passed.
- `cargo test --locked --offline --test source_layout`: 2 passed after resolving the web-catalog deletion.
- `cargo clippy --locked --offline --all-targets --features test-fixtures,native-ui-probe -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- Initial full-suite run identified one language-assumption assertion; the focused test passed after allowing both supported locale wordings. The complete post-fix locked/offline all-target suite then passed.

## Architecture trace

The coordinator safety and refusal behavior follows R-206 → BG-161 → SF-194 → UC-198 → FR-199/FR-200/NFR-88/NFR-89 → QAS-204 → DDR-55 → DD-218. The native locale catalog is the UI boundary for translated refusal text; read-only argument validation remains in the Rust coordinator.
