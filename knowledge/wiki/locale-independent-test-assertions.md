# Locale-independent test assertions

Tests for user-facing validation errors must not assume the machine's saved UI language. The app loads its language preference from the user profile and defaults to English when no preference exists, as in CI.

For behavior that is identical across translations, assert the stable condition where available. If the message itself is the behavior under test, accept the supported localized variants and include the actual message in the assertion failure. Run the regression under both a clean English-default home and the configured local language.

## Evidence

- GitHub Actions run `39562614295` failed on macOS and Windows because `tests/startup_chat.rs` expected only `코디네이터` while CI returned the English `coordinator` message.
- Red reproduced the failure with a clean temporary HOME.
- The test now accepts `코디네이터` or `coordinator`; no production code changed.
- The focused test passes under clean English-default and existing Korean-preference environments. Full local tests, formatting, and warning-denied Clippy pass; exact-head Actions revalidation is pending.
