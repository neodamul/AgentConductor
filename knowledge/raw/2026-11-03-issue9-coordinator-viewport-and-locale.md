# Raw source: issue #9 coordinator viewport and locale regression

- Retrieved: 2026-11-03
- Repository: `neodamul/AgentConductor`
- Rust branch: `codex/issue-9-native-ui-parity`
- PR: [#22](https://github.com/neodamul/AgentConductor/pull/22)
- Inspected PR head: `c5740c8005c02ca66456d80a47a6451cb265844c`
- Actions run: [41557402619](https://github.com/neodamul/AgentConductor/actions/runs/41557402619)
- Source revision under test: merged PR ref `3f9fdc211cc7072f5883483aa211f9beabcbb10b`

## Observations

1. A real egui accessibility-tree measurement at 1140×740 found the coordinator model selector ending at x=1217, beyond the visible viewport. The new test failed before the layout fix and passes after the workspace fields were split into responsive columns.
2. The macOS validation job failed three `workspace_model` library tests. Those tests searched for Korean-only labels (`조정자 모델`, `CLI 기본값`) and Korean-only warning text while the CI-rendered labels were English. The product’s selected locale was correct; the assertions were not locale-neutral.
3. The UI interaction tests now resolve selector/default labels from the active native message catalog and format the expected blocker with the active language and actual compatibility reason.
4. Actions run `41557402619` had macOS fail in native library tests; its Windows native tests were still running when inspected. The run used the old PR head and cannot verify the local fix.

## Local verification

- `cargo test --locked --offline --features native-ui-probe --lib`: 116 passed.
- Four native Workspace model tests: 4 passed.
- Native sidebar density/details tests: 5 passed.
- Feature-gated library Clippy with `-D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- This verifies the local macOS source only; current-head macOS/Windows CI and exact packaged UI acceptance remain pending.

## Scope and revision

This is a source observation captured from PR #22 and Actions run `41557402619`, then reproduced and corrected locally on the existing Rust issue #9 branch. The implementation-plan entry records the current TDD checkpoint. No end-to-end UI acceptance credit is added.
