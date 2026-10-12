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

### UC-52 Resolve shorthand safely (planned)

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
