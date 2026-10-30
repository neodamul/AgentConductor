# Git HEAD fast path for native revisions

## Decision

Routine native state-revision and snapshot paths must not invoke `git rev-parse`. Common repository layouts can answer HEAD from `.git/HEAD`, a validated loose ref, or `packed-refs`. Linked worktrees and layouts the files cannot answer use a cached or pending value while a single background worker asks Git; concurrent refreshes coalesce and stale fallback results retry after five seconds.

`git_head::Heads` belongs to the native setup state and supplies the revision digest. `git_head::head` remains available only to code that may wait. Mutation ordering and approval/revision guards remain at the existing state owner.

## Verification

- `tests/snapshot_without_git.rs` hides Git from PATH after a proposal exists and verifies that a native snapshot still succeeds with a stable revision.
- `tests/git_head.rs` verifies loose, packed and detached refs, new commits, ref-name validation, and asynchronous linked-worktree fallback.
- The complete locked/offline all-target suite passed on branch head `6bbb4254683cb2d147696d98384643d4dea7fc6e`: 1,143 passed and 15 ignored across 364 suites. This supersedes the earlier interrupted full-suite attempt; it does not replace exact packaged macOS/Windows UI acceptance.

## Limitation

The linked-worktree fallback can be stale until its background refresh completes. A future action requiring exact current HEAD must add a stronger freshness check rather than treating a pending/cached digest as proof.

Trace: R-193 → BG-148 → SF-181 → UC-184 → FR-185/NFR-80 → QAS-191 → DDR-45 → DD-205.
