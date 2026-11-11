# Native modal accessibility test frame

Verified: 2026-11-11. Status: focused Rust test passes locally; exact-head macOS and Windows CI is pending.

## Rule

When a native egui test inspects an `egui::Modal` through AccessKit, run the UI once to initialize the modal layer, then collect accessibility nodes from a second frame using the same `egui::Context`. Reading the first frame can omit the modal's labels even though the UI state requests confirmation. This is a test-harness requirement; it does not change the product UI.

## Evidence

- Issue #9 / PR #22, failed platform run `41647040730` (macOS and Windows)
- Regression: `tests/native_ui_candidate_decision_language.rs`
- Requirements context: `docs/07-implementation-plan.md`, Rust UI acceptance remains 22/50
- Local red: the focused test failed on the localized approval-confirmation label before the helper rendered a setup frame.
- Local green: focused localization test passes after adding a setup frame; `source_layout` passes after removing generated `tests/__pycache__`; rustfmt and diff checks pass.

| Source | SHA-256 |
|---|---|
| `tests/native_ui_candidate_decision_language.rs` | `162a1b1c0a970b3cb0b3b424b21552f84f815c76e0650e3a2ee59eea536cef98` |

This test fix adds no UI parity acceptance. Recheck the exact-head macOS and Windows run before marking the unit complete.
