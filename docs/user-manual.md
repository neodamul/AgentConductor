# AgentConductor User Manual

Version **0.1.17** · [한국어](user-manual.ko.md) · [Short introduction](../README.md)

## Part 1 — Desktop application

### 1.1 Requirements and installation

Use macOS 11+ on Apple silicon, or Windows x86_64 with WebView2 Runtime. Install Git and
at least one coding CLI, already signed in: Claude Code, Codex, OpenCode or your company CLI.
Projects can be ordinary folders; a Git repository and initial commit are optional.
There is no model API key registration or local UI web server.

Download your platform archive and `SHA256SUMS` from GitHub Releases. Verify the archive,
extract it and open `AgentConductor.app` or `agentconductor.exe`.
Packages include the standalone executable, both manuals, screenshots, dependency licenses and SBOM.
The app version is beside the title. The published release is 0.1.17; screenshots show the latest source build. Theme selection, task-first rows and Git-aware plan repair were added after that release and require a source build until a newer package includes them.

### 1.1a Application updates

The app checks GitHub Releases in the background every **10 minutes**. A newer installable release shows an update icon left of Settings. Choose **Workspace ▾ → Check for updates**, inspect the current/latest versions and choose **Update and restart**. Finish or stop runs and active app jobs first; the desktop refuses installation while any saved workspace has running work.

The installer downloads the matching macOS/Windows archive and its `SHA256SUMS`, checks the archive hash, replaces the executable or macOS bundle and restarts with the same arguments. It needs `curl`, access to GitHub and permission to write the installation location. On failure, inspect the update dialog; do not assume a newer version was installed.

`agentconductor update` installs a newer release over the executable without restarting it; finish or stop work before running the command. The current updater makes unauthenticated GitHub requests and does not reuse GitHub CLI credentials: private or restricted releases require manual download with your authorized browser or GitHub CLI. Do not add model API keys for application updates.

### 1.2 Workspaces and settings

A workspace holds goals, plans, role assignments and local history. Name it, choose an
existing project folder with **Browse…** or type the path, then **Save workspace**.
Git worktree isolation is optional; shared execution is the default.
Saving updates the name in the titlebar immediately and selects **Agent roles**. Tab changes preserve draft inputs; unsaved changes still require **Save workspace**.

Use **Workspace ▾** to create, open or switch workspaces, edit plan rules, share settings,
or configure messenger notifications. The same titlebar is available in Agent Room.
A workspace with an existing run opens its work view when appropriate; returning to
Workspace does not start a new plan. Unsaved changes require a choice before navigation.

**Delete** is available for the current workspace too. If planning is active,
**Stop planning and delete** waits for its owned process and admitted metadata.
Deleting the current workspace opens the most recently used remaining workspace; if none remains, the app creates an empty one. Active implementation requires **Stop run** first.
Only saved app-owned workspace data can be removed; the project is preserved.
Data moves to `~/.agentconductor/workspace-trash/<id>` for recovery.
**Remove from recents** hides an entry without deleting it.

![Workspace and goal configuration](assets/ui-workspace-current.png)

**Settings** includes:
- **Interface language**: English or 한국어. Typed goals, prompts and raw evidence are never translated.
- **Theme**: Light (default, neutral surfaces with a blue accent) or Dark (sage accents on dark surfaces). Save to apply immediately. The choice is stored with language in `~/.agentconductor/preferences.json` and follows you across workspaces. Both themes retain the same controls and information.
- **MCP servers…**: OpenCode-shaped JSON/JSONC command or remote definitions; use environment references for credentials.
  Register first, discover tools, then bind read-only Jira/Confluence imports. [MCP guide](mcp-sources.md).
- **Signal transport**: Local Rust (default) or agentmemory preferred, with local fallback. The server port appears only for agentmemory; switching to Local Rust preserves its value.
- **Clear cache…**: review paths and sizes before permanently removing eligible disposable metadata.

