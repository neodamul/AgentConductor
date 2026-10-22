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

### UC-28 Write the plan yourself

The operator enters a goal, chooses **Write plan manually**, and receives three empty tasks. For each task they choose an agent and write instructions, owned paths, dependencies and a test command, save the edits and approve the exact plan. Incomplete or overlapping tasks are refused with the reason and nothing runs. If the repository changed after the form was created, approval is refused. Trace: SF-31.

### UC-29 Choose the repository with a folder dialog

The operator types the goal, picks a coordinator, presses **Browse…** next to Repository and selects the project folder in the system dialog; the path appears in the field. Cancelling leaves the field unchanged. A workspace without a repository can be saved; plan buttons stay disabled until a repository is set. Trace: SF-32.

### UC-30 Switch workspaces from the menu

The operator opens the workspace menu in the title bar, sees the current folder and recent workspaces, and selects one, chooses **Open folder…**, or **New workspace…**. Unsaved edits are confirmed first. The window reloads with the selected workspace; a workspace already open in another window, or a running planning job, refuses the switch with the reason. Trace: SF-33.

### UC-31 Keep several named workspaces

The operator names the current workspace above the goal and saves. From the workspace menu they choose **New workspace…**, type a name and get an empty workspace with that name; the menu now lists both names. They switch back and forth, and remove an old entry with **Remove from list**; its folder stays on disk and **Open existing workspace…** lists it again from `~/.agentconductor` so it can be reopened. The current workspace cannot be removed. Trace: SF-34.

### UC-32 Read what changed in a release

The operator opens a GitHub Release and reads its added features, fixes and known limitations before downloading. A maintainer who tags a version without a changelog section sees the publish job fail before anything is released. Trace: SF-35.

### UC-33 Start from a repository with work in progress

The operator presses **Generate plan** or **Write plan manually** while the repository has uncommitted work. A dialog explains that agents' changes must be separable from the operator's own work, lists the changed files and offers **Commit and continue** or **Stop**. Commit records all current changes in one commit on the current branch and then runs the requested planning step; Stop leaves the repository and workspace untouched. A failed commit (for example a missing Git identity or a rejecting hook) shows Git's reason and does not plan. Trace: SF-36.

### UC-36 Ignore OS files and continue

The baseline dialog lists only `.DS_Store` files. It explains they are created by Finder and offers **Ignore and continue** with a note on making it permanent for every repository. The operator chooses it; the patterns are added to the repository's local exclude list and planning continues without a commit. If other changes remain, the dialog shows just those with **Commit and continue** / **Stop**. Trace: SF-36.

### UC-37 Watch a plan being generated

The operator presses **Generate plan**. The review area immediately shows "Planning with claude… 0:05" and updates every second. About a minute later the plan appears, a notice says it is ready and the window scrolls to it. If the coordinator fails, the review area shows the exit status and the log folder instead. Trace: SF-38.

### UC-38 Read and edit a task's instructions

The operator opens a generated plan. Each task's instructions read as labelled sections — *Goal*, *Inputs*, *Steps*, *Expected output*, *Done when* — one item per line, in the language of the goal. The box shows the whole text without scrolling for typical tasks and grows while the operator adds lines. Trace: SF-39.

### UC-39 Define who does what, then plan

In **Workspace settings → Agent roles** the operator sets Verification: claude, Development: codex, Documentation: codex, Knowledge: codex, Testing: opencode (alternate: company-cli) and saves. **Generate plan** sends these rules to the coordinator. Each proposed task shows its role and an agent from that role's list. If the coordinator assigns a task outside the rules, planning fails with the task and rule named, and nothing is proposed. Trace: SF-40.

### UC-40 Register an agent and keep it

The operator adds a company CLI with **+ Add → Use profile**. The card appears and a notice confirms it was saved; closing and reopening the window still shows it, while a half-typed goal that was not saved is unaffected. Removing the agent later also takes it out of any role rule; if it was the coordinator, the first remaining agent becomes the coordinator. Trace: SF-41.

### UC-41 Choose models from ready lists

The operator adds a Codex agent with **+ Add**. Within a few seconds its card shows the model count and the model selectors in **2. Agent roles** list those models without the table redrawing or focus moving. **Models** on a card refreshes that agent's list. If a CLI cannot list models, the card keeps *Models not queried* and a notice explains; it is not retried until the next session. Trace: SF-42.

### UC-42 Share the workspace configuration through Git

Developer A chooses **Workspace ▾ → Export settings…**, accepts the repository folder and commits `agentconductor.workspace.json`. Developer B pulls, chooses **Import settings…**, selects the repository folder and sees the agents, executables, arguments, capabilities, role rules and warnings. Capabilities that A verified for an external sandbox or read-only use are marked to be imported as *Not verified* unless B confirms. B applies; the workspace now has the shared agents, roles and coordinator, B's own environment values and model lists are kept, and the repository is set to that folder. Trace: SF-43.

### UC-43 Continue a project from its plan document

The operator registers a workspace for a project that already has `docs/plan.md` with nine open items. In step 2 they keep the role rules and press **Read plan…**, choose the file and wait while "Planning with claude…" is shown. Seven tasks appear, each with a role and an allowed agent, covering the open items; completed items are absent. They remove one task they want to postpone, add one manual task, save the edits and approve. Trace: SF-44.

### UC-44 Pursue a new goal in the same workspace

The operator enters "Adjust the health check interval" and presses Generate plan. The window proposes the slug `healthcheck_interval_adjust` (editable), creates its tree and shows it in the slug list next to the earlier `healthcheck_feature` slug, whose progress stays visible. Trace: SF-45.

