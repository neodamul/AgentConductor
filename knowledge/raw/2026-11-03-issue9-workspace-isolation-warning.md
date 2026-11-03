# Raw source: issue #9 Workspace isolation control and warning panel

- Retrieved: 2026-11-03
- Repository: `neodamul/AgentConductor`
- Rust branch: `codex/issue-9-native-ui-parity`
- Legacy reference revision: `origin/main` at the time of inspection
- Rust branch checkpoint: `10818b2b` plus the current follow-up commit
- Related implementation-plan section: `docs/07-implementation-plan.md`, “Match Workspace isolation and blocker presentation”

## Observations

1. The legacy `src/ui/startup.html` places the localized worktree checkbox in the project-folder row next to Browse. The Rust UI briefly moved it below goal/import controls based on an outdated reference; the latest available source and screenshot confirm that was incorrect. It is now back beside the project picker. An AccessKit bounds regression failed against the misplaced control and passes with Browse and checkbox centers aligned.
2. The legacy reference exposes coordinator incompatibility through a prominent warning notice. Rust renders a tinted, wrapping warning panel with the full localized refusal text.
3. Clicking the accessible isolation checkbox sets `WorkspaceMode::Worktree`; the generated workspace-save request serializes `workspace_mode: worktree`.
4. The current main screenshot asset is labeled v0.1.55. It is used only to verify the isolation-control placement, not for full-screen parity or acceptance.

## Local verification

- Focused UI targets: 16 passed across language, workspace, project-picker, and isolation-layout suites; source-layout: 2 passed.
- Warning-denied all-target Clippy, rustfmt, `git diff --check`, and the ≤300-line source/test guard passed.
- A fresh Rust capture uses a disposable saved workspace, three synthetic agents, and synthetic Claude/Codex quota observations. No provider CLI, credentials, or user repository was accessed.
- Exact package, full screenshot parity, platform CI, and user-visible end-to-end acceptance remain unverified; no #9 acceptance point is added.

## Scope and revision

This is a correction for issue #9. It retains the existing Rust save contract and optional worktree behavior. It does not change workflow or overall acceptance counts. See `docs/evidence/issue9-workspace-isolation-layout-20261103.json` for the superseded placement claim and correction evidence.