The workspace menu includes **Agent tools guide…** for optional agentmemory, RTK, Ponytail, Graphify and CocoIndex installation instructions. These tools are not bundled or required by the app.

![Korean workspace](assets/ui-workspace-korean-current.png)

![Dark Korean workspace](assets/ui-workspace-dark-current.png)

### 1.3 Agents and models

The app-owned workspaces share one machine-wide CLI list, `~/.agentconductor/agents.json`.
Deleting a workspace does not remove this list. Role assignments belong to each workspace.
Drag cards or use **Move up/Move down** to reorder agent selections.
An assignment to a missing agent is highlighted and prevents planning until corrected.

Use **+ Add** for Claude, Codex, OpenCode or Custom. **Edit** uses a pencil; Remove uses a trash icon.
- Executables are found asynchronously; **Browse…** selects a file when discovery fails.
- Arguments are a JSON array. The CLI keeps its existing authentication.
- Capabilities follow the executable and invocation rather than a permission dropdown. Standard Claude and Codex profiles run with full access by default, bypassing their native permission prompts.
  Changed/custom executables do not inherit that capability. OpenCode/custom profiles need an operator-verified sandbox or read-only configuration.
- Each profile supports up to three sessions and a quota group for a shared account.
- Set a fallback and its model. Mutual fallbacks are allowed; the engine stops after trying each agent once.
- Models load without a generation prompt where supported. **Models** refreshes the observed list.
  A company CLI without discovery uses operator-configured model IDs; `{model}` must map to its actual argv option.
- Quota refresh defaults to **5 minutes**, configurable independently from **1–60** in Edit.

Saving **Use profile** is asynchronous and closes after success; validation errors stay in the dialog.
Changing execution settings requires reviewing a plan again. Metadata-only refresh does not invalidate it.
One metadata collector runs at a time; queued requests do not block unrelated editing.

![Agent profile and refresh settings](assets/ui-agent-settings-current.png)

### 1.4 Goals, sources and roles

Use the persistent **Workspace settings**, **Agent roles** and **Review the plan** tabs. Switching tabs preserves unsaved inputs; only **Save workspace** or **Save plan edits** persists them. Planning completion selects the review tab automatically.
Describe the goal and choose the planning coordinator. Insert explicit `jira:EMM-52599`
and `confluence:53553565` references after registering MCP servers.
Preview and apply selected documents before planning; source changes require renewed review.
Registration alone does not run an MCP command.

Roles include verification, development, documentation, knowledge assetization and testing.
Choose agents and optional models/alternates. **Suggest defaults** uses the registered team.
The coordinator proposes scoped work following these assignments; the Rust engine owns execution and state.

Each goal has a short English slug, such as `ladder_game`, translated from the goal rather than romanized.
Goal folders are directly under the project, `<project>/<goal>/`; ordinary-folder history remains app-managed.
Existing older goal folders migrate when used. The goal tree contains
`plan/`, `phase/`, `task/`, `verification/` and `completed/`.
Do not edit progress files by hand; the app checks their identity.

### 1.5 Plan and approval

**Generate plan** reads a temporary snapshot and requests analysis plus ordered phases.
Preparation shows elapsed time before the coordinator starts. A failure keeps its cause in **Review the plan**; expand **Error details** to inspect and copy the retained CLI or parsing output before retrying. Logs remain available.
**Read plan…** converts an existing Markdown plan (up to 64 KiB), skipping completed work.
**Write plan manually** launches no coordinator; fill scoped tasks and test commands yourself.

Review every proposed phase up front. Phase headings identify its order and stage;
expand folded task details to edit instructions, agent/model, owned files, dependencies and tests.
Use **Save plan edits**, then **Approve plan & run** for the current phase.
Unfilled/overlapping tasks, a changed baseline or stale digest prevent approval.

