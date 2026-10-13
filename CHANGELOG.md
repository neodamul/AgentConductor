# Changelog

Releases for macOS (Apple silicon) and Windows (x86_64). Stable versions start at
0.1.0 and continue with 0.1.1. Historical preview entries are retained. Each section is published as
the GitHub Release notes for its tag.

## v0.1.16
Includes every change prepared for v0.1.15, which was not published.

- Workspace setup steps 1, 2 and 3 are tabs. Switching preserves drafts, supports keyboard navigation and opens plan review when planning starts or a plan is loaded.
- The startup page no longer stops loading partway (a leftover handler for the removed delete-dialog name field), so Escape and outside clicks close the workspace menu again.
- Agent CLIs run with full access by default: Claude without permission prompts and with the operator's own MCP servers (for example Playwright, so a verifier can open the web UI), Codex without approvals or sandbox. Only the standard claude and codex executables get it; the coordinator still plans and reviews read-only.
- When every task completed but integration verification failed (a failing test or a report recording a blocking issue), the coordinator reviews the evidence, revises the tasks that must change and the run resumes, for up to three rounds; Resume on such a failed run starts it. Findings the run cannot resolve are recorded as deferred follow-ups.
- Interrupted runs are recovered and resumed automatically when a workspace or the Agent Room opens (on by default; at most two retries without progress).
- A phased run records its goal phase when it starts, so a run whose process ended early still completes its phase in Goal and progress.
- The workspace menu closes when you click anywhere outside it (as with Escape).
- Open existing workspace lists workspaces only (no Delete button breaking the rows); a finished plan whose private folder copy was removed no longer blocks deleting or opening its workspace.
- The workspace list shows only workspaces the app manages, so every listed one can be opened and deleted; folders left by the Windows desktop test no longer appear (the test now uses its own HOME).
- Deleting a workspace no longer asks for a new workspace name: the most recently used other workspace opens, so several can be deleted in a row; deleting the last one leaves one empty workspace.
- Review the plan keeps each phase's tasks inside that phase; a plan whose phase already completed is no longer shown as under review.
- New ordinary-folder executions honor the selected project directory: shared task files and results are created there instead of in home-directory folder snapshots. Planning remains read-only; baseline commits and all Git metadata remain outside the project. Accept finalizes the result; Reject restores owned files with conflict checks. Existing runs and pre-existing Git metadata are preserved.

- Jira/Confluence document preview: discovered tools can be connected through Use for Jira / Use for Confluence with editable document arguments. Missing document-reader bindings now open configuration with clear guidance instead of making a registered MCP server appear missing.
- Interface wording is managed by stable message IDs in separate English/Korean JSON catalogs, including native app errors, menus and picker captions. Language changes preserve typed values, raw evidence and icons; missing translations fall back to English.
- Windows: Claude usage loads when Claude asks "Enter y/n:" to trust the project folder (screen-reader form); the query now answers y instead of Enter, which left it waiting until it timed out, and stops with guidance if Claude keeps asking.
- Agent Room status: a run status banner shows the state in bold colour, task and phase progress, why a run failed (the failed task and its test output) and the next step with its action (Review changes → Approve → Apply changes → Back to workspace); task states are bold and coloured, failed tasks are red and tasks waiting for a decision blink in the task list, which no longer shows usage badges.
- The coordinator is listed after the tasks as its own entry (integration verification); selecting it shows its run steps, tool decisions and integration test output. The interface uses the single name coordinator.
- Fix with feedback on a failed run selects the failed task and drafts a direction with the failure output.
- A verification report changed in the run that records a blocking issue (a Blocking/차단 section) now fails integration verification for the task responsible, instead of letting the run complete.
- Phased goals: Review the plan shows every phase, which one is under review and Plan phase n for the next one; a finished run says which phase completed and which is next. A completed phase is recorded even after its goal folder moved.
- Goal folders are created directly in the project folder as <project>/<goal>/ (for example C:\Temp\ladder\ladder_and_priority_games) instead of <project>/.agentconductor/<goal>/; existing ones move there when first used.
- Tool requests: the coordinator decides routine requests outside the workspace sandbox (also for runs started before v0.1.13) and each decision is listed in Activity with its reason; commands in a workspace stored under ~/.agentconductor no longer ask for their own folder. Allow once and Deny work at once in the request window while the run is working.
- Activity shows agent output like the CLI (messages, tool lines, result summaries); raw JSON stays in Raw logs.
- Agent CLI list: kept as one list for all workspaces (~/.agentconductor/agents.json) and not removed with a workspace; assignments to agents that are no longer listed are red and block Generate plan until reassigned; cards reorder by drag and drop or Move up/Move down, and every agent selection follows that order.
- Paths are shown without the Windows \\?\ prefix everywhere in the app and in messenger notifications.

