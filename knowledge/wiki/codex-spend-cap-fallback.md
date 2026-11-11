# Codex workspace spend-cap fallback

Verified: 2026-11-11. Status: local Rust integration passes; exact-head PR validation remains pending.

## Behavior

Treat Codex's workspace spend-cap `turn.failed` event as an account availability restriction. Keep an already observed restriction when a later error event does not contain new limit evidence. This allows the existing durable availability timer to select only the reviewed fallback after its five-hour horizon. Do not infer a reset time from the spend-cap message.

## Architecture trace

R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. The source PR is GitHub #713 for bug #712; the Rust port is on the existing issue #9 root branch.

## Verification

- TDD red: the exact mock Codex `turn.failed` spend-cap event left the run `failed` with no restriction.
- Green: the run enters `waiting_availability`; after five hours the reviewed `backup` agent is selected and completes the task.
- Availability, timer, Claude availability workflow, and Codex fallback suites: 14 passed across four suites.
- `source_layout`: 2 passed. `cargo fmt --all -- --check` and `git diff --check` pass.
- All modified Rust source and test files are below 300 lines.

| Source | SHA-256 |
|---|---|
| `src/availability.rs` | `0485cacdf13476da2a08b73d9a9a7f8227c904623bdc7b677991ea5cd42d2e46` |
| `src/workflow/execution/worker.rs` | `7a7059819fbf2d90af9cc9f74e64814ca7e229b2f6e3364660c0e70018c758c7` |
| `tests/availability.rs` | `5d76a1163f69be245d27abfa13d54766c6f5e5ecbbfbc7b10505e938acce6b6d` |
| `tests/codex_spend_cap_workflow.rs` | `9bff40d35ba6f727d8ab4edd9e21500281faa27eabddf9295b8ca8c3cefb1ffe` |
| `tests/support/mock_agent.rs` | `b3e17d3bccc5e636f84535000b877478288f8afc16ca8b523c4d6a85034ceafe` |

## Limits

These deterministic local fixtures do not query a real enterprise spend cap. They verify event classification, persisted waiting state, fallback timing, and reassignment. Keep #712's main fix and this Rust port linked; do not count additional #9 UI acceptance for this backend behavior.