**Goals & progress** shows each goal's phase tags and description.
After integrating a phase, **Plan phase n** loads the next saved phase without another planning call.
Replanning an existing phased goal loads its current phase with current profiles.
**Plan rules…** supports machine-wide rules or a workspace override.

For an ordinary folder, planning uses a temporary snapshot. Approved shared execution works directly in the selected folder, using Git metadata under `~/.agentconductor/project-history/` without creating `.git` in the project. Worktree mode isolates execution instead; review the candidate before applying it. Shared execution changes are already visible while agents work.
For a committed repository a clean baseline is required. Review the dialog before **Commit and continue**.
OS files can be ignored locally without changing `.gitignore`.
Shared ordinary-folder execution uses existing local dependencies. Worktree execution needs its own dependency setup.
The planner is told whether the project uses Git. For ordinary folders it proposes direct-file or project-runner checks instead of Git-dependent tests; any retained Git test command resolves through the external history. If evidence shows a test command cannot work here, the coordinator can replace it with a command verifying the same work, records the old/new command and reason, and reruns verification. It must not weaken a test that can run. Final candidate approval remains yours.

### 1.6 Agent Room and feedback

The status banner explains the run state, task/phase progress, failure cause and next action.
Session rows show the task name first, followed by the small agent icon, agent name and status tag. Long or translated statuses wrap within the rail. The left list uses green/yellow/red lights; running lights pulse and
tasks awaiting a decision blink. Reduced-motion settings suppress animation.
Usage badges stay in workspace agent cards, so the task list remains compact.

Select a task for **Activity**, **Diff & tests**, **Raw logs** or **Lessons**.
Activity displays messages, tool lines and result summaries; raw JSON remains in Raw logs.
Thinking sections can be folded; silence/stall warnings report observation limits rather than prove failure.
The coordinator is a selectable entry after tasks, showing integration steps, decisions and test output.

- **Send feedback** (⌘/Ctrl+Enter) revises one task. An idle run continues after feedback is recorded.
- **Fix with feedback** selects a failed task and drafts feedback from its failure output.
- Reassign agents or change observed models through the task inspector; existing work/logs remain.
- **Resume at** schedules an unavailable session. **Stop run**, **Resume** and **Recover** manage owned work.
- **Brief team** summarizes assigned work, latest activity, freshness, restrictions and next actions.

On reopening, automatic recovery checks that the interrupted run and its owned processes are quiescent before continuing unchanged work. It permits at most two attempts without new task progress. Explicitly stopped runs stay stopped, unknown ownership prevents recovery, and candidate approvals are revoked. Inspect the retained evidence if recovery cannot proceed.

![Agent Room showing an offline verification run](assets/ui-agent-room-current.png)

![Dark Korean Agent Room](assets/ui-agent-room-dark-current.png)

### 1.7 Tool decisions and verification

The following tool gates apply to guarded `native_callback` profiles. Standard full-access Claude/Codex profiles bypass native per-tool prompts; plan review and final candidate approval still apply.

Routine guarded Claude reads, edits, searches, builds and tests confined to the permitted workspace can run immediately.
Unsupported or unsafe requests are refused before evaluation.
Other requests are sent to the workspace coordinator for an allow/deny decision, recorded with its reason.
If the coordinator cannot return a valid decision, a person is asked.
The coordinator uses existing CLI authentication; it is not a new API service.

The tool request window shows a readable action, exact input under Raw input, live activity and the deadline.
**Allow once**, **Deny** or **Decide later** controls pending human decisions.
Approvals bind task, session, plan, workspace and input, expire within 900 seconds and are consumed once.
Waiting for a tool decision is excluded from the task time limit; a timed-out task is blocked for Resume.
Tool decisions do not approve integration.

