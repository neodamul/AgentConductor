# Step lessons

## Purpose

A coordinator revision, an unusable test-command replacement, or an approved scope change can teach the run something the later agents need. A step lesson stores the originating task, cause, response, and timestamp as evidence.

## Delivery

- `workflow::step_lessons` stores the newest 50 lessons in the run document and records each lesson in the existing memory outbox.
- Duplicate task/response pairs are ignored; each text field is clipped to 1 KiB.
- Worker prompts receive at most 4 KiB of lessons under an evidence-only heading.
- Native task detail snapshots include the lessons. Agent Room presents them under the selected task's Lessons view next to reviewed task-scoped knowledge.
- When agentmemory transport is preferred, outbox synchronization is best-effort. Local run state remains authoritative.

Lessons do not change task scope, tool permissions, or merge approval. The task contract and human approvals remain authoritative.

## Verification

The workflow tests cover de-duplication, field and collection bounds, prompt delivery, and memory outbox persistence. The native rendered regression covers source task, cause, and response visibility in the Agent Room. The full local suite passed with `cargo test --locked --all-targets --features native-ui-probe`; warning-denied all-target Clippy, rustfmt, public-doc, source-layout, and focused lesson tests also passed. All touched Rust source and test files remain under 300 lines.

Graphify was refreshed to 8,446 nodes and 14,536 edges. CocoIndex Code could not run because its launcher currently fails to import `mcp.server.fastmcp`.

Trace: R-167 → BG-123 → SF-155 → UC-158 → FR-153/NFR-63 → QAS-164 → DDR-27 → DD-178.