### UC-45 Execute a phased plan

The coordinator returns an analysis and three phases. The operator reviews phase 1, approves it, watches its tasks move to verification when tests run and to completed after merge, then approves phase 2. Phase 3 cannot start before phase 2 is completed. Trace: SF-46.

### UC-46 See where every goal stands

The slug list shows `healthcheck_feature — completed`, `healthcheck_interval_adjust — phase 2 of 3, 1 task in verification`. Opening a slug lists phases with their tasks per stage. A file moved by hand into the wrong folder is reported as a tree mismatch. Trace: SF-47.

### UC-47 Hand a workspace to a teammate with its history

Export writes `agentconductor.workspace.json`, `healthcheck_feature.zip` and `healthcheck_interval_adjust.zip`. The teammate imports from that folder; missing or renamed archives refuse the import; afterwards both slugs appear with the same progress. Trace: SF-48.

### UC-48 Adjust the team's planning rules

The operator opens **Plan rules**, edits the global rules (for example the line limit) or switches the workspace to its own rules, saves, and the next plan follows them. Trace: SF-49.

### UC-34 Receive a decision request in the messenger (planned)

A worker asks to run a command while the operator is away. The linked channel receives a message with the workspace, task, agent, a redacted summary of the request and a link to open it in the desktop application. A run completion, failure or usage-limit wait produces a similar message. Trace: SF-37.

### UC-35 Answer a pending decision from the messenger (planned)

A linked approver presses **Allow** or **Deny** on the message. The application verifies the messenger signature, the approver's link and that the exact request is still pending and unexpired, then records the decision with the messenger identity in the audit log. Plan execution consent and final merge remain desktop-only until separately approved as a requirement. Trace: SF-37.

### UC-49 Review Claude capacity

While a Claude task runs, the operator sees a provider-reported utilization of 0.27 as 73% remaining in the relevant window. Refresh rechecks the retained observation; missing utilization stays unavailable and observations older than five minutes stay stale. Trace: SF-50.

### UC-50 Scan capacity while choosing an agent — The operator sees 73% beside an agent name; the tooltip explains the reported windows, source and freshness. Trace: SF-51.

### UC-51 Import a Story and supporting pages (planned)

Configure Jira and Confluence MCP servers, enter `Implement jira:EMM-52599. References confluence:53553565, confluence:548443335.`, preview the resolved Story and pages, and import them before generating a plan. The Story supplies description and acceptance criteria; pages supply supporting evidence.

### UC-52 Resolve shorthand safely

For `Implement EMM-52599. References (53553565, 548443335).`, show proposed Jira/Confluence chips for confirmation. Bare numeric strings never silently trigger document access. The source chooser inserts explicit keywords; missing bindings or ambiguous tenants stop resolution. Trace: SF-52.

### UC-53 Refresh one agent

Activate the refresh icon beside one paid agent to update its account metadata; the other agents keep independent deadlines. Automatic refresh happens every two minutes while the visible UI coordinator is idle. Unsupported providers show a disabled refresh control. Trace: SF-53.


### UC-54 Read and refresh account windows

An operator scans an agent row, sees slash-separated percentages, inspects Current / Total for an explicitly reported Enterprise plan or provider duration labels in the tooltip, then refreshes that same compact group. Missing values remain unknown and stale observations remain visibly stale. Trace: SF-54.


R-64 / UC-54 interaction refinement: keep a 5px gap between quota values and refresh. All enabled buttons, including their icon/text children, show a pointer cursor; disabled controls keep the default cursor.

### UC-55 Edit agent runtime preferences

Open Edit; the installed executable path is suggested without replacing a custom path. Browse if needed, set refresh minutes, and save. A cadence-only change preserves a reviewed plan; executable changes require another review. Background quota checks update values silently. The launcher uses the same AC mark as the titlebar. Trace: SF-55.

### UC-56 Switch an agent to another provider

Open Edit or + Add, choose Provider (first field). ID, executable path and options change together; the operator may still adjust them, then saves. Role rules, coordinator and fallbacks that used the old ID now use the new one. Trace: SF-56.

### UC-57 See when an exhausted agent returns

An agent row shows `0%/40% · 14:30`; hovering explains each window and its reset. Trace: SF-57.

### UC-58 Choose the interface language

Open **Settings** in the title bar, choose 한국어 and save. Menus, steps, buttons, dialogs and the Agent Room switch to Korean at once without losing unsaved edits; the next start opens in Korean. Choosing English switches back. Trace: SF-58.

### UC-59 Choose a fallback and its model

Pick **Alternate agent · on account restriction** → agent; the model list shows `Loading models…` until the agent's models arrive, then the operator picks one or the CLI default. Trace: SF-59.

### UC-61 Connect a workspace to a messenger (phase 1)

From the workspace menu choose **Messenger…**, pick Slack or Company messenger (webhook), paste the webhook URL, choose events, optionally edit the JSON body template (company) and allow the goal text, press **Send test message**, then **Save**. Afterwards the channel receives one message per event, for example "AgentConductor · Payments team: candidate ready for review (3/3 tasks completed)". Trace: SF-37 / UC-34.

DD-68 UI amendment (2026-10-11): configure MCP connections through the top-bar Settings menu; keep reference insertion and preview beside the goal. The Settings gear uses a 26 px glyph in a 44 × 44 px pointer target.


