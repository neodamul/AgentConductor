# Use cases

## UC-01 Register and prepare

Actor: user. Features: SF-01/06/09. Preconditions: Git repository and installed,
authenticated CLIs. Register command/argv, inspect version/help, verify permissions,
then mark prepared. Missing executable/authentication blocks preparation; unknown
flags/parsers fail before execution. Never collect or log authentication file contents.

## UC-02 Plan and execute a goal

Actors: user/coordinator. Features: SF-02–04. Retrieve wiki/lessons, generate a read-only
coordinator plan, validate the three-task schema/DAG/scopes, review the plan, create
optional worktrees (shared workspace by default), start up to three ready tasks, and collect JSON events/logs.
Reject cycles, overlapping ownership, and escaping paths. Failed-task dependents are blocked.

## UC-03 Stop, report, and revise

Actors: user/coordinator. Features: SF-04/06/07. Show progress/phase, receive stop,
confirm process-tree termination, preserve stopped artifacts, inspect diff/failure evidence,
and resume the exact session in a new attempt. Persist reports in SQLite before outbox
delivery to agentmemory. Silence alone does not fail a task.

## UC-04 Verify and merge with human approval

Actors: coordinator/user. Features: SF-03–06. Validate ownership, create candidate commits,
in worktree mode, apply them sequentially to an integration worktree; in shared mode,
freeze the combined change set after all agents stop. Run integration tests, show the exact
diff/evidence, then enter waiting_approval. Accept approve/reject/revise; recheck
base/candidate/test hashes before updating the protected branch. Conflicts return to the
responsible agent. Changes invalidate approval. Candidate commits are separate from
final-branch merge; no automatic push.

## UC-05 Accumulate knowledge assets

Actors: coordinator/user. Feature: SF-07. Collect source metadata/hashes, generate wiki
drafts, mark evidence/conflicting claims, update index/log, synchronize reviewed pages
and lessons to recall storage. Failures retry through the outbox; old sources become stale
until revalidated. Workers write separate drafts; one writer promotes canonical pages.

## UC-09 — Resume restricted work

The provider reports a restriction. The coordinator retains logs, session ID, partial changes, and the exact reset time. If unspecified, the person schedules an explicit timestamp. A running watch coordinator wakes when due, or `resume-due` processes overdue work after restart. Repeated restriction is bounded; authentication/configuration failures require intervention.

## UC-10 — Transfer a role

After the old process has exited, the person reassigns a task to another registered agent. A handoff links owned-path changes, prior session, and evidence. The target opens a new provider session. Dependents and integration tests are re-evaluated; prior integration approval is revoked.

## UC-11 — Choose a session model

At registration, query the CLI model catalog and retain provenance. Display catalog models and query age. The person selects an observed model for a new task or changes an idle session model. Running processes cannot be edited. Re-run changed work and review the new candidate before merge.
