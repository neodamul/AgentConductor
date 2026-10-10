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

### UC-15 — Prepare and start a goal

1. Open the native app and register CLI profiles without provider API credentials.
2. Select a repository, coordinator and goal; optionally select worktrees.
3. Query observed catalogs and request read-only planning.
4. Review/edit exactly three tasks, disjoint paths, models and test argv.
5. Approve the exact plan. Changed settings, plan or repository require a new review.
6. Open the resulting run in the same desktop review surface.

Trace: SF-18. Unknown outcomes retain evidence and require explicit recovery rather than automatic replay.

### UC-16 Inspect paid-agent capacity

The operator opens an agent summary and sees observed remaining quota and reset times, or an explicit unavailable/stale state. A bounded refresh uses existing CLI authentication. No reset credits are consumed and no billing notification is sent.

### UC-17 Continue on Windows and release

The developer clones the private repository, follows the checked plan and resumes TDD from committed sources. Credentials and workspace paths are configured on that host. Launching the executable without arguments opens the default workspace under HOME, or under USERPROFILE on Windows when HOME is absent or empty; an explicit ui-new path remains available. Native platform tests prove cancellation, recovery, independent sessions and approvals before release publication.

### UC-18 Continue through paid-account exhaustion

The operator registers the company CLI and selects it as Claude/Codex fallback while reviewing the plan. A paid worker reports a usage restriction. Rust records the account restriction and reset, drains owned jobs, retains existing files/evidence and starts the permitted fallback in a new session. If that target is unavailable, the task remains waiting. Tests and exact human merge approval follow. The operator may instead schedule or reassign manually. Trace: SF-21.

### UC-19 Ask what the team is doing now

The operator requests current status. The orchestrator returns the goal and overall state, then each agent/session's task, last observed activity/report, report age, restriction or waiting reason, evidence and next action. Approval requests are listed separately. Stale reports and unknown progress are visible. This read-only query never resumes, reassigns or approves work. Trace: SF-22.

### UC-20 Scan the agent list

The operator sees active blinking green lights, steady completed green lights, yellow waits and red failed/blocked sessions without opening individual logs. Selecting a row shows the actual state and evidence. Reduced-motion users receive a steady light with the same state label. Trace: SF-23.

### UC-21 Accept direction at an agentmemory checkpoint

Before starting work and at configured safe checkpoints, a company CLI writes its current task, phase, progress evidence and next step to agentmemory. It retrieves directives addressed to its logical/provider session, validates issuer/scope/revision/expiry, acknowledges a directive and cooperatively pauses, revises or relinquishes its current task. It records completion/rejection so Rust and other agents can observe the outcome. A transfer cannot start another writer until relinquishment and process quiescence are verified. After a completed handoff, the reviewed fallback agent continues the task in a new session with the partial work; without a reviewed target the task waits for the operator. Native stdin steering is not required. Trace: SF-24.

### UC-22 Decide a paused tool request

A worker asks to run a command. Its row shows it waiting on a tool decision while the process stays alive. The operator reviews the exact input and approves (the same attempt runs it once and continues) or rejects (that call is refused and the worker chooses another step). If the operator stops the run or does not answer within the configured window, the request remains pending with evidence and the task waits as before. Trace: SF-25.

### UC-23 Plan with a company CLI

The operator selects a company CLI as planning coordinator. The product creates a tracked-files snapshot of the reviewed base, runs planning there and shows the proposal. If the target workspace changed during planning, the proposal is refused. Trace: SF-26.

### UC-24 Work in Korean

The operator types a Korean goal, feedback and task instructions with the system input method while the window refreshes. Text appears exactly as composed in the plan, worker prompt and evidence. Trace: SF-27.

### UC-25 Leave the workspace without saving

The operator opens the workspace from the Agent Room, edits the goal, a profile or plan tasks, then decides not to keep them. They choose "Back to run" (or "Discard changes"). The application lists the unsaved areas and asks for confirmation. On confirmation the edits are dropped, the saved workspace is shown again and, for "Back to run", the Agent Room reopens; cancelling keeps the edits. With no edits, "Back to run" returns immediately. Trace: SF-28.

### UC-26 Recognize agents by their application icon

The operator opens the workspace or Agent Room. Claude Code, Codex and OpenCode profiles show the icon of the corresponding installed application; a custom company CLI shows its letter avatar. If an application is missing, its profiles also show the letter avatar. Trace: SF-29.

### UC-27 Get started from the README

A new operator opens the repository or release README, reads what the product does, sees the actual workspace and Agent Room, installs the package, launches the app and follows the walkthrough (register agents, enter a goal, review the plan, approve, review the result, merge). Questions beyond that are answered by the linked user manual. Trace: SF-30.
