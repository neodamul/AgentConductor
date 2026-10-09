# CLI orchestrator OSS comparison

Checked: 2026-10-09 KST using READMEs, adapters, default-branch commits and GitHub API.
No installed performance or full permission-model E2E verification. Activity dates are latest
default-branch commit dates in UTC, not pushed_at or crawl dates.

| Project | License / Latest commit | Architecture, UI, isolation | Authentication and I/O | Collection, feedback, gaps |
|---|---|---|---|---|
| [conduct](https://github.com/ldlac/conduct) | MIT / 2026-06-21 | TS/Node, Ink TUI, per-task worktrees/processes | No additional key; Claude bidirectional stream-json; default Codex exec/OpenCode run | Diffs, manual merge, state files, conversations. Default Codex/OpenCode output is not JSON; DAG, automatic decomposition, custom registry, common approval need work |
| [ao](https://github.com/Mats2208/ao) | **Unconfirmed**: API license=null; no LICENSE in inspected tree / 2026-07-08 | Node JS brain/router/workers, worktrees, local web/PTY | Claude stdin/stream-json; Codex exec JSONL; OpenCode run JSON; existing CLI auth | Planning, routing, sessions, review/test/merge. OpenCode --auto; read-only not enforced. Exclude code reuse until license confirmed |
| [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) | Apache-2.0 / 2026-09-19 | Rust backend/React web, worktrees, parallel executors | CLI children; no separately required completion key by default. Claude stream-json, Codex app-server JSON-RPC, OpenCode serve/local HTTP | Rich board, diff, follow-up, approval. Different exec/run stdout contract; larger core. Custom CLI, automatic three-task DAG, wiki need assessment |
| [Agetor](https://github.com/alamops/agetor) | MIT / 2026-10-08 | Bun/Electrobun/React WebView, SQLite/worktrees | Existing harness auth; Claude/Codex tmux and Codex app-server paths | Board, conversation, pipelines. OpenCode absent from checked README; differs from minimal Rust/headless JSONL scope |
| [agent-deck](https://github.com/asheshgoplani/agent-deck) | MIT / 2026-10-04 | Go TUI, tmux sessions/worktrees | Already authenticated CLIs; terminal session control | Useful immediate CLI management. Standard JSONL results, automatic DAG, evidence-bound approval require further verification/extensions |

Adapter sources:
[conduct](https://github.com/ldlac/conduct/blob/main/src/core/agents.ts),
[ao Claude](https://github.com/Mats2208/ao/blob/main/src/providers/claude.js),
[ao Codex](https://github.com/Mats2208/ao/blob/main/src/providers/codex.js),
[ao OpenCode](https://github.com/Mats2208/ao/blob/main/src/providers/opencode.js),
[Vibe Codex](https://github.com/BloopAI/vibe-kanban/blob/main/crates/executors/src/executors/codex.rs),
[Vibe OpenCode](https://github.com/BloopAI/vibe-kanban/blob/main/crates/executors/src/executors/opencode.rs).

## Recommendation

Evaluate **Vibe Kanban** for immediate task/review UI; **conduct** for a lighter terminal workflow.
conduct can start with pnpm start /path/to/repo, create separate tasks, and manually merge
reviewed diffs. Neither is guaranteed to satisfy automatic three-role goal decomposition.

The requested combination is Rust, direct exec/run JSON I/O, custom CLI, pre-execution
approval, and wiki/TDD traceability. The inspected candidates are insufficient unchanged;
recommend a small Rust core beginning with CLI/pager. Use permitted Vibe Kanban code as a
process/logging reference; do not copy before checking contracts and transitive dependencies.
ao is an idea-comparison reference only. This is not an exhaustive claim that no suitable OSS exists.
