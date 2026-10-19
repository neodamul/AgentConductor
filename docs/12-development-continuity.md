# Development continuity

Use shared agentmemory project `AgentConductor` for Codex–Claude development handoffs.
Keep engineering records in English and brief the operator in Korean. Do not place
internal handoffs or implementation plans in the public README.

## Checkpoint procedure

Save a durable checkpoint after each verified work unit and before yielding for an
account limit. Record intermediate decisions when a unit is too large to finish.
Frequent checkpoints preserve continuity even if interruption occurs without warning.

Each handoff must contain:

1. Repository URL, local checkout path, branch, exact HEAD and remote HEAD.
2. Dirty files, their purpose and the location of any preserved patch or Git stash.
3. Requirement/use-case/design references and Plan → Task → Verify → Complete status.
4. Test command, platform, source revision, passed/failed/ignored counts and evidence.
5. CI run IDs with observed terminal status; never label an old running run current.
6. Remaining work in execution order, unresolved findings and the next concrete check.
7. Owned process/session IDs, cleanup status, pending approvals and sensitive-data limits.

The receiving agent recalls shared memory, checks actual Git history and CI, and
preserves local changes before updating the checkout. A different checkout path is
not evidence that work is missing. Do not blindly replay an older patch if the
remote history already includes it. Revalidate source fingerprints after integration.

## GitHub issue-linked branch procedure

Before editing an issue, run `gh issue develop --list <number>`. If the issue has no
linked branch, create one from `main` with `gh issue develop <number> --name
codex/issue-<number>-<work> --base main`, then fetch and switch the local worktree to
that exact branch. Verify both `git status --branch` and `gh issue develop --list`
before making changes; a matching branch name or a `Fixes #N` PR body alone does not
prove the issue has a linked branch. Push verified work to that branch and keep its
PR draft while acceptance is incomplete. After an approved merge to `main`, propagate
the merge to `rust` only with a verified fast-forward or a reviewable merge, then
confirm the remote branch heads. If an existing issue branch has diverged from the
newly linked branch, preserve its history and resolve conflicts explicitly rather
than discarding it or force-pushing.

After a pull request is opened from an issue-linked branch, GitHub removes the
branch from the issue's linked-branch list and shows the pull request in
**Development** instead. An empty `gh issue develop --list <number>` after PR
creation is expected; verify the branch before opening the PR, then verify the
issue-linked PR and its head branch. For issue #9, PR #22
(`codex/issue-9-native-ui-parity` → `main`) is the current development link.
See [GitHub's branch-for-an-issue behavior](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-a-branch-for-an-issue).

The #9 branch was linked on 2026-10-18 as
`codex/issue-9-native-ui-parity`; its existing Rust migration history was preserved
on that branch and is reviewed in draft PR #22.

## Reconciliation on 2026-10-10

Claude worked in `/Users/neodamul/Development/AgentConductor`; Codex worked in
`/Users/neodamul/Documents/ChatGPT/AgentConductor`. GitHub main reached `2211ba3`,
19 commits after `3eb669c`. The first new commit, `cff4646`, incorporated the pending
Codex continuation work. The older Codex changes were preserved in Git stash
`codex pre-Claude synchronization: continuation checkpoint preserved`; retain it
until the source and evidence reconciliation is complete.

CI run [38037224504](https://github.com/neodamul/AgentConductor/actions/runs/38037224504)
on `bbabefe` completed with macOS success and Windows test failure. Windows compilation,
the three Job Object tests and the WebView2 window smoke test passed. The failed gate
test reached successful single-consumption assertions, then attempted directory
cleanup while a reopened SQLite store was still alive (Windows error 32).

Remaining priorities at this historical checkpoint: verify the cleanup correction on native Windows; implement
DD-41 source/test separation and the original 500-line CI limit (superseded by the 300-line bound on 2026-10-16); finish DD-40 wiki and Windows
icon evidence; validate native Korean IME and modal accessibility; package verified
macOS/Windows binaries with checksums and license notices for the private Release.
The full product acceptance backlog remains authoritative in the implementation plan.

## Canonical checkout update

The operator confirmed the path change on 2026-10-10. Codex app project registration
now points to `/Users/neodamul/Development/AgentConductor`. Use that checkout for
future work. The previous Documents/ChatGPT checkout remains an untouched backup
of pending implementation and verification. The new checkout received the committed
handoff and DD-41 patch without replacing `.claude/`. Verification evidence was
copied with an explicit migration manifest; do not mislabel the pre-migration full
regression as a new-path rerun.


## Current handoff — 2026-10-16

Canonical checkout: `/Users/neodamul/Development/AgentConductor`; branch `codex/issue-9-accessible-workspace-menu`, local and remote HEAD `a92bbee`, in sync after four verified refactor units; based on current `origin/main` plus 32 issue-branch commits. Preserve the operator-owned `LICENSE` edit; it remains unstaged. Issue #9 is open. The 300-line requirement update is in progress: tests were changed first and correctly went red; 57 source/test files exceeded the new ceiling initially. Five responsibility-based splits now pass their focused regressions; native UI launcher split commit `1f4b155` and poller split commit `a92bbee` are pushed; the strict layout guard now reports 52 oversized files. Four source-refactor units (`3c2640b`, `599307b`, `1f4b155`, `a92bbee`) are pushed. Full green verification and the requirement/document commit remain pending. Open bugs take priority over enhancements, with the oldest actionable bug first; #7 remains blocked for missing sanitized CLI evidence.