### UC-60 Inspect agents on Windows
The operator registers an installed CLI and sees its provider badge, then refreshes capacity. The collector returns actual reported windows or an explicit authentication/trust/timeout error; it never generates a model turn or accepts trust automatically. Trace: SF-60.


### UC-62 Configure models for a CLI without discovery
The operator presses Models, sees the discovery limitation, enters company-provided IDs one per line and saves. Reopening shows retained IDs; role/session selectors use the catalog. A model selection requires an actual {model} execution-argument binding. Trace: SF-61.

### UC-63 Allow or deny a tool request from the messenger (phase 2)

A worker pauses on a tool request while the operator is away. The channel receives "Tool request needs a decision — task build, tool Bash. Reply `allow 3FA9C21B` or `deny 3FA9C21B`." A listed approver replies `allow 3FA9C21B`. Within about 15 seconds AgentConductor applies the decision exactly as the desktop **Allow** would, records the approver identity and posts "Allowed by U123 (task build)". Replies from other users, unknown or expired codes, repeated replies and bot messages are ignored and logged. Trace: SF-37 / UC-35.

### UC-64 Start a goal without naming its folder

Type the goal "헬스체크 주기 조정" and generate the plan. The work folder becomes `.agentconductor/helseuchekeu_jugi_jojeong/`; a second goal with the same words becomes `…_2`. Goals & progress lists the slug. Trace: SF-46.

### UC-65 Review Enterprise allocation
The operator refreshes an Enterprise agent and sees remaining/allocated from reported budget metadata; an absent total stays unknown. Hover shows scope/source/reset. If CLI collection fails, preserve the exact bounded failure locally for diagnosis rather than fabricate capacity. Trace: SF-62.


### UC-66 Browse a project and run agents without hidden or extra windows
The operator presses Browse, selects or cancels the foreground picker, and retains the form. Launching the Windows app and refreshing metadata produces no command window. Terminal CLI commands still return output and exit status. Trace: SF-63.


### UC-67 Save a fresh project and query account capacity
Select a plain directory and save a named workspace. Reopen to recover the same path. Quota refresh runs without a Git repository or execution run. Attempting planning explains the missing Git/initial commit before creating artifacts. Trace: SF-64.


### UC-68 Register another agent during a slow model query
While agent A queries models, the operator opens and saves another profile, changes the workspace name and requests agent B metadata. B is queued. Late results update only a matching profile; operator changes remain intact. Executable discovery never overwrites text entered while awaiting the result. Trace: SF-65.


### UC-69 Generate a plan for a fresh project
Select an existing plain project folder and save. Generate plan asks to create a Git baseline. Stop changes nothing. Approve initializes Git and an empty commit; existing untracked/staged files require separate baseline-file consent. The normal plan flow then continues. Trace: SF-66.


### UC-70 Plan and apply changes in a folder without Git
Select a plain folder, register agents and generate or write a plan. Approve the exact plan; three agents work in the private shared copy or optional worktrees. Review the diff and tests, approve, then choose Apply changes. The original has no new .git. If another editor changes the original after planning or verification, starting/applying is refused and a new plan is required. Trace: SF-67.


### UC-71 Look up how to install an agent tool
Open Settings, choose Agent tools guide…, read the tool's purpose and copy its Claude Code or Codex command into a terminal, then close the dialog. Nothing is installed or executed by the app. Trace: SF-68.


### UC-72 Browse to a Korean-named project folder on Windows
On Korean Windows, choose Browse…, pick C:\사다리\프로젝트 and save. The saved path equals the folder; Generate plan proceeds. Trace: SF-69.


### UC-73 Watch a slow plan
Generate plan on a large ordinary folder on Windows: no timeout notice appears while the copy is prepared. The review area shows elapsed time and "Waiting for the coordinator's first output.", then "Last coordinator output Ns ago (K KB).", and warns after two minutes without new output. Trace: SF-70.


