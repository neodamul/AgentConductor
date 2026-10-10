# AgentConductor User Manual

This manual is the complete reference. The [README](../README.md) is the short
introduction. Most people only need the desktop application (Part 1). The command line
(Part 2) is for automation, scripting and advanced recovery.

## Part 1 — Desktop application

### 1.1 Requirements

- macOS 11 or later (Apple silicon) or Windows (x86_64).
- Windows only: the Microsoft Edge WebView2 Runtime (preinstalled on current Windows).
- Git.
- At least one coding agent CLI installed **and already signed in**: Claude Code
  (`claude`), Codex (`codex`), OpenCode (`opencode`), or a company/custom CLI.
  AgentConductor uses that existing login. It never asks for an API key.
- A Git repository with a clean working tree to work on.
- Optional: a local agentmemory server, used
  only when you explicitly sync or import memories. Everything else works offline.

### 1.2 Install and launch

1. Download the archive for your platform and `SHA256SUMS` from GitHub Releases.
2. Verify the checksum, then extract the archive.
3. Start `agentconductor` (macOS) or `agentconductor.exe` (Windows) without arguments.
   The desktop workspace opens. It is stored in `~/.agentconductor/desktop`.

Each package includes license notices, an SBOM and the binary's own checksum.
The window talks to the built-in Rust engine directly through native messages. There is
no local web server, port, browser session, Node runtime or cloud service. SQLite data,
logs and artifacts stay on your machine.

### 1.3 Register agents

The left column lists registered agents. Use **Register agent** to choose a provider template
(Claude, Codex, OpenCode) or **Custom** for any other CLI. Each agent has:

- **Executable and arguments** — prefilled from the template; review before saving.
- **Permission mode** — Codex uses its native workspace sandbox. Claude can use the
  guarded `native_callback` mode (every tool request is shown to you). OpenCode and custom
  CLIs need an operator-verified external sandbox or a read-only configuration;
  unrestricted profiles are rejected.
- **Sessions** — up to three concurrent sessions per agent (`max_sessions`).
- **Quota group** — agents that share one subscription/account share a group, so a usage
  limit pauses all of them together. The default group is the executable.
- **Fallback** — optional agent that takes over when this account is limited (§1.8).
- **Models** — **Models** queries the CLI's catalog using its existing login without
  sending a generation prompt. Available models depend on your account.

Provider agents show the installed application's icon; custom agents show a letter.

### 1.4 Plan: describe the goal

Pick the repository, type the goal (Korean and other IME input is supported), choose a
**Planning coordinator** and optionally **Isolate tasks in Git worktrees**
(shared directory is the default). **Generate plan** asks the coordinator to read a
temporary snapshot of the repository and propose exactly three scoped tasks. The
coordinator runs read-only on a copy, so it cannot change your repository. **Save
workspace** keeps your inputs; **Discard changes** leaves without saving.

### 1.5 Review and approve the plan

Each proposed task shows its agent, prompt, owned paths, dependencies and test command.
Edit anything — including which agent does which task — then **Save plan edits** and
**Approve plan & run**. Execution starts
only after your approval, which is bound to the exact plan digest. Owned paths must not
overlap. A task waits until the tasks it depends on are completed.

### 1.6 Task: watch the Agent Room

![Agent Room with completed tasks and captured evidence](assets/ui-agent-room-native.jpg)

The workflow strip shows **Plan → Tasks → Verify → Complete**. Sessions on the left use
lights: green for running/completed, yellow for waiting/stopped, red for failed/blocked;
running lights pulse (steady with reduced motion). Text labels keep the exact state.

![Session lights for waiting, failed and running sessions](assets/ui-status-lights-native.jpg)

Select a task to see **Activity**, **Diff & tests**, **Raw logs** and **Lessons**.
From the task inspector you can:

- send targeted feedback (**Send feedback**, ⌘/Ctrl + Enter) — previous evidence is
  retained and dependent tasks are re-verified;
- reassign the task to another agent, or choose another observed model;
- set a **Resume at** time for a limited session;
- **Stop run**, **Resume** and **Recover**.

**Brief team** shows a read-only snapshot of every session: assigned work, latest
activity, report freshness, process presence, restrictions and next actions.
Closing the window does not stop a running coordinator; use **Stop run**.

### 1.7 Tool requests

With a guarded Claude worker, each tool call (for example a shell command or file edit)
appears under **Tool requests** with its exact input and context. **Review & allow** or
**Deny** while the worker waits (default 600 seconds, `AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS`;
`0` restores stop-and-resume). Approvals bind the task, session, plan, workspace and exact
input, expire within 900 seconds and are consumed once. Tool approval never approves a
merge.

