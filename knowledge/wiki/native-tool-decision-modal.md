# Native tool-decision modal

Verified: 2026-11-11. Status: local Rust regressions pass; live desktop/package acceptance remains separate in GitHub #348.

## Sources

- GitHub issue [#382](https://github.com/neodamul/AgentConductor/issues/382)
- Requirements: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160
- Implementation: `src/native_ui_probe/room_actions.rs` and `src/native_ui_probe/room_actions/tool_dialog.rs`
- Regression: `tests/native_ui_tool_dialog.rs`; eligibility and payload regressions: `tests/native_ui_launch.rs`

| Local source | SHA-256 |
|---|---|
| `src/native_ui_probe/room_actions.rs` | `f45601775cea69b112bcf936e2649bd204112b17b456922bad878064ba37a815` |
| `src/native_ui_probe/room_actions/tool_dialog.rs` | `d64f10415ee722bc55d240790001882969bfa5e1b446c1e9558679e6d8039b38` |
| `tests/native_ui_tool_dialog.rs` | `899cfadb75b5dfa93b4e48d9785abbf7925952e3606ea62de009850a00d6561d` |

## Decision and evidence

Route every explicit Allow, Deny, or Refresh confirmation through the existing modal. A current confirmation first selects its exact receipt as the modal target. Pending receipts remain eligible for their supported actions; an approved receipt is selectable only for a current Refresh confirmation at the same run revision. Keep the existing pending/busy guards, evidence display, Cancel and Decide later behavior, and `ToolDecision(action, id)` event payload. Remove the inline confirmation renderer so a deferred or approved request cannot produce a second decision surface.

TDD reproduced the split renderer for a deferred receipt while another pending request existed, and for an approved receipt while a different pending request existed. Both now expose one confirmation action in the existing modal and select the exact receipt. Focused modal, eligibility, action-payload and source-layout checks pass (17 tests); warning-denied Clippy and formatting pass. The full native-feature suite passes 277 tests, with 2 ignored, across 67 suites. The touched source and test files are each below 300 lines.

## Limits and revalidation

These are deterministic egui/accessibility-tree and protocol tests, not a live authenticated tool call. Reverify the exact macOS/Windows native package journeys tracked by #348 before treating native tool-decision acceptance as complete. Revalidate after changes to receipt fields, tool actions, run revision semantics, or modal ownership.
