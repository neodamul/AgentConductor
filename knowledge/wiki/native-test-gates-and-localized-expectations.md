# Native test gates and localized assertions

## Cargo feature gates

Integration tests that import `native_ui_probe` or `eframe` must be gated at the test crate boundary:

```rust
#![cfg(feature = "native-ui-probe")]
```

The Cargo manifest explicitly gates some native targets, but Cargo also auto-discovers Rust test files. A newly added UI integration test can therefore compile during a default-feature CI run unless it has the crate-level gate. `tests/native_ui_targets.rs` scans discovered test files and verifies that contract.

Verify both paths when adding a native UI test:

```sh
cargo test --locked --offline --test native_ui_targets
cargo test --locked --offline --features native-ui-probe --test native_ui_targets --test <native-ui-test>
```

## Locale-aware tests

The application selects localized strings from the native message catalog. Tests that verify catalog coverage should inspect English and Korean catalog entries directly, including matching placeholders. Tests that exercise a runtime helper without an explicit locale should not hard-code English text. Assert stable identifiers plus accepted localized wording, or provide an explicit locale to the production interface when it supports one.

Keep behavior assertions stronger than “some localized text exists”: retain the profile identifier, refusal reason, and disabled action checks. Never bypass read-only validation to make a localization test pass.

For pointer-driven UI tests, resolve the accessibility label through the same active message catalog used by the widget. Fixed Korean or English labels can pass on one workstation and fail on CI even when the UI is correct. For refusal notices, format the expected text from the active locale and the real refusal reason. Keep separate coverage for the viewport bounds of controls: a field can remain in the accessibility tree while extending beyond the visible window.

Workspace isolation should be tested through the same checkbox used by the reference UI, then verified in the serialized save request as `workspace_mode = worktree`. Coordinator blockers should remain accessible while receiving a visibly distinct, wrapping warning container; a present accessibility label alone does not prove visible emphasis.

## Traced use case

Coordinator read-only refusal flow: R-206 → BG-161 → SF-194 → UC-198 → FR-199/FR-200/NFR-88/NFR-89 → QAS-204 → DDR-55 → DD-218.

## Evidence

The 2026-11-02 gate and localization correction is recorded in [the raw source note](../raw/2026-11-02-issue9-ci-gates-and-localized-tests.md) and [the implementation plan](../../docs/07-implementation-plan.md).

The 2026-11-03 coordinator-field overflow and English-runner regression are recorded in [the raw source note](../raw/2026-11-03-issue9-coordinator-viewport-and-locale.md) and [the implementation plan](../../docs/07-implementation-plan.md).

The 2026-11-03 isolation-checkbox and warning-panel rendering change is recorded in [the raw source note](../raw/2026-11-03-issue9-workspace-isolation-warning.md) and [the implementation plan](../../docs/07-implementation-plan.md).
