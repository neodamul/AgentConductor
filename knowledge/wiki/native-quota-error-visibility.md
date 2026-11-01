# Native quota error visibility

Quota refresh failures are stored with the agent's quota snapshot and already appear in the quota badge title. Hover-only diagnostics are easy to miss, so the native agent sidebar and agent settings list also render the saved reason as inline text when the quota record has a non-empty error. The Agent Room run status area lists stored quota errors as well. The full reason remains available to accessibility clients and on hover; records without an error keep the compact quota layout.

The UI displays the collector's error verbatim and does not infer or rewrite an Enterprise provider screen. English and Korean labels reuse the existing localized unavailable-quota text.

## Windows Enterprise report

The operator confirmed Claude Code 2.1.296 on Windows displays an aggregate Enterprise spend/limit line at the `>` prompt after `/usage`. The exact account amounts are intentionally not stored. The existing synthetic parser fixture covers that output shape, including a date-only reset. The app showed only `—`; the saved error was hidden in the badge tooltip, which this change now renders inline.

This report proves that the CLI's screen format is recognized by the parser fixture. It does not prove whether the Windows app collector timed out, received an alternate screen, or persisted a parse error. Do not ask the operator to repeat the CLI check; inspect the app's own `quota-query-*` capture if the visible inline reason still indicates a collector failure. When `HOME` is set, Windows stores data under `%HOME%\\.agentconductor\\desktop`, otherwise under `%USERPROFILE%\\.agentconductor\\desktop`.

## Trace and verification

- Requirement/design trace: R-202 → BG-157 → SF-190 → UC-193 → FR-194 → QAS-200 → DD-214. GitHub #295.
- `tests/native_ui_quota_error.rs` and `tests/native_room_quota_error.rs` verify inline diagnostic labels in all native surfaces and both languages.
- Focused quota/sidebar/profile/source-layout tests, warning-denied Clippy, rustfmt, diff hygiene, and a native UI release build passed locally.
