# Cross-agent development continuity

## Evidence

### Exact-package native manual link check — 2026-10-23

On package source `6b9641a1f6e47c9601618747a2b842b6e1b96da9`, open the bundled manual and click its visible official configuration-reference link. It should dispatch the HTTPS URL to Safari. This confirms one component interaction only; it adds no workflow acceptance. Close the app without saving the displayed workspace and close only the browser tab created by the check. Startup can refresh quota metadata, and screenshots of the saved workspace must not be retained.

### Issue #9 audit-head synchronization — 2026-10-23

`tests/issue9_current_status.rs` pins the PR head observed before an audit synchronization. When that guard fails, check the live PR and branch first; update the recorded observed head and the audit checkpoint together, then run the focused docs/status tests and the complete offline native-feature suite. A documentation-only synchronization does not change packaged workflow acceptance.

### GitHub issue-linked branches — 2026-10-18

Create issue work branches through `gh issue develop <number> --name <branch>
--base main`, then verify the exact local branch and `gh issue develop --list`
before editing. A branch name containing the issue number, or a PR that mentions the
issue, is not sufficient evidence of a GitHub Development-linked branch. Push the
verified branch and keep its PR draft while acceptance is incomplete. After merging
to `main`, apply the merge to `rust` with a verified fast-forward when histories allow;
preserve and explicitly integrate existing work if the branch histories have diverged.

For issue #9, `codex/issue-9-native-ui-parity` was created from `main` through this
flow. Its 141 existing implementation commits were preserved; because `main` had
advanced, the histories were explicitly merged and one accessibility-test conflict
was resolved from the richer issue-9 test coverage. The branch is pushed and tracked
by draft PR [#22](https://github.com/neodamul/AgentConductor/pull/22). The migration
remains incomplete and issue #9 stays open.

GitHub main at `2211ba3` contains 19 commits after Codex checkpoint `3eb669c`.
Claude used a different checkout; the first continuation commit `cff4646` already
incorporated pending Codex source changes. Preserve local work before synchronizing;
replaying an old patch would duplicate or overwrite newer implementation.

The handoff's running Windows CI is now terminal. Run
[38037224504](https://github.com/neodamul/AgentConductor/actions/runs/38037224504)
on `bbabefe` passed macOS validation and Windows compilation, Job Object containment
and WebView2 smoke tests. The gate test's assertions passed, but cleanup failed
because a reopened SQLite store remained alive. Close the store before directory
removal; retries cannot compensate for an owned handle deliberately left open.

Focused macOS gate regression after correction: 4 passed, 0 failed, 0 ignored.
Native Windows verification remains pending; this is not a Windows completion claim.

## Sources

- [Gate regression](../../tests/gates.rs), SHA-256
  `24c62d5fa69a58496524eecb01adf09ada5e0fd1713b1263d7f256658d11880b`.
- Local CI evidence `.agentconductor/verification/claude-handoff/windows-ci.log`,
  SHA-256 `0b43ca55cdf50d700c9ba196a17ae966148c792210671b906dc71fda5c850712`.
- [Checkpoint procedure](../../docs/12-development-continuity.md).
- Shared agentmemory `mem_mv25cgtz_2a8a07bee3cc`.

Save handoff checkpoints throughout work, not only immediately before a known
usage limit. Exact revisions, separate platform evidence, dirty-file preservation,
live handle status and the next check are required for reliable continuity.