## v0.1.14

Includes every v0.1.13 change below; v0.1.13 was not published because its macOS check failed.

- The macOS approval tests now match the workspace sandbox policy introduced in v0.1.13; leftover translations of the removed permission menu are gone.
- Generate plan shows "Preparing the project folder for planning…" with elapsed time as soon as it is pressed, instead of only "Workspace saved locally." while the folder is copied before the coordinator starts.
- Saving Workspace settings updates the workspace name in the titlebar at once; before, a renamed workspace (for example one created as "New workspace") kept showing its old name until the workspace menu was opened.
- Deleting the current workspace asks for the name of the fresh workspace that replaces it, instead of always naming it "New workspace".
- Claude usage loads again: the usage query accepts Claude's "Do you trust the files in this folder?" prompt (the query session has no tools, hooks or project settings) instead of stopping with an approval error; it stops with guidance if Claude keeps asking.
- The permission capability menu is removed from agent profiles; it follows from the CLI (Codex runs in its workspace sandbox, Claude behind the workspace policy). Existing workspaces are updated when opened.
- Claude tasks no longer stop for routine approvals: commands and searches confined to the workspace (builds, tests, `git status`/`diff`/`log`, file listings) run at once. Requests reaching outside the workspace (network, package installs, commit/push, privilege, outside paths, secrets) are decided by the workspace coordinator as orchestrator; a person is asked only when it cannot decide. A request the policy refuses denies that call only instead of ending the task.
- Time spent waiting for a tool decision no longer counts toward the 30-minute task limit, and a task that times out is blocked and continues on Resume instead of failing.

## v0.1.12

