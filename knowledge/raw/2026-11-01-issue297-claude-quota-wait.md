---
source: https://github.com/neodamul/AgentConductor/issues/297 and Claude fix commit aa394f8e
retrieved_at: 2026-11-01T15:00:00+09:00
revision: aa394f8e
hash: git commit aa394f8e
---

# Source observation: slow Claude quota query

The issue reports that a managed Windows machine may take longer than the prior 20-second collector limit. The CLI's `/usage` command itself returns substantial quota details when it completes. The fix on Claude's branch increased the wait to 60 seconds and added elapsed query time to the JavaScript UI.

Porting notes for the Rust UI branch:

- Keep existing Claude CLI authentication and interactive `/usage`; do not introduce an API key or generation request.
- Apply one bounded 60-second deadline to startup, trust handling, prompt detection, and `/usage` response on macOS and Windows.
- Include both the deadline and whether timeout occurred before or after `/usage` in the error.
- Inject a short test deadline and use a local fake CLI so tests do not wait one minute or contact a provider.
- Render elapsed time only on the card for the active quota job; schedule local UI repaint without backend snapshot polling.

Unknown: how long the reported Windows company machine normally needs. A real packaged Windows check remains necessary to establish that 60 seconds covers that machine.

## Windows-only compile follow-up

Platform run `41309135388` reached the Windows build step and failed. The run logs were not available while macOS validation remained active. Source inspection found one outer-deadline branch still calling the old one-argument `quota_timeout(sent)` helper after the helper signature gained the configured wait duration. A new `claude_wait` source regression failed before the correction; both Windows timeout branches now call `quota_timeout(sent, wait)`. Local verification after the fix: 36 focused quota/UI/driver/docs/schedule tests, warning-denied Clippy, rustfmt, and diff hygiene pass. Cross-platform confirmation remains pending.
