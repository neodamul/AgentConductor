# QAS: Quality Attribute Scenarios

These are targets, not measured results. Tests must provide verification evidence.

| ID / Attribute | Source and stimulus | Environment / Target | Response | Measure |
|---|---|---|---|---|
| QAS-01 Concurrency | User starts three ready tasks | Local scheduler | Selected workspace mode within limit | Overlapping mock intervals; processes ≤3 |
| QAS-02 Safety | Unapproved merge or changed candidate | Merge executor | Reject before execution | Zero protected-ref changes; audit recorded |
| QAS-03 Recovery | Supervisor killed | Running / SQLite and processes | Identify survivors, recover stale runs | No lost artifacts/duplicate runs; recovered display within 5s target |
| QAS-04 Stopping | User stop | Child and grandchild running | Interrupt, terminate, kill, reap | Tree gone within 15s target |
| QAS-05 Parser reliability | Split UTF-8, large stderr, unknown JSON | Process I/O | Preserve raw bytes, normalize, bound memory | No lost events/deadlock; diagnose malformed output |
| QAS-06 Change isolation | Out-of-scope edits | Workspace verifier | Block review | Detect new/deleted/renamed files and symlink escapes |
| QAS-07 Availability | agentmemory unavailable | Running / outbox | SQLite persistence, later sync | Work continues; no lost reports; suppress duplicate display |
| QAS-08 Resource efficiency | Three mocks stream for 10min | Specified host / Rust core | Disk spool, bounded queues | Core RSS <100MiB, idle CPU <1% targets; exclude CLI/model/sidecar |
| QAS-09 Availability | Quota/reset and coordinator restart | Persisted scheduler | Claim due work once; retain ambiguous cases | No early attempt; all attempt logs retained |
| QAS-10 Session ownership | Same provider session resumed elsewhere | Parallel coordinators | Refuse duplicate/live/unknown ownership | No second spawn; retain assignment and evidence |
| QAS-11 Model selection | Invalid model or changed approved model | Session controls | Reject unknown model; revoke prior approval | No invalid spawn or stale approved merge |
| QAS-12 Extensibility | Register custom CLI | Profile/parser | Common execution/review | argv+JSONL fixture without scheduler changes |
| QAS-13 Knowledge integrity | Changed source/conflicting lesson | Wiki/index | Mark stale/conflicting | No promotion without evidence; detect hash changes |
| QAS-14 Distribution | Dependency/bundle change | Release verification | Policy, notices, SBOM | Zero unreviewed licenses/assets for completion |

Traceability: UC-02→QAS-01/05/06/08/12; UC-03→QAS-03/04/07;
UC-04→QAS-02/06; UC-05→QAS-07/13; UC-01→QAS-14;
UC-09→QAS-09; UC-10→QAS-10; UC-11→QAS-11.

## Availability and session scenarios

- QAS-09: Given an exhausted quota and explicit reset, restart the coordinator before reset; no attempt starts early and exactly one new attempt is claimed when due. Unknown times remain waiting for manual scheduling. Preserve every attempt artifact.
- QAS-10: Given several tasks assigned to one profile, enforce its concurrency limit and never concurrently resume the same provider session. Reassignment after exit must not send a foreign provider session ID.
- QAS-11: Given an unknown model, reject before spawn. Given a valid model change after approval, merge must fail until new verification and human approval.

### QAS-16 — Startup consent and private configuration

Invalid/stale/oversized native requests must not change settings or start a child. Stored profile environment values must not appear in snapshots. Plan approval binds the actual stored profiles, tasks, workspace mode and repository base. Closing/reopening the UI must not silently rerun a planner or executor. Trace: UC-15 / R-21.

### QAS-17 Quota semantics

Given used percentage 27 in a reported window, the summary displays 73% remaining with that window's reset/source. Missing windows stay unavailable; context tokens and session dollar estimates never imply subscription balance. Future-dated or expired observations are not presented as current. Account metadata requests cause no generation or reset-credit consumption.

### QAS-18 Portable delivery

A clean Windows checkout can build the supported target without macOS absolute paths. Owned process descendants stop on timeout/cancel/crash; repository leases prevent concurrent writers; secret/alias boundaries remain enforced. Platform release smoke tests operate on the exact packaged binaries.

### QAS-19 Policy fallback without live steering

Given a restricted paid account and reviewed self-hosted fallback, no running attempt is steered or overlapped during transfer. A fresh target session receives bounded owned-path evidence; source quota remains blocked after transfer. Unknown targets/models, cycles and same-account fallbacks fail before spawn. A stopped run never launches a fallback. Failed tests never become approval. Ordinary errors or silence do not trigger quota fallback. Trace: UC-18.

### QAS-20 Trustworthy status briefing

Given concurrent workers, a stale agentmemory report and a terminated local process, a status query identifies the process termination and stale report rather than claiming ongoing work. Every statement of progress has a timestamp/source; unknown information is not guessed. The query changes no task revision, permissions or approval. Trace: UC-19.

### QAS-21 Visible and accessible status

Given simultaneous running, waiting and failed tasks, the list shows blinking green, steady yellow and steady red with matching text. Polling does not reset the active pulse on every refresh. Unknown states stay yellow. Reduced-motion preference disables animation without losing state information. Trace: UC-20.

### QAS-22 Cooperative instruction acknowledgment

Given a non-steerable worker at a checkpoint and a valid addressed directive, it reports acceptance and applies the permitted change exactly once. Duplicate, expired, foreign-session or unauthorized directives are rejected with retained evidence. Memory-server unavailability does not fabricate acknowledgment or authorize a competing writer. An ordinary memory observation is never executable direction. Trace: UC-21.

### QAS-23 Live decision without regeneration

Given a waiting worker, approving its exact request from the CLI or UI while the run is active completes the task in attempt 1 with the command executed once and the approval consumed; rejecting leaves the command unexecuted and the task continues. Parallel owned-scope writes between decision and consumption do not invalidate the decision; task, session, plan, epoch or workspace-path changes do. Stop or timeout never executes the command. Trace: UC-22.

### QAS-24 Planning cannot touch the repository

Given a coordinator that attempts to write or delete files, the target repository and its untracked files are unchanged after planning, the snapshot contains no untracked or ignored files, and the snapshot is removed. Trace: UC-23.

### QAS-25 Korean composition

Given 1-second polling, composing and committing Korean syllables in the goal/feedback fields loses or duplicates no characters, and the saved goal and worker prompt contain the identical UTF-8 text. Trace: UC-24.

### QAS-26 Side-effect-free exit from the workspace

Given unsaved goal, profile and plan edits, "Back to run" asks for confirmation naming the unsaved areas; confirming returns to the Agent Room with the saved workspace unchanged on disk and no save, plan or start request issued; cancelling keeps every edit. With no edits the return is immediate. Trace: UC-25.

### QAS-27 Icons without redistribution or unsafe content

Given installed Claude, Codex (ChatGPT) and OpenCode applications, built-in profiles render their application icons within one second of opening; a custom profile renders letters. With an application removed, its profiles render letters and nothing fails. The release binary contains no provider artwork, icons are bounded (≤ 64 × 64 px, ≤ 64 KiB each) PNG data URLs permitted by the existing `img-src data:` policy, and no network request is made. Trace: UC-26.