- Windows: Claude tasks can read and edit workspace files again instead of stopping with "control channel failed; owned process group terminated: tool path must be an unambiguous bounded workspace-relative path"; `C:\…`, `C:/…` and `\\?\C:\…` paths inside the workspace (any letter case, `\` or `/`) are checked by the same scope, traversal and protected-folder rules.
- Windows: an approved tool request no longer fails with "tool context verifier failed" while agents are busy; the pre-execution check of the approval allows 30 s instead of 10 s (it starts about a dozen Git processes, which took up to 6 s on an idle Windows machine).
- Tool requests read in plain terms: the action (Run a command, Edit a file, Read a file, …) with labelled inputs such as the command and its purpose, the file, or the text replaced and its replacement; the exact JSON stays under Raw input.
- Approving a tool request while the run is not running continues the run once the approval is recorded, and the approval window's button reads Close after a decision.
- The Agent Room uses the Workspace titlebar: the workspace menu and Settings stay available while a run is open (instead of the run's project path), and ← Workspace returns to the workspace screen.

## v0.1.11

- Preserve one external signal attempt across restart; uncertain delivery retains the local directive without replay, and a kernel-owned lease prevents concurrent publications.
- Settings → Signal transport chooses Local Rust (default) or agentmemory preferred for cooperative directives; when agentmemory is unavailable the same directive is delivered locally, and Settings and the briefing show the effective route and the reason.
- Windows: Generate plan, tasks and model discovery start npm-installed agents again instead of failing with "spawn agent: program not found" (`claude`) or os error 193 (`...\npm\codex`); bare names and npm shell shims resolve to their `.cmd` launcher.
- Windows: Approve plan & run starts Codex `native_sandbox` tasks whose executable is a full path such as `...\npm\codex` instead of failing with "native_sandbox requires the verified Codex initial workspace-write invocation"; the Codex read-only coordinator and Claude callback checks accept such paths too, and profile checks recognize `cmd.exe` and `powershell.exe` as shells.
- Two agents can be each other's fallback (for example Claude → Codex and Codex → Claude) instead of being rejected as a "cyclic fallback policy"; when every account on the chain is limited, the work waits for availability.
- Generate plan for a goal that already has a phased plan loads its current phase for review with the current agent profiles instead of failing with "this slug already has a plan".
- Agent Room session cards no longer wrap the task name one letter per line beside the quota badge: the quota sits above the agent name right of the icon, task and status use the full width below it, and the icon shows the status as a coloured border that pulses while running.
- A tool request waiting for your decision opens a window with the request, the time left before the worker stops and the task's live activity log; Allow once and Deny decide it there, Decide later keeps it in Tool requests, and the log keeps updating whatever is decided.
- Feedback sent to a run that is not running continues the run after the revision is recorded, so the agent answers instead of the run staying idle.

## v0.1.10

- Generate plan accepts phased plans whose first task depends on a task of an earlier phase instead of failing with "unknown dependency"; phases already run in order. An unknown dependency error now names the task and the missing ID.
- Generate plan accepts directory scopes written with a trailing slash (for example `docs/`) instead of failing with "unsafe or overlapping scope".
- Show the running build version beside the app title in Workspace and Agent Room.
- Bound completed disposable metadata cache at idle startup and after refreshes; add Settings -> Clear cache with a reviewed permanent-deletion preview.
- Stop a blocked Generate plan operation before deleting its current workspace; open a fresh workspace in the same window and preserve project sources and recoverable prior app data.

## v0.1.9

- Generate plan accepts a coordinator plan wrapped in a fenced ```json block instead of failing with "coordinator response contains no valid JSON tasks document".
- Windows: the C runtime is linked statically, so the app starts without the Visual C++ Redistributable (VCRUNTIME140.dll).

## v0.1.8

- Preserve each Claude model-specific weekly window alongside the all-models total.
- Explain missing project folders and unsupported Codex API-key quota queries before retrying.

- Save edited agent profiles asynchronously; show errors inside the dialog and close after success.

- Show normal Windows drive/UNC project paths and preserve drive-root separators.
- Create named workspaces by Enter or Create, waiting asynchronously for admitted metadata.
- Add confirmed workspace deletion with recoverable local app data and project-source protection.
- Name generic MCP server registration correctly in Settings and update the user manual.

## v0.1.7

- Generate plan no longer shows "Native response timed out" from the background status refresh while a large folder is being prepared.
- Windows: Claude capacity refresh runs the CLI in the project folder (previously cmd.exe fell back to System32) and reports "first-run setup is not finished" immediately instead of timing out; nothing is answered on the operator's behalf.

## v0.1.6

- Goals in other languages get an English slug proposed by the coordinator (for example `ladder_game`) instead of a romanized name.
- Enterprise capacity is shown as a percentage, like other windows (for example `64%/100%`); the tooltip keeps the raw amounts.
- Windows: a project folder picked with Browse, including Korean folder names, now saves like a typed path.
- Generate plan no longer shows a false timeout while preparing a large folder; the review area shows the coordinator's real output activity and warns when it stalls. The progress light blinks even with reduced motion.
- Settings → Agent tools guide… lists install commands for agentmemory, ponytail, rtk, graphify and cocoindex-code for Claude Code and Codex. Guidance only; the app installs and runs nothing.

## v0.1.5

- Plan and execute in ordinary project folders without initializing their Git repository. Private bounded change history supports reviewed diffs, tests and approved application; concurrent original edits are refused.
- Keep agent registration and edit forms responsive while executable, model and quota metadata is loading; preserve operator typing and profile changes.
- Run model and quota queries through one bounded background worker with a durable queue instead of locking the planning slot or unrelated controls.
- Avoid skipping another agent's quota refresh or consuming a cooldown when its refresh request was not accepted.
- Preserve account capacity when model catalogs refresh; catalog observations do not change account identity.
- Settle failed metadata without blocking settings; prune superseded private snapshots and compact completed working copies while retaining review evidence.
- Update the English user manual and README for workspace setup, asynchronous controls and ordinary-folder review/application.

## v0.1.4

- Open macOS folder and file pickers in the desktop app so project selection stays in front.
- Remove automatic Windows command windows from desktop startup and background Git, agent, metadata and messenger operations; retain CLI output and process containment.
- Display Codex Enterprise remaining allocation / allocated total; preserve unknown totals and individual scope.
- Reduce Settings button whitespace while preserving the visible gear.

## v0.1.3

- Slugs are named automatically from the goal (Korean is romanized, duplicates get `_2`…); the slug field is gone.
- **Answer tool requests from the messenger** (phase 2): listed approvers reply `allow <code>` / `deny <code>` in Slack, or a company relay returns HMAC-signed decisions; applied through the same exact, single-use approval as the Agent Room. Polling only while a request is waiting; tokens and secrets stay on this computer.

## v0.1.2

- Configure company model IDs directly for custom CLIs without a model-list command, with explicit operator provenance.

- **Messenger notifications** (phase 1): Slack Incoming Webhook or any company messenger webhook per workspace — plan ready, review/tool decisions, usage-limit waits and run outcomes. The webhook URL stays on this computer (never exported); messages are metadata-only unless the goal is allowed; delivery uses the system `curl` without new dependencies.
- Put MCP connection settings under the top-bar Settings menu; enlarge the gear and its pointer target.
- Fix Windows npm executable discovery and recover quota collection for existing extensionless profiles.
- Add a bounded, contained Windows ConPTY collector for Claude /usage; no model generation or automatic trust approval.
- Show original provider badges when desktop artwork is unavailable and repair legacy Windows icon alpha.

- Register OpenCode-shaped local and remote MCP servers in Workspace settings.
- Discover read-only tools and bind Jira/Confluence arguments to explicit goal references.
- Preview source text and provenance, approve selected evidence and retain immutable local snapshots for planning.
- Keep document bodies out of portable settings and shared memory; verify goal/settings/content digests before execution.
- Reuse one MCP session per server, bound responses/deadlines and close owned processes/sessions after each operation.
- Support authenticated local proxies and environment-backed remote headers; native OAuth browser authorization is not included.

## v0.1.1

### Added

- Goals become slugs with work folders; plans start with an analysis and run phase by phase (task → verification → completed); **Goals & progress** overview, **Plan next phase** and editable **Plan rules**.
- Slug folders live in `<repo>/.agentconductor/<slug>/` (or `~/.agentconductor/<workspace>/<slug>/` without a repository) and are kept out of `git status`.
- Repository published under the MIT license.
- Agent dialog: **Provider** comes first; switching provider replaces the Agent ID (unique suffix), executable path and options together and clears the old CLI's models. Renaming an agent keeps it as coordinator, in role rules and as fallback.
- **Settings** menu with interface language: English (default) or Korean; switches instantly and is remembered. UTF-8 is explicit everywhere (pages, release notes, Windows runners).
- Windows fixes found by the first real Windows CI run: release notes no longer fail under the cp1252 console encoding; executable lookup also searches `%APPDATA%\npm` and `%USERPROFILE%\.local\bin`.
- Agent dialog: choosing an alternate agent loads its models into **Alternate agent's model**, with an explanation of what the fallback does; the Settings button is a ⚙ icon.
- Exhausted quota shows when it returns, e.g. `0%/40% · 18:39`; the tooltip lists every window's reset time.

## v0.1.0

- Default quota refresh to five minutes, configurable per agent; update capacity silently in the background.
- Locate installed CLI executables automatically, provide native Browse, and standardize input control sizing.
- Use the project-owned AC mark for native application and executable identity.
- Clarify the three workspace steps and update actual native screenshots and the user manual.

### Added

- AgentConductor is published under the MIT license; `LICENSE` ships in every package.
- New workspaces start with an empty agent list; add the CLIs you use with **+ Add** (templates for Claude Code, Codex and OpenCode remain).
- Lower idle cost: windows refresh every 5 s when nothing is happening (1 s while work runs), pause while hidden, and no longer resend app icons every second (idle CPU ≈0.1 %).
- Workspace export now includes one `<slug>.zip` per goal with its progress; import verifies every archive before restoring.
- Quota collection in fresh workspaces, individual refresh icons and automatic two-minute
  refresh while idle; authenticated macOS Claude `/usage` metadata without generation.
- Compact Edit, Models reload and Remove icons in agent cards.

- Remaining quota beside paid agent names in the workspace and session lists, with
  window/source tooltips and explicit unavailable or stale states.

- Claude capacity in Agent Room: automatically collect provider-reported utilization and
  reset windows from owned task streams. **Check observed quota** preserves freshness
  without launching another prompt; absent utilization stays unavailable.

- When planning starts on a repository with uncommitted or untracked changes, a dialog now
  explains why a clean baseline is needed, lists the changed files and offers **Commit and
  continue** (one baseline commit on the current branch) or **Stop** (nothing changes).
- **Ignore and continue** in the same dialog for operating-system files (`.DS_Store`,
  `Thumbs.db`, `desktop.ini`, `._*`): excludes them locally without a commit, with a guide
  to ignore them in every repository.

- Planning progress: **Review the plan** shows the coordinator and elapsed time while a
  plan is generated, a *Plan ready* notice with scroll when done, and the exit status and
  log folder when planning fails.

- Task instructions are requested as readable *Goal / Inputs / Steps / Expected output /
  Done when* sections in the goal's language; the instructions box is twice as tall and
  grows with its content.

- **Agent roles** (step 2): decide per workspace which agent does verification,
  development, documentation, knowledge assetization and testing; the coordinator must
  follow these rules, and each planned task shows its role. Each role agent can be pinned
  to a model from its observed catalog; the engine applies that model to the task.
- The workspace screen is now numbered: **1. Workspace settings**, **2. Agent roles**,
  **3. Review the plan**.

- Agents added, edited or removed are saved immediately (other unsaved edits are kept
  as typed); removing an agent also updates the coordinator and role rules.

- Model lists load automatically once when a built-in agent is added; **Models** refreshes
  them. Lists update quietly in the role table without moving focus.
- Compact agent cards: provider, sessions, model count, permission capability and fallback
  at a glance.

- **Export settings… / Import settings…** in the Workspace menu: share name, coordinator,
  workspace mode, agents and role rules through `agentconductor.workspace.json` in the
  repository. No secrets, model lists, local paths, goals or plans are exported; imported
  sandbox claims must be re-confirmed on each machine.

### Fixed

- New workspaces showed no model lists; they now reuse lists already observed on this
  machine and fetch missing ones once in the background.
- Saving plan edits dropped each task's role.
- A generated plan was discarded when models were loaded or the workspace was saved with
  unrelated changes; it is now kept unless the goal, repository, workspace mode or an
  agent's execution settings change.

- Plans can have one to twelve tasks; **+ Add task** and **Remove task** in review.
- **Read plan…**: convert an existing Markdown plan document into tasks with the
  coordinator (read-only, role rules applied, completed items skipped).
- **Save workspace** moved into step 1.

### Changed

- **Open existing workspace…** now lists the workspaces saved in `~/.agentconductor`
  (including ones removed from the menu) instead of opening a folder dialog, so an
  arbitrary folder can no longer become a workspace by mistake.

## v0.1.0-preview.6

### Added

- **Write plan manually** — plan the three tasks yourself without any LLM coordinator; the
  draft uses the same validation and exact-digest approval as a generated plan.
- **Workspace name** above the goal, and a **Workspace** menu in the title bar: see every
  registered workspace by name, create one with **New workspace…** (name only), add an
  existing folder, switch in place, or remove an entry from the list (files are kept).
- **Workspace settings** section: goal first, then planning coordinator, then repository
  with a **Browse…** folder dialog next to the Git-worktree option.
- A workspace can be saved without a repository; only planning needs one.
- Release notes are now taken from this changelog.

### Fixed

- README and manual named the agent button "Register agent"; the button is **+ Add**.

### Known limitations

- Codex live re-verification waits for the account usage reset (2026-10-16).
- Claude account-capacity refresh is unavailable (no provider channel).

## v0.1.0-preview.5

### Added

- `memory-discover RUN PORT` finds and imports cooperative checkpoint reports from a local
  agentmemory server without memory IDs, scoped to each task's current address.

### Fixed

- Windows containment test no longer times out on a cold PowerShell start (test only).

## v0.1.0-preview.4

### Added

- A completed cooperative **handoff** now continues automatically on the agent's reviewed
  fallback once the relinquishing process has exited; without a fallback it stops as before.

### Verified

- Live Claude workers: an out-of-scope write is blocked before it happens; a rejected
  command never runs and that worker continues.
- Merge is refused when the target branch moved after approval.

## v0.1.0-preview.3

### Changed

- README rewritten as a short desktop-first introduction: what it does, how orchestration
  works, how roles are assigned, usage-limit handling and what you need.
- New user manual (`docs/user-manual.md`) with the desktop guide and every command; shipped
  in the package.

### Added

- Windows: provider icons are also found through Start Menu shortcuts.

## v0.1.0-preview.2

### Fixed

- Checksums and license notices are written with LF line endings on every platform, so
  `SHA256SUMS` verifies on macOS and Windows.

## v0.1.0-preview.1

### Added

- First pre-release: native desktop workspace and Agent Room for Claude Code, Codex,
  OpenCode and custom CLIs; three-task plans with exact approval; parallel sessions in a
  shared directory or Git worktrees; live tool approval; usage-limit waiting and reviewed
  fallback; reviewed lessons; macOS and Windows packages with licenses, SBOM and checksums.

- Compact agent quota groups show slash-separated percentages on one line beside their refresh icon; tooltips explain Enterprise Current/Total or provider duration labels.
