# Native coordinator chat: uncertain start feedback

When a submitted chat request reaches the controller but its response cannot be confirmed, `State::outcome_unknown` stores the diagnostic in `uncertain_error` and keeps the chat turn pending so a later snapshot can recover the result. The view must expose this diagnostic while the request remains pending. Hiding it leaves the operator with a spinner and causes Windows package tests to mistake an explicit Job Object refusal for a missing chat job.

`State::visible_error()` returns the uncertain diagnostic first and the ordinary error otherwise. `workspace/chat/view.rs` renders it in the chat transcript. The UI-driven regression is `tests/native_workspace_chat_uncertainty.rs`.

## Trace and verification

- Bug context: #189 and exact package run `41380213230` on #22.
- Fix commit: `5a318acf`; current branch includes main integration commit `00b39c5a`.
- Focused chat suites: 8 tests pass; warning-denied Clippy and rustfmt pass; relevant source/test files stay below 300 lines.
- Exact packaged Windows confirmation is pending run `41385932410`; this fix does not count toward packaged workflow parity.