### 1.8 Usage limits, roles and redistribution

When a CLI reports a temporary usage limit (hourly, weekly or any other window), the
engine records the reset time the provider reported and marks every waiting session in
the same quota group as limited. Then:

1. **Fallback configured** — work moves to the fallback agent, which must use a different
   quota group (for example a company CLI). Active attempts exit first; scoped partial
   files and logs are kept; the fallback starts a new session with a handoff. Cycles and
   unobserved models are rejected. If the fallback is also limited, the work waits.
2. **No fallback, known reset time** — the session is scheduled for that time and resumes
   when due (**Resume**, or `agentconductor watch` while it runs). Up to three automatic
   retries; a manual schedule renews them.
3. **Unknown reset time** — the session waits for you to set **Resume at**, reassign it
   or change its model.

Authentication and configuration errors are never retried automatically. Ordinary
failures and your own stops do not trigger fallback. Verification and your merge approval
are always required again after a transfer.

**Account capacity** shows remaining percentage, reset time, source and freshness for an
account. Codex quota can be refreshed through its authenticated CLI. Claude headless quota
refresh is not yet available. Token counts and cost estimates are not account balance.

### 1.9 Verify and complete

**Diff & tests** captures the integrated candidate: the combined diff and every task's
test result. **Approve** binds the exact candidate commit, base branch, diff and passing
test evidence; any change revokes it. **Merge approved candidate** is a second, separate
confirmation. A reviewer coordinator can propose up to three rounds of quality feedback
but can never approve a merge.

### 1.10 Lessons and memory

A reviewed lesson records context, failure, cause, response and evidence with source file
hashes. Accepted lessons are stored in SQLite and mirrored to wiki pages in the run.
Before reuse, source hashes are checked; changed sources make a lesson stale. Fresh lessons
are given to the matching task (up to 8 KiB) and grant no permissions. agentmemory is
optional: `memory-sync` publishes reviewed summaries and `memory-import` reads cooperative
checkpoint reports from a local agentmemory server only when you ask; `memory-discover`
finds them without memory IDs.

## Part 2 — Command-line reference

Run directories must be outside the repository, and the repository must start clean.
`RUN` is a run directory, `TASK` a task ID, `AGENT` a registered agent ID.

### 2.1 Workspace and agents

| Command | Purpose |
|---|---|
| `agentconductor` | Open the default desktop workspace. |
| `agentconductor ui-new <workspace-dir>` | Open a desktop workspace in a chosen directory. |
| `agentconductor ui <run-dir>` | Open the Agent Room for an existing run (one window per run). |
| `agentconductor profiles <profiles.json>` | Write provider profile templates to review. |
| `agentconductor models-discover <profiles.json> AGENT <new-dir>` | Query a CLI's model catalog without a generation prompt. |
| `agentconductor catalog-import <profiles.json> AGENT <catalog.json>` | Import an operator-observed catalog when a CLI cannot list models. |

### 2.2 Plan and run

| Command | Purpose |
|---|---|
| `agentconductor plan <profiles.json> <coordinator> <repo> <new-plan-dir> "<goal>"` | Ask a read-only coordinator for a three-task plan. |
| `agentconductor validate <plan.json>` | Validate a plan and print its digest. |
| `agentconductor run <plan.json> <repo> <new-run-dir> <digest>` | Execute the approved plan. |
| `agentconductor status RUN` | Show run and task state. |
| `agentconductor logs RUN TASK` | Show a task's logs. |
| `agentconductor progress RUN TASK` | Show recent normalized events of the current attempt. |
| `agentconductor diff RUN` | Capture the integrated candidate diff and tests. |
| `agentconductor approve RUN <candidate-sha>` | Approve the exact candidate. |
| `agentconductor reject RUN "<reason>"` | Reject the candidate. |
| `agentconductor merge RUN` | Merge the approved candidate. |

A plan file can also be written by hand: a `tasks` array where each task has `id`,
`agent`, `prompt`, `paths`, `depends_on` and `test_command`, plus the agent profiles and
optional `"workspace_mode": "worktree"`. Validate it to obtain the digest.

### 2.3 Sessions, limits and recovery

