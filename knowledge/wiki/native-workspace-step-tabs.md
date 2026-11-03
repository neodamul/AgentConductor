# Native Workspace step-tab viewport sizing

## Decision

The three Workspace stage tabs use equal widths bounded by the smaller of the layout's available width and the visible clipped viewport, with the content inset reserved before dividing the row. This prevents a long localized label or an expanded parent layout from pushing later tabs beyond a narrow window.

## Evidence

The existing regression in `tests/native_ui_header.rs::all_step_buttons_fit_the_content_column_in_both_languages` covers English and Korean at 480 px and 836 px. Before the fix, the 480 px English third tab extended to x=559. After commit `e2419db3`, all six header tests and source-layout pass, along with the locked/offline all-target suite.

## Limits

This verifies the Rust header component's measured bounds. It does not establish same-state screen parity, exact macOS/Windows packaged UI behavior, or any additional issue #9 acceptance credit.

## Links

- Implementation: `src/native_ui_probe/workspace/header.rs`
- Regression: `tests/native_ui_header.rs`
- Plan: `docs/07-implementation-plan.md`
- Audit: `docs/09-completion-audit.md`
- Evidence: `docs/evidence/issue9-native-workspace-step-tabs-20261103.json`