Review **Diff & tests**. A changed verification report containing a Blocking/차단 section fails integration
for the responsible task. Fix it and rerun verification.
Large patches show a shortened preview and a command such as `agentconductor diff "<run-directory>"` for the complete patch. Truncation alone does not disable approval: verification hashes the entire patch, including omitted lines. Inspect the complete diff before approving when needed.
**Approve** binds the exact candidate, baseline, complete diff hash and passing tests.
Then use **Merge approved candidate** for Git, **Accept changes** for direct ordinary-folder execution, or **Apply changes** for legacy private-copy runs. Rejecting a direct-folder candidate restores owned files to their recorded baseline only when later manual edits would not be overwritten.
Any changed evidence revokes approval. Only the human approves final integration.

When all tasks complete but integration verification fails, the recorded coordinator can review evidence, revise affected tasks and rerun them. Repair stops after three review rounds or an invalid response; then revise or reassign the work yourself.

### 1.8 Quota and fallback

Paid CLI cards display reported remaining capacity, for example `64%/100% ↻`.
Enterprise metadata uses remaining/allocated percentages; rolling windows use the provider's labels
(5h/weekly, daily/weekly, etc.). Hover for raw amounts, reset time, source and freshness.
Credits without a known allocation do not imply a percentage. Token counts/cost estimates are not account balance.
Unavailable shows `—`; stale observations suppress percentages. Exhausted windows keep percentages compact; their reset time appears in the tooltip.

Queries run asynchronously with bounded output and deadlines, without generation requests.
Claude uses an isolated metadata session with tools/hooks/project settings disabled;
its folder-trust prompt can be answered only in that restricted collection session.
Theme/login setup must be completed in the CLI. Codex requires its existing ChatGPT login.
Actual Enterprise formats and a company CLI still require operator acceptance.

A reported restriction applies to the quota group. An explicit reset prefills **Resume at** and shows a countdown.
An independent coordinator stores the reservation and resumes at that deadline, even after the app closes.
Reopening restores a missing coordinator; reassignment cancels the old reservation, and **Stop** cancels automatic continuation.
If the reset requires more than five hours of waiting, the coordinator uses only the fallback in the reviewed plan.
With no known reset, that policy applies five hours after the original restriction.
Without an available planned fallback, the task waits. Unknown timezones are not guessed; use **Resume at** to specify a time.
The inspector shows the scheduled local time and remaining countdown, or a paused-timer reason. You can override the prefilled time. Up to three automatic resume attempts are admitted; changing a manual schedule renews the retry allowance. Reservation ownership belongs to the lightweight Rust coordinator, not an LLM session or a browser timer; only one watcher owns each run. Inspect retained evidence before restoring a reservation with unknown process ownership.
A transfer starts a new provider session after the current owner exits; files and logs remain.
Authentication/configuration errors are not automatically retried. Transfers require fresh verification and approval.

### 1.9 Memory, sharing and cache

Local SQLite checkpoints and audited decisions remain authoritative. A cooperative custom CLI polls
at safe boundaries; registering it does not give an arbitrary CLI live steering support.
**Local Rust** never contacts agentmemory. **agentmemory preferred** records one external attempt;
if unavailable or uncertain after restart, it retains the same local directive without replay.
The client has an 8-second bound. A stored memory receipt is not recipient acceptance.
These options do not deliver to agents on another computer.

Reviewed lessons include evidence and source hashes, are mirrored to Markdown wiki assets,
and become stale when their source changes. They grant no permissions.
Manual memory sync/import is optional.

**Export settings…** shares profiles, roles and verified goal archives.
Secrets, environment values and local model observations remain on the machine.
Review archives before sharing; imported capability claims require local confirmation.

**Messenger…** supports Slack/webhooks and optional allow/deny replies from listed approvers.
Webhook/bot tokens and relay secrets stay local. Goal text is included only by explicit choice.
Remote replies never approve a plan or merge.