### UC-74 Plan a Korean goal
Enter "사다리 게임을 만들자", Generate plan: the goal folder starts as goal_<date>, and when the plan is ready it is ladder_game (or the coordinator's English name, with _2 when taken). Trace: SF-71.


### UC-75 Read Enterprise capacity
A Codex Enterprise account with 640 of 1000 remaining shows 64%/100%; hovering shows 640/1000. Trace: SF-72.


### UC-76 Refresh Claude capacity before first CLI use
On a Windows computer where the Claude CLI was never set up, choose ↻ on the Claude agent: the tooltip says first-run setup is not finished and to run claude once in a terminal. After setup, ↻ shows percentages. Trace: SF-73.


### UC-77 Readable Windows project paths
Trace: R-87 -> BG-42 -> SF-74 -> UC-77 -> FR-72 -> QAS-83 -> DD-95.
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity.


### UC-78 Reliable named workspace creation
Trace: R-88 -> BG-43 -> SF-75 -> UC-78 -> FR-73 -> QAS-84 -> DD-96.
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable.


### UC-79 Delete an unwanted workspace
Trace: R-89 -> BG-44 -> SF-76 -> UC-79 -> FR-74 -> QAS-85 -> DD-97.
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first.


### UC-80 Register MCP servers in Settings
Trace: R-90 -> BG-45 -> SF-77 -> UC-80 -> FR-75 -> QAS-86 -> DD-98.
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration.


### UC-81 Save a Claude profile with a Codex fallback
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


UC-76 amendment (PR #1): refreshing Claude without a selected project folder reports the prerequisite without spawning a collector. With a trusted project, all-model and named-model weekly observations remain distinct. UC-16 amendment: Codex API-key login reports unsupported subscription quota and directs the operator to existing ChatGPT CLI login.


### UC-82 Choose signal transport and continue through a memory outage
Actor: operator; secondary actors: Rust engine and addressed CLI session. Preconditions: a signal-capable session with an exact current binding; direct input or a verified cooperative delivery boundary exists.
1. Open workspace Settings and choose Local Rust or agentmemory preferred; persist the choice asynchronously.
2. With Local Rust selected, issue a direction/report without invoking agentmemory, even if its server is healthy.
3. With agentmemory preferred selected, persist the outgoing signal locally before attempted delivery.
4. If the agentmemory server or memory/signal MCP adapter fails or times out, retain the same identity and deliver through the Rust local adapter when the recipient is locally reachable.
5. Record receipt and acceptance separately; retain the signal until its definite outcome. Acknowledge only the exact session/message.
6. Show the effective transport and fallback cause in settings/status/briefing. After recovery, apply the selected preference to new messages; do not replay acknowledged ones.
Alternate flows: a non-steerable session accepts at its supported checkpoint; an incapable runtime needs an explicitly reviewed resume/new-attempt transition. Remote recipients with no live transport remain pending with a visible reason. A backend setting change cannot authorize execution, merge, secrets or an automatic process takeover.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### UC-83 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.


### UC-84 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.


### UC-85 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.

### UC-86 Agents work inside the workspace sandbox; the coordinator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Work confined to the task workspace runs at once; a request reaching outside it (network, installs, history-changing Git, privilege, other processes, outside paths, secrets) is decided by the workspace coordinator , and a person is asked only when no coordinator can decide. The permission level is not chosen in the agent profile; it follows from the CLI. Time spent waiting for a decision does not count toward the task limit, and a task that times out can be resumed instead of failing.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.

### UC-87 Usage query accepts the Claude folder-trust prompt
Claude usage must load even when Claude first asks whether to trust the project folder. The usage query runs in a separate session with no tools, hooks or project settings, so it accepts the prompt automatically in both forms (Enter for the default choice, y for the screen-reader "Enter y/n:" form) at most twice, and stops with guidance instead of waiting for a timeout if Claude keeps asking.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.

### UC-88 Planning start and workspace naming feedback
Generate plan shows progress with elapsed time from the moment it is pressed, including while the project folder is copied before the coordinator starts. Saving workspace settings updates the titlebar name at once. Deleting the current workspace asks for the name of the fresh workspace that replaces it instead of always naming it "New workspace".
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.

### UC-89 Run and task status at a glance
The Agent Room shows the run's current final state immediately: a run status banner with the state in bold colour, task progress and the last error, and a prominent Resume button whenever the run can continue. Task states (failed, completed, pending, waiting, running) are bold and colour-coded in the session heading and in the left session list. In the left list a failed task is red, a task waiting for a decision or tool request has a blinking border, pending tasks use a strong colour, and the Tool requests count turns red and blinks while requests wait.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.

### UC-90 Goal folders directly in the project folder
When a workspace has a project folder, each goal (slug) folder is created directly in it as <project>/<slug>/ (for example C:Templadderladder_and_priority_games) with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Goal folders in a former place, including the interim <project>/<workspace name>/<slug>/ of an earlier build that misread the request, move there when first used.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.

### UC-91 Agent CLI list is a master list
The registered agent CLI list is kept as a master list for the machine, not per workspace: deleting a workspace never removes agents, and every workspace opens with the master list. When a workspace refers to an agent the master list no longer has (planning coordinator or role assignments), the assignment is shown in red and can be reassigned; Generate plan stays disabled until every assignment refers to a listed agent.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.

### UC-92 Reorder agent cards
Agent CLI cards can be moved up and down with the mouse (drag and drop), with keyboard-accessible move controls as an alternative. The order is saved, and every agent selection list (planning coordinator, roles, fallback) follows the same order.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.

### UC-93 Readable file paths everywhere
Screens, error messages and notifications never show Windows extended-length path prefixes such as \\?\C:\Work\Temp or //?/C:/Work/Temp; paths read as C:\Work\Temp.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.

### UC-94 Activity tab reads like the CLI
The Activity tab shows agent output the way the CLI shows it: assistant messages as text, tool calls as a one-line action with its key input, tool results as short summaries and errors highlighted. Raw JSON is never shown there; it remains in the Raw logs tab.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.

### UC-95 Coordinator final step in the task list
The Agent Room task list shows the step the coordinator runs after the tasks, integration verification of the combined result, as its own entry with its status (pending, running, completed, failed) and which task tests failed, so a run that fails after every task completed shows why without opening other views.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.

### UC-96 Decide a live tool request from its window
When a tool request window opens while the run is working, Allow once and Deny are available at once, without closing the window with Decide later and deciding again from the Tool requests list.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.

### UC-97 Coordinator decides routine requests and records each decision
Simple decisions are made by the coordinator so the user is not asked often: it allows routine development requests, denies only publishing, outside-project changes, secret exposure or system changes, also for runs started before orchestration existed, and every decision is recorded with the request, the result, the reason and the time, visible in the task's Activity. Commands inside a workspace that itself lives under an app folder (such as ~/.agentconductor) are workspace commands.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.

### UC-98 Task list without account usage
The Agent Room task list does not show account usage badges or refresh buttons; account limits are shown in the inspector's Account capacity section.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.

### UC-99 The coordinator is selectable and shows its work; one name
The interface and documents use one name, coordinator, for the agent that plans, verifies the combined result and decides tool requests (previously also called orchestrator). In the Agent Room task list the coordinator entry is selected like a task: the center shows its heading, status and an updating Activity of its run steps, tool decisions and integration test results, and Raw logs show the integration test output, following the Agent room design (select on the left, progress in the center, evidence on the right).
Trace: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117. Source: user request, 2026-10-13.

### UC-100 Failure cause and the next action on a failed run
When a run fails, the status banner shows why (for an integration verification failure, the failed task and the first lines of its test output) and offers Fix with feedback, which selects the failed task and drafts a direction with the failure output for the user to review and send; sending it reruns the task and repeats verification.
Trace: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118. Source: user request, 2026-10-13.

### UC-101 The next step is always shown
The Agent Room status banner always says what to do next and offers that action: review the changes when the run waits for approval (Approve stays unavailable until the latest evidence is reviewed), approve after review, apply or merge after approval, and return to the workspace to plan the next phase when the run is complete.
Trace: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119. Source: user request, 2026-10-13.

### UC-102 Plan review and run completion are organised by phase
A phased goal shows its phases in Review the plan: which phase is under review (phase n of m), which are completed, which comes next, and a Plan phase n action when the next phase is ready to load. When a run finishes, the Agent Room says which phase completed and which phase is next, and that it is planned from the workspace; a completed phase is recorded even if the goal folder moved after the run started.
Trace: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121. Source: user request, 2026-10-13.

### UC-103 A blocking verification finding fails integration verification
When a verification report changed in the run records a blocking issue (a Blocking/차단 section with items, or a positive blocking-issue count), integration verification fails like a failing test, naming the report and the task responsible for the files the issue names, so the coordinator does not accept a candidate its verifier rejected and Fix with feedback drafts the correction for that task.
Trace: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120. Source: user request, 2026-10-13.

### UC-104 The run's phase beside the workspace name
The Agent Room titlebar shows, beside the workspace name, the phase the run executes (phase n of m and its name) and what it covers (the phase description, or its task list when the phase file has no description); the full text is in its tooltip.
Trace: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122. Source: user request, 2026-10-13.

### UC-105 Readable Agent Room text and agent icons sized to their names
The Agent Room's text is at least 11 px and scaled up in proportion (body text 14 px instead of 12 px) for readability, workspace screen text is at least 11 px, and the agent icons in the task list are the height of the name beside them instead of a large tile.
Trace: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123. Source: user request, 2026-10-13.

### UC-106 Task cards in Review the plan fold to one line
In Review the plan each task is folded by default, showing only its number, name and a one-line description (the first line of its instructions that is not a label); an arrow unfolds it to show and edit every field.
Trace: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124. Source: user request, 2026-10-13.

### UC-107 Review the whole phased plan up front
The coordinator plans every phase of a goal when the plan is generated, so Review the plan briefs each phase (what it will do, from the coordinator's analysis) and lists each phase's planned tasks, with the coordinator's analysis of the whole goal available; the phase about to run is open and its tasks are edited below, completed phases are marked, and later phases can be unfolded to review their tasks before any of them runs.
Trace: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125. Source: user request, 2026-10-13.

### UC-108 A run survives closing or losing the desktop app
Closing the desktop app (or the app process ending) must not end a run: the run coordinator and its agent workers keep working and the app reattaches when reopened. Closing the window while a run is active asks: Keep running in the background (default), Stop the run and close, or Cancel.
Trace: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127. Source: user request, 2026-10-13.

### UC-109 Interrupted runs recover and resume automatically
When a run is recorded as running but its coordinator and workers are gone (app killed, crash, reboot), opening the workspace or the Agent Room recovers it automatically (verifying every recorded process is absent, as Recover does) and, with automatic resume on by default, resumes it: completed tasks are kept, the interrupted task continues in its own native session (or a new one if the session cannot resume) with a note that its previous attempt was interrupted and its owned files must be inspected. After two consecutive automatic resumes that are interrupted again, it stops and asks the operator.
Trace: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128. Source: user request, 2026-10-13.

### UC-110 Interruptions and silent workers are visible
The Agent Room shows when a run was interrupted and recovered (banner history and coordinator Activity), collapses a worker's consecutive thinking_tokens events into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows, and warns when a running task has produced no output for several minutes or its process is gone, suggesting Recover.
Trace: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129. Source: user request, 2026-10-13.

### UC-111 Reassign tasks that have not started while the run is working
While a run is working, the operator can change the agent (and model) of a task that has not started yet, for example to move work from a busy Claude account to Codex; the change applies before that task starts. Running and finished tasks still need the run to be idle.
Trace: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130. Source: user request, 2026-10-13.

### UC-112 Agents run with the highest permission by default
Agent workers run with the highest permission their CLI offers by default (Claude without permission prompts and with the operator's MCP servers such as Playwright, so it can open and inspect a browser UI; Codex without approvals or sandbox), while the planning and review coordinator stays read-only. The operator can lower a profile's permission.
Trace: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131. Source: user request, 2026-10-13.

### UC-113 The coordinator resolves verification failures
When every task of a run has completed but integration verification fails (a failing test command or a verification report that records a blocking issue), the run does not stop at Failed waiting for the operator: the coordinator reviews the evidence, revises the tasks whose files must change and the verifying task, and the run resumes. A finding the run cannot resolve (outside every task scope, or needing an environment the agents lack) is recorded by the verifying task as a deferred non-blocking follow-up with its reason and owner. After three repair rounds without passing, the run stays Failed with the reason, for the operator to revise or reassign. Resume on such a failed run starts this repair.
Trace: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132. Source: user request, 2026-10-13.

### UC-114 Execute, accept or restore a selected folder
Choose a project, review a plan, and start. Files appear in the selected project without `.git`. Review evidence and Accept the exact result, or Reject to restore owned files. If files changed after verification, show the conflict and preserve the manual edit. Existing private-copy runs continue with their previous Apply flow.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user workspace-path requirement and Claude review, 2026-10-13.

### UC-114 Deleting workspaces needs no replacement name
Deleting a workspace asks for no new workspace name. When the current workspace is deleted, the most recently used other workspace opens, so several workspaces can be deleted in a row; only when none is left does one empty workspace start.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user request, 2026-10-13.

### UC-115 Read the current application manual
Open either manual, follow workspace tabs, register CLIs and review a candidate. Match displayed labels, permissions, folder execution and recovery behavior to the documented version.
Trace: R-125 -> BG-80 -> SF-112 -> UC-115 -> FR-110 -> QAS-121 -> DD-134. Source: user release/manual requests, 2026-10-14.

### UC-116 The app updates itself from GitHub releases
Every 10 minutes the app checks the latest GitHub release in the background. When a newer version exists, an update icon, a downward arrow, appears left of Settings; the icon and the workspace menu item Check for updates open the same dialog with the installed and latest versions. Update and restart downloads the archive for this platform, verifies it against the release SHA256SUMS, replaces the installed executable (or macOS app bundle) and restarts the app. It is refused while any run or app job is active, because restarting ends their agents. The update repository defaults to the project's GitHub repository. Settings can change it to a GitHub `owner/name` or an HTTPS repository URL on any GitHub-compatible host, such as an in-house GitHub Enterprise server; release notes, the archive and `SHA256SUMS` come from that repository's latest release, and only the repository is configured, not the download path (#130).
Trace: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135. Source: user request, 2026-10-14.


### UC-117 Wait, restart, reassign or fall back after quota exhaustion
Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

### UC-118 Switch the theme

Open ⚙ Settings, choose Theme → Dark, save. Both screens turn dark at once without losing edits; the next start opens dark. In Agent Room a long status such as "Waiting for decision" wraps under the agent name next to its icon instead of leaving the list. Trace: SF-115.

### UC-119 Plans know whether the project uses Git, and verification commands that cannot work are replaced
The planning coordinator is told whether the project folder is a Git repository and does not plan git test commands when it is not. If a planned test command cannot work in the project (for example git in a folder without Git) or an agent reports that its test command must change, the coordinator review replaces it with a command that verifies the same work instead of deferring the failure, and the replacement is recorded. Test commands of a folder without Git run against the history AgentConductor keeps outside it.
Trace: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138. Source: user request, 2026-10-14.

### UC-120 Read quota and prepare a workspace

The operator sees an exhausted account as 0%/24% followed by refresh. Hovering the value reveals each window and reset time, or reset unknown. Opening Workspace, Agent roles or Review the plan shows its explanation directly under the title. Helper text beside action buttons uses remaining width without a fixed narrow column.
Trace: R-130 -> BG-85 -> SF-117 -> UC-120 -> FR-115 -> QAS-126 -> DD-139. Source: user, 2026-10-14.

### UC-121 Select signal transport and open tool instructions

Open Settings: labels and selection controls are separated. Choose Local Rust: server fields disappear. Choose agentmemory: the retained server port appears. Open the workspace menu and Agent tools guide: the existing static guide opens without executing installation commands.
Trace: R-131 -> BG-86 -> SF-118 -> UC-121 -> FR-116 -> QAS-127 -> DD-140. Source: user, 2026-10-14.

### UC-122 Custom CLIs that approve their own tools can run
A custom agent CLI that approves its own tool use (for example one set to auto-approve tools) can be marked so in its agent settings; it then runs after plan approval instead of being refused as unverified. The mark is the operator's claim for this machine and is not trusted when settings are imported from another machine.
Trace: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141. Source: user request, 2026-10-14.

### UC-123 Read release notes before update

Open Check for updates. Read the release link followed by its actual body; scroll or focus the body with the keyboard for long notes. Missing notes hide the panel without affecting installation.
Trace: R-133 -> BG-88 -> SF-120 -> UC-123 -> FR-118 -> QAS-129 -> DD-142. Source: user, 2026-10-14; next release after v0.1.18.

### UC-124 Preview and summarize referenced documents

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.


### UC-125 Retry a failed plan without stale results

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.


### UC-126 Approve a reviewed plan and wait for Claude availability

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.


### UC-127 / UC-128 Recover transport and open the manual

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

### UC-129 Workspace and Agent Room feedback refinements
Step 1 shows the planning coordinator right of the workspace name and the project folder above the goal, without a divider. MCP lookups show that they are running; Preview opens its window at once with a progress pulse and fills it when the lookup ends or shows the failure. jira:KEY and confluence:ID references in the goal are shown as tags that stay plain, deletable text, and hovering one shows its title and summary. The goal heading status follows the started run instead of staying Started. In the Agent Room, account usage shows Enterprise budgets as a remaining percentage like the agent cards, and the task shown on the right has a strong border in the list.
Trace: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148. Source: user requests, 2026-10-14.

### UC-130 Runs keep going and stay visible
A model reply such as "Selected model is at capacity" is retried after a short wait, at most three times, for tasks and for the coordinator. Files an agent creates outside its owned paths in a project without its own Git (for example a CLI saving .aipro/plans) are moved out of the project into the run folder and recorded. Agent Room task rows lead with the reviewed task summary. A pressed button shows that its work is running, distinct from a disabled button, until the request and any background job it started end. The workspace page and the Agent Room stay open side by side: the operator switches freely, the room keeps its live progress, and the workspace shows the run state on its back-to-run button.
Trace: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149. Source: user requests, 2026-10-14.


### UC-131 Work without memory tools / UC-132 pick a project or file

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

### UC-134 Read command tables from the app menu

Open Workspace menu and User manual. Read real column headers and command rows, scroll long content, view screenshots and code. Changing interface language while open loads that language. A standalone HTML copy works offline.
Trace: R-144 -> BG-99 -> SF-131 -> UC-134 -> FR-129 -> QAS-140 -> DD-153. Source: user, 2026-10-14.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

### UC-137 Create a workspace after viewing completed work

View a completed Agent Room, return home, create a named workspace; the new workspace opens and old evidence remains. Reject navigation while the previous execution is active.
Trace: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156. GitHub #4, user2026-10-14.

### UC-138 Query quota from a Claude chevron prompt

A Claude CLI starts with ❯, > or $; the collector sends /usage once and stores reported metadata, then reaps its process. Onboarding/login/trust blockers retain their existing handling.
Trace: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157. GitHub #5, user2026-10-14.

DD-157 implementation note: the operator confirmed Windows Claude2.1.296 uses a > prompt and a Yes-I-trust-folder choice, with an explicit spent/allocated Enterprise line. Parse that declared allocation using existing exact-decimal subtraction; the display percentage is derived from remaining/allocated amounts. Date-only resets remain unscheduled. Committed fixtures use synthetic amounts; no operator account values are published.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

### UC-140 Add marble-game functionality after a ladder-game goal

Complete a goal; select New goal in the same workspace/project; save another goal and plan it. Switch back to inspect the previous goal/run. If a new workspace chooses the same folder, identify its existing owner and preserve the form.
Trace: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159. GitHub #6, user2026-10-15.

### UC-141 Build and release AgentConductor using Rust
A maintainer changes an application screen, verifies the behavior with Rust tests, builds the release binary, produces bilingual manuals and release archives, and validates the packages without running project-owned JavaScript or Python code. The installed application provides the same accessible, bilingual controls without a JavaScript UI runtime.
Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160.

### UC-142 Retry a plan after slow or failed preparation
The operator retries Generate plan after an earlier planning failure or partial phase. The review screen clears stale failure feedback, shows preparation progress, and continues automatic status checks. If a request times out, localized guidance offers reopening the workspace or run if status remains stale.
Trace: R-152 → BG-107 → SF-139 → UC-142 → FR-137 → QAS-148 → DD-162. GitHub #36, 2026-10-19.

### UC-144 Configure a custom CLI's event and quota behavior
Actor: workspace operator. Export workspace settings, add a custom profile's `event_types`, optional `session_id_pointer`, and optional `quota_command`, then import and review the settings. Run a goal and verify mapped progress/result/session events. Request account metadata only when a quota command is configured; validate its bounded JSON Snapshot and retain stdout/stderr logs. Invalid pointers, event values, timeouts, or output fields are rejected or reported without affecting built-in CLI behavior.
Trace: R-154 → BG-109 → SF-141 → UC-144 → FR-139 → QAS-150 → DDR-17 → DD-164. GitHub #12, 2026-10-19.

### UC-146 Review and finalize a completed candidate
After tasks and combined tests finish, open the latest diff and test evidence, approve that reviewed candidate, and apply or merge it. While applying, follow the persistent merge stage and primary status banner; when complete, use the return-to-workspace action to continue the goal.
Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166. GitHub #50, user, 2026-10-20.

### UC-147 Run concurrent tasks while the state database is briefly busy
Several independent task processes write progress to one run database. A writer encountering a transient lock waits within the configured bound and records its state after the lock clears. If the lock outlasts that bound, the task reports the database error and retains all committed evidence.
Trace: R-157 → BG-112 → SF-144 → UC-147 → FR-142 → QAS-153 → DDR-18 → DD-167. GitHub #56.

### UC-148 Register a declarative custom CLI profile
An operator configures an optional catalog discovery command and its response pointers, argument groups with allowed/default/model-specific values, and result extraction/error matching. They save/export and restore the workspace, assign task overrides, resume a task, and reassign it to verify that settings persist and are revalidated. Unset optional arguments are omitted; built-in CLI profiles retain their existing execution behavior.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62.
### UC-148 Configure and run an additional CLI agent
An operator edits or imports a custom agent profile, enters model identifiers manually or configures an explicit model-list command and JSON mapping, and defines argument groups with defaults and model-specific values. During plan review, task options inherit profile defaults or select valid overrides. The runner omits a group with no resolved value, preserves chosen options when resuming its exact session, and validates them again before an agent reassignment. The operator configures the final-result JSON Pointer and a value condition that marks error results. Profile and workspace export/import retain this configuration; Claude, Codex, and OpenCode keep their established adapters.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62, 2026-10-20.

### UC-149 Judge a quiet task from the Agent Room
An operator opens a running task that has printed nothing for minutes. The task header shows the run state, the tool in progress and how long it has run, the last output time with its age, and the process state. A tool result clears the tool line; a missing process or connection error is stated plainly. No wording claims the agent is thinking unless the CLI reported reasoning.
Trace: R-159 → BG-114 → SF-146 → UC-149 → FR-144/NFR-54 → QAS-155 → DDR-20 → DD-169. GitHub #100, user, 2026-10-21.

### UC-150 Watch a multi-session run without re-selecting
An operator watches the running session. When it completes and another starts, the activity panel moves to the new one. When all sessions finish and the run waits for approval, the integration entry opens with the tests and decisions to review.
Trace: R-160 → BG-115 → SF-147 → UC-150 → FR-145 → QAS-156 → DDR-21 → DD-170. GitHub #109, user, 2026-10-21.

### UC-152 Approve a wider scope after an out-of-scope failure
A run fails with "out-of-scope change". The operator opens **Review recovery plan**, reads the offending files and quarantined files, adjusts owned paths and the test command, previews the change and the restore list, approves it, then resumes. Integration verification runs again and the run reaches review only if it passes.
Trace: R-162 → BG-117 → SF-149 → UC-152 → FR-147/NFR-56 → QAS-158 → DDR-23 → DD-172. GitHub #122, user, 2026-10-22.

### UC-153 A task fails overnight and the run still reaches review
An agent exits with an error, or asks a question and exits without a result. The coordinator reads the task's output, writes scoped repair instructions, and the task runs again with them. The operator finds the run waiting for approval; if three rounds could not repair it, the run shows the coordinator's explanation and asks for a decision.
Trace: R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173. GitHub #135, user, 2026-10-22.

### UC-154 A later task benefits from an earlier repair
The coordinator repairs a failing task with scoped instructions. The operator opens the Agent Room, selects any task and reads the lesson (what failed, what was done). The next task prompt contains it, and an agent that recalls from agentmemory finds it there.
Trace: R-164 → BG-119 → SF-151 → UC-154 → FR-149/NFR-58 → QAS-160 → DDR-25 → DD-174. GitHub #138, user, 2026-10-22.

### UC-155 Work with the detail column folded and still be called for a decision
An operator folds the detail column while agents work. When the integration passes and the run waits for approval, the column opens with the candidate review on top. The operator folds it again to read the diff; the strip's dot keeps blinking until the operator approves or rejects, then the column returns to folded, and stays folded after a restart.
Trace: R-165 → BG-120 → SF-152 → UC-155 → FR-150/NFR-59 → QAS-161 → DDR-26 → DD-175. GitHub #146, user, 2026-10-22.

### UC-156 A block is decided and the run continues
A test command cannot run, a task needs a file outside its paths or two tasks collide. The coordinator names the violation, proposes an amendment, the validator accepts it, it is applied and shown with an undo, and the run continues.
Trace: R-166 → BG-121 → SF-153 → UC-156 → FR-151/NFR-60 → QAS-162 → DDR-27 → DD-176. GitHub #149, user, 2026-10-22.

### UC-157 The operator audits what the coordinator decided
The operator sees a count on the coordinator row, opens the coordinator view or a task's activity and reads each judgment with its situation and result, including those that were applied automatically.
Trace: R-167 → BG-122 → SF-154 → UC-157 → FR-152/NFR-61 → QAS-163 → DDR-28 → DD-177. GitHub #149, user, 2026-10-22.

### UC-158 The operator learns before launch that a test cannot run
The operator approves a plan whose test command needs a tool that is not installed. Starting the run is refused with the task, the missing tool and the code, so the operator or the coordinator can fix the plan instead of watching a run fail and resume.
Trace: R-168 → BG-121 → SF-155 → UC-158 → FR-153/NFR-62 → QAS-164 → DDR-27 → DD-178. GitHub #149, user, 2026-10-22.

### UC-159 A task needs one more file and the run does not stop
A task writes a file next to its own files and fails with an out-of-scope change. The coordinator adds that file to the task's paths, the amendment is applied and listed with an Undo, and the run resumes and reaches review. If the coordinator asks for a whole top-level directory or a key file, the run stops with the reason instead.
Trace: R-169 → BG-121 → SF-156 → UC-159 → FR-154/NFR-63 → QAS-165 → DDR-29 → DD-179. GitHub #149, user, 2026-10-22.

### UC-160 The coordinator keeps widening scopes and is stopped
After a few amendments the plan owns many more paths than the charter. The next amendment is refused with goal_drift, the run stops, and the operator sees the charter, each applied amendment with its drift and the proposal that was stopped, then decides.
Trace: R-170 → BG-121 → SF-157 → UC-160 → FR-155/NFR-64 → QAS-166 → DDR-30 → DD-180. GitHub #149, user, 2026-10-22.

### UC-161 Verify worker and process lifecycle on a loaded runner
The test starts an asynchronous worker, waits for its ready marker and active session state, then issues a directive. For process timeout coverage, it waits for the child PID marker before checking that the owned process tree is gone. If readiness is not observed by the deadline, the test reports whether the process exited or timed out before publishing evidence.
Trace: R-171 → BG-123 → SF-158 → UC-161 → FR-156/NFR-65 → QAS-167 → DDR-31 → DD-181. GitHub #156, 2026-10-22.

### UC-162 The operator asks the coordinator to reconcile a failed integration
The development agent reports passing tests but the run shows an integration failure. The operator selects the coordinator, types what to investigate and sends it. The request is shown as processing, the coordinator's answer and any revisions appear in the judgment list, and finished tasks are not repeated unless the coordinator found them wrong.
Trace: R-172 → BG-124 → SF-159 → UC-162 → FR-157/NFR-66 → QAS-168 → DDR-32 → DD-182. GitHub #165, user, 2026-10-23.
