# Native update blockers

Before starting an application update, the shared update guard checks whether workspace operations are active and whether readiness checks failed. Active work is not an update-check failure; preserve those as separate sections in the refusal result.

The Rust UI presents the refusal in its update dialog. Keep the localized English and Korean catalogs in sync, preserve elapsed time when available, and show the next action. The guard runs before the updater is launched; the UI must not add duplicate checks, background polling, or subprocesses.

Regression coverage:

- `tests/update_blockers.rs` verifies guard classification and refusal guidance.
- `tests/native_update_blocker_messages.rs` verifies English/Korean keys and message separation.
- `tests/native_ui_update_blocker_notice.rs` renders the actual native dialog and checks its multiline text.

Source observation and revision hashes: [2026-11-02 raw record](../raw/2026-11-02-native-update-blocker.md). The port was pushed as `a86ec103` in PR #22. Local verification passed; hosted exact-head macOS/Windows validation was queued. This is source-level evidence only and does not increase issue #9 packaged workflow acceptance.