**Clear cache…** previews eligible completed query/metadata/catalog files.
Permanent removal cannot be undone. Live/unknown/aliased data, current diagnostics, source folders,
reviewed plans, diffs, tests, audit and workspace recovery remain.
Eligible cache is trimmed to eight newest folders plus protected references at idle startup/settlement.
There is no resident cleanup daemon. Cargo build output is separate from application cache.

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
| `agentconductor plan <profiles.json> <coordinator> <repo> <new-plan-dir> "<goal>"` | Ask a read-only coordinator for a plan (1–12 tasks); `[roles.json]` adds role rules, `--from plan.md` converts an existing plan document; `--rules rules.md` adds plan rules; `--phases slug-dir` asks for an analysis and ordered phases and records them in a slug work tree. |
| `agentconductor validate <plan.json>` | Validate a plan and print its digest. |
| `agentconductor run <plan.json> <repo> <new-run-dir> <digest> [--phase slug-dir phase-id]` | Execute the approved plan; `--phase` moves that slug phase's tasks to verification and, after merge, to completed. |
| `agentconductor status RUN` | Show run and task state. |
| `agentconductor update` | Install the latest GitHub release over this executable (no restart). |
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
| `agentconductor quota-refresh RUN AGENT` | Refresh account quota (Codex; Claude `/usage` on macOS). |
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

## Part 3 — Validation and troubleshooting

The packaged native executable is checked on macOS and Windows using deterministic offline workers,
including Git/ordinary-folder projects, shared/worktree execution and refused unapproved integration.
This does not establish acceptance of your company's agent, Enterprise account or live messenger.

Previously exercised CLI contracts: Claude Code 2.1.287; Codex 0.162.0 tasks/resume and
0.162.1 quota metadata; OpenCode 2.0.25; agentmemory server 0.9.29.
Installed provider versions may differ. Unknown output stays unavailable or fails safely.

- **Missing executable / Windows error 193**: use the actual native or npm `.cmd` launcher; Edit → Browse can override discovery.
- **Quota unavailable**: inspect the tooltip, finish CLI theme/login setup and refresh. No allocation means no invented percentage.
- **Missing role agent**: select a registered agent for every highlighted assignment.
- **Dirty baseline**: inspect listed files before committing, stashing or choosing Stop.
- **Blocked/failed run**: inspect the cause and test output; Fix with feedback, correct configuration and Resume.
- **Interrupted ownership**: Recover only after owned processes are confirmed gone. Reopening permits bounded automatic continuation only after verified process absence; manually stopped runs remain stopped.
- **Changed source/approval**: plan or verify again. Ordinary-folder application has backups and a receipt; interrupted application is not replayed blindly.
- **MCP timeout**: inspect the registered command/server and its bounded timeout; keep raw logs private.
- **Window already open**: use or close the existing window rather than opening another.

## Optional tool commands

Run these yourself in the indicated agent CLI or a terminal; the app never installs them.

- `brew install uv`
- `winget install astral-sh.uv`
- `npx -y @agentmemory/agentmemory@latest`
- `/plugin marketplace add rohitg00/agentmemory`
- `/plugin install agentmemory`
- `codex plugin marketplace add rohitg00/agentmemory`
- `codex plugin add agentmemory@agentmemory`
- `/plugin marketplace add DietrichGebert/ponytail`
- `/plugin install ponytail@ponytail`
- `codex plugin marketplace add DietrichGebert/ponytail`
- `codex plugin add ponytail@ponytail`
- `brew install rtk`
- `winget install rtk-ai.rtk`
- `rtk init -g`
- `rtk init -g --codex`
- `rtk gain`
- `uv tool install graphifyy`
- `graphify install`
- `graphify install --platform codex`
- `graphify update .`
- `uv tool install --upgrade 'cocoindex-code[full]'`
- `/plugin marketplace add cocoindex-io/cocoindex-code`
- `/plugin install cocoindex-code@cocoindex-code`
- `codex mcp add cocoindex-code -- ccc mcp`
- `ccc init`
- `ccc index`