| Command | Purpose |
|---|---|
| `agentconductor sessions RUN` | List task sessions. |
| `agentconductor briefing RUN` | Read-only team briefing. |
| `agentconductor quota-refresh RUN AGENT` | Refresh an account's quota (Codex). |
| `agentconductor schedule RUN TASK <YYYY-MM-DDTHH:MM:SS+HH:MM>` | Schedule a limited session. |
| `agentconductor reassign RUN TASK AGENT [model]` | Move a task to another agent. |
| `agentconductor session-model RUN TASK <model>` | Change a session's observed model. |
| `agentconductor revise RUN TASK "<feedback>"` | Targeted feedback (up to 16 KiB). |
| `agentconductor stop RUN "<reason>"` | Stop processes, keeping logs and partial files. |
| `agentconductor resume RUN` | Explicitly resume a stopped run. |
| `agentconductor resume-due RUN` | Resume scheduled work that is due. |
| `agentconductor watch RUN` | Keep resuming due work while running. |
| `agentconductor recover RUN` | After a crash, confirm processes are gone and mark the run stopped. |

A cooperative custom CLI that completes a `handoff` request continues automatically on its
reviewed fallback agent once its processes have exited; without a fallback the task stops
and waits for you. A `pause` always stops.

Fallback in a profile:

```json
"quota_group": "company-codex-account",
"fallback": { "agent": "company-cli", "model": null }
```

### 2.4 Tool requests

| Command | Purpose |
|---|---|
| `agentconductor tool-requests RUN` | List pending tool requests. |
| `agentconductor tool-approve RUN <request-sha256> [seconds]` | Allow one exact request. |
| `agentconductor tool-reject RUN <request-sha256>` | Deny it. |
| `agentconductor tool-refresh RUN <request-sha256>` | Create a new pending request (never renews approval). |

### 2.5 Review

| Command | Purpose |
|---|---|
| `agentconductor review RUN <coordinator>` | Request a quality-review proposal and its SHA-256. |
| `agentconductor apply-review RUN <proposal-sha256>` | Apply an inspected proposal. |
| `agentconductor review-apply RUN <coordinator>` | Review and apply immediately. |

### 2.6 Knowledge and memory

| Command | Purpose |
|---|---|
| `agentconductor knowledge-digest <lesson.json>` | Print a lesson draft's digest. |
| `agentconductor knowledge-accept RUN <lesson.json> <digest>` | Promote a reviewed lesson. |
| `agentconductor knowledge-status RUN` | Show lessons and their freshness. |
| `agentconductor recall RUN "<query>"` | Search local lessons offline. |
| `agentconductor memory-sync RUN <port>` | Publish reviewed summaries to local agentmemory. |
| `agentconductor memory-publish <port> <envelope.json>` | Publish a checkpoint envelope. |
| `agentconductor memory-read <port> <memory-id>` | Read one memory by exact ID. |
| `agentconductor memory-import RUN <port> <memory-id>` | Import a validated checkpoint observation. |
| `agentconductor memory-discover RUN <port>` | Find and import new checkpoint reports for every cooperative task without memory IDs. |

A lesson draft is JSON with `task`, `context`, `failure`, `cause`, `response`,
`evidence`, `confidence` (0–1), `revalidate` and `sources` (repository-relative `path`
and current `sha256`).

### 2.7 Cooperative checkpoints (custom CLIs)

A custom CLI that polls safely can register `"cooperative": "checkpoint_v1"` with exactly
one `{session_id}` argument. The engine supplies `AC_CHECKPOINT_RUN_DIR`,
`AC_CHECKPOINT_ADDRESS_FILE` and `AC_CHECKPOINT_EXECUTABLE`. Full contract:
[cooperative checkpoints](https://github.com/neodamul/AgentConductor/blob/main/docs/11-cooperative-checkpoints.md).

| Command | Purpose |
|---|---|
| `agentconductor checkpoint-issue RUN <issue.json>` | Address a pause/handoff request to a worker. |
| `agentconductor checkpoint-poll RUN <address.json>` | Worker polls for requests. |
| `agentconductor checkpoint-ack RUN <ack.json>` | Worker acknowledges a request. |
| `agentconductor checkpoint-report RUN <progress.json>` | Worker reports bounded progress. |

### 2.8 Diagnostics

| Command | Purpose |
|---|---|
| `agentconductor normalize <claude\|codex\|opencode\|custom> <stdout.jsonl>` | Normalize a CLI's JSONL output into events. |

## Part 3 — Troubleshooting

- **"coordinator requires read-only capabilities"** — choose a Codex or guarded Claude
  coordinator, or a custom CLI; planning always runs on a temporary snapshot.
- **"Agent Room is already open"** — close the other window for that run first.
- **Repository not clean** — commit or stash your changes before planning or running.
- **Session waiting with no reset time** — set **Resume at**, reassign or change model.
- **Windows window does not open** — install the Microsoft Edge WebView2 Runtime.
