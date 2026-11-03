# Raw source: issue #9 Workspace isolation control and warning panel

- Retrieved: 2026-11-03
- Repository: `neodamul/AgentConductor`
- Rust branch: `codex/issue-9-native-ui-parity`
- Legacy reference revision: `origin/main` at the time of inspection
- Rust branch checkpoint: `10818b2b` plus the current follow-up commit
- Related implementation-plan section: `docs/07-implementation-plan.md`, “Match Workspace isolation and blocker presentation”

## Observations

1. The legacy `src/ui/startup.html` places a checkbox labeled `startup.isolate_tasks_in_git_worktrees` in the project-folder row next to Browse. The Rust UI showed a separate “Execution isolation” heading with Shared folder and Git worktree radio buttons.
2. The legacy reference exposes coordinator incompatibility through a prominent warning notice. The Rust UI only rendered small warning-colored inline text.
3. A native interaction test failed before the checkbox change because the accessible isolation checkbox was absent. After the change, clicking it sets `WorkspaceMode::Worktree` and the generated workspace-save request serializes `workspace_mode: worktree`.
4. A rendered-shape test failed before the warning-panel change because no tinted warning panel existed. It now confirms a large, tinted visible container while the existing accessibility assertion continues to check the full localized refusal text.

## Local verification

- Workspace model UI tests: 6 passed.
- Sidebar density/details and source-layout tests: 7 passed.
- All 118 library tests passed.
- Warning-denied all-target native-feature Clippy, rustfmt, `git diff --check`, and the ≤300-line source/test guard passed.
- Graphify code graph refreshed to 13,234 nodes / 21,331 edges without clustering or an LLM provider.
- Exact package, full screenshot parity, platform CI, and user-visible end-to-end acceptance remain unverified; no #9 acceptance point is added.

## Scope and revision

This is an implementation observation for issue #9. It retains the existing Rust save contract and optional worktree behavior. It does not change workflow or overall acceptance counts.
