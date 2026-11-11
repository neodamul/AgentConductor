# Cross-platform UI test contracts

Rust source tests that inspect bundled UI styles must follow the actual assembly in `src/ui/setup/document.rs`. The startup document combines `startup.css`, `room_layout.css`, and `plan_review.css`; layout rules can move between these files without changing the rendered contract. A source assertion should search the assembled sources or the file that owns the rule, not assume the rule remains in `startup.css`.

Browser fixtures that inspect a scroll position should set scrolling explicitly to `instant`, wait for layout to settle, and include viewport width, `scrollTop`, scroll and client heights, element bounds, and footer position in a failing assertion. This distinguishes incomplete smooth scrolling from a genuine layout overflow and makes Windows-only CI failures actionable when local macOS rendering passes.

Evidence from PR #757, 2026-11-11: the first post-rebase full suite found two stale CSS source assertions; exact-head Windows CI also found the workspace tool row below the footer after the fixture scrolled. Updating the source contracts and adding deterministic scroll plus geometry diagnostics produced 6 passing focused layout/settings/scroll tests and a full local result of 922 passed, 16 ignored across 299 suites. The refreshed Windows and macOS CI still provides the final cross-platform check.

Relevant tests: `tests/step_card_css.rs`, `tests/workspace_settings.rs`, and `tests/workspace_scroll_741.cjs`.
