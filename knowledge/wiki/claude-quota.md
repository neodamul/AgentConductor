# Claude subscription quota

The retained-only refresh behavior below is historical; macOS now also has a verified terminal metadata probe documented in [Workspace quota collection](workspace-quota-refresh.md).

## Purpose and evidence

R-60 → BG-22 → SF-50 → UC-49 → FR-45/NFR-20 → QAS-51 → DD-66.
AgentConductor collects Claude quota from stdout of its owned tasks using existing CLI
authentication. It introduces no dependency, API key, credential reader or extra inference.

The official [SDK types](https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/types.py)
define utilization as a fraction. The official [wire parser](https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/_internal/message_parser.py)
reads `rate_limit_event.rate_limit_info`, camelCase `rateLimitType` and `resetsAt`.
[Interactive commands](https://code.claude.com/docs/en/commands) document `/usage`;
an independent headless account-quota RPC has not been verified.

## Behavior

The progress reader binds metadata to the reviewed agent profile and persists it before
process completion. Valid utilization 0.27 becomes 73% remaining. Five-hour, weekly,
weekly Opus, weekly Sonnet and overage windows retain the reported reset time. The board
labels model-specific windows explicitly. A new event replaces the prior snapshot;
omitted windows never gain freshness from a different window's event.

Missing utilization is unknown even when status says allowed or rejected. Malformed
metadata is audited without aborting otherwise valid task output. Existing five-minute
freshness and expired-reset rules suppress stale percentages. Profile changes invalidate
retained quota. Claude **Check observed quota** reads retained evidence and audits the
check without launching a process, restamping observation time, or changing execution
approval evidence. Codex retains its independent authenticated metadata query.

## Verification and limits

- TDD Red: missing parser and bound observation entry point failed compilation.
- Six Claude quota regressions cover conversion, missing/invalid utilization, decoder
  classification, live persistence before worker release, profile invalidation,
  metadata-only refresh, and subprocess → worktree workflow → native board integration.
- Focused quota/progress/layout/UI/native-room checks: 21 passed.
- Complete local all-target regression: 268 passed, 11 ignored, 76 suites; this run
  preceded the final additional subprocess case, which passed in the focused rerun.
- Formatting, Clippy with warnings denied, JavaScript syntax check and optimized locked
  macOS build passed.
- No live subscription event was available in retained verification JSONL. Tests use
  official-schema fixtures; actual Enterprise utilization emission is unverified.
- No new Windows runtime evidence: GitHub runners remain blocked by account billing.
- Graphify structural update: 2369 nodes, 4476 edges, 159 communities. CocoIndex
  structural search found the observer and both workflow call sites, no failed languages.
  Ponytail is unavailable in this tool session.

## Source fingerprints

- `src/quota/claude.rs` SHA-256 `87455b64d80be5216233a9947a71ff55b6b79e38a0227d0b605ba65117298ff9`
- `src/progress.rs` SHA-256 `2c9cf9107367b3880ad8afe0f754d9e6fbf24ca4007a9a9c816c477e7d4ddb37`
- `src/protocol.rs` SHA-256 `deb521fe14a6f2378cd9dc98af04e2755629e0549310bfc29e0234dea50d3028`
- `src/workflow.rs` SHA-256 `fb514621bcddbc51a2e87869fc890fc2f5ebd7627d181d4eae2bbf535ac647ba`
- `src/workflow/execution.rs` SHA-256 `e6723dd74f4825c4bb59271a7cb55ae8be888114b400740660c1ad026f4b9bdf`
- `src/ui/room.js` SHA-256 `6f45a070079f6cc06f982661be7145ede102327f21a95139553b773ce2439604`
- `tests/claude_quota.rs` SHA-256 `7f3a9e8ba0b08ca71ed8033bdcadf5cc9ae9c763e67579d3d3a6a39aeb5bb9a2`
