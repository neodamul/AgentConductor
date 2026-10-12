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

A clean Windows checkout can build the supported target without macOS absolute paths. Owned process descendants stop on timeout/cancel/crash; repository leases prevent concurrent writers; secret/alias boundaries remain enforced. Platform release smoke tests operate on the exact packaged binaries. On Windows with HOME absent, a no-argument launch resolves the default workspace from USERPROFILE and opens the native startup window without a local UI server.

### QAS-19 Policy fallback without live steering

Given a restricted paid account and reviewed self-hosted fallback, no running attempt is steered or overlapped during transfer. A fresh target session receives bounded owned-path evidence; source quota remains blocked after transfer. Unknown targets/models, cycles and same-account fallbacks fail before spawn. A stopped run never launches a fallback. Failed tests never become approval. Ordinary errors or silence do not trigger quota fallback. Trace: UC-18.

### QAS-20 Trustworthy status briefing

Given concurrent workers, a stale agentmemory report and a terminated local process, a status query identifies the process termination and stale report rather than claiming ongoing work. Every statement of progress has a timestamp/source; unknown information is not guessed. The query changes no task revision, permissions or approval. Trace: UC-19.

### QAS-21 Visible and accessible status

Given simultaneous running, waiting and failed tasks, the list shows blinking green, steady yellow and steady red with matching text. Polling does not reset the active pulse on every refresh. Unknown states stay yellow. Reduced-motion preference disables animation without losing state information. Trace: UC-20.

### QAS-22 Cooperative instruction acknowledgment

Given a non-steerable worker at a checkpoint and a valid addressed directive, it reports acceptance and applies the permitted change exactly once. Duplicate, expired, foreign-session or unauthorized directives are rejected with retained evidence. Memory-server unavailability does not fabricate acknowledgment or authorize a competing writer. An ordinary memory observation is never executable direction. A completed handoff continues automatically only on the reviewed fallback target after owned process absence is proven, preserving partial work and requiring fresh tests and approval; without one the task stays stopped. Trace: UC-21.

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

### QAS-28 First-contact documentation

Given the README only, a first-time operator can name the product's purpose and start the desktop workflow: the README shows real screenshots, install and launch steps and a desktop walkthrough, contains no shell command blocks and stays under 120 lines. It has sections answering how orchestration works, how roles are assigned, what happens at usage limits and what to prepare, and states that agentmemory is optional. Every command listed by `agentconductor --help` is documented in the user manual, which the README links and release packages include. Trace: UC-27.

### QAS-29 Planning without a coordinator

Given registered agents, a clean repository and a goal, **Write plan manually** produces a three-task draft immediately, launches no process and contacts no LLM. Saving an incomplete or overlapping draft is refused; a completed draft is approved by its exact digest and runs; a repository change after drafting refuses approval. Trace: UC-28.

### QAS-30 Repository selection without typing

Given the workspace screen, **Browse…** opens the platform folder dialog (macOS `choose folder`, Windows `FolderBrowserDialog`) without blocking the window's request thread; a selection fills the field, cancel changes nothing. A workspace with an empty repository saves; plan creation without one is refused. Trace: UC-29.

### QAS-31 In-place workspace switching

Given two workspaces, selecting another from the menu reloads the window on it within one request, records it as recent, and releases the previous window lease so the old workspace can be opened again. Switching to a workspace held by another window, or while a planning job or run view is active, is refused and the current workspace stays open. Trace: UC-30.

### QAS-32 Named workspace registry

Given one workspace, entering a name and saving shows it in the title bar and menu; **New workspace…** with a name creates a distinct folder under `~/.agentconductor/workspaces/`, switches to it and lists it by name; names longer than 80 characters or with control characters are refused. Removing a registered workspace leaves its folder intact; removing the current one is refused. **Open existing workspace…** lists exactly the folders under `~/.agentconductor` (`desktop`, `workspaces/*`) that contain `startup.json`, by name, and opening one re-registers it; a folder without saved settings cannot be opened from this list. Trace: UC-31.

### QAS-33 Release notes match the version

Given a version tag, the published release body is exactly that version's `CHANGELOG.md` section (features, fixes, known limitations); a tag without a section, or with an empty one, fails before `gh release create`. Trace: UC-32.

### QAS-34 No silent baseline commit

Given a repository with modified and untracked files, planning shows the reason and the changed files and creates no commit until **Commit and continue**; then exactly one commit containing all of them is created on the current branch and planning proceeds from it. **Stop** changes nothing. A stale workspace revision or a failing `git commit` creates no plan. Trace: UC-33.

### QAS-35 Messenger notification without data leakage (planned)

Given a workspace linked to a channel, a pending tool request produces one message within 10 seconds containing only the workspace name, task ID, agent, request kind and a redacted summary; no file contents, diffs, tokens or environment values are sent unless the workspace policy allows it. With the messenger unreachable, the desktop flow is unaffected and delivery is retried with a bound. Trace: UC-34.

### QAS-36 Messenger decision integrity (planned)

Given a pending request, a decision from the messenger is applied only when the payload signature is valid, the sender is a linked approver for that workspace, the request ID and digest match the pending request and it has not expired or already been decided; replays and unlinked users are refused and audited. Trace: UC-35.

### QAS-37 Ignoring OS files changes no tracked content

Given untracked `.DS_Store` files (nested included) and optionally other changes, **Ignore and continue** appends each missing pattern once to `$(git rev-parse --git-path info/exclude)`, creates no commit and modifies no tracked file; the repository is then clean, or only the other changes remain. Tracked metadata files and non-OS files are never excluded. Trace: UC-36.

### QAS-38 Planning is never silent

Given **Generate plan**, within one refresh (1 s) the review area shows the coordinator and an elapsed time derived from the job's recorded start, updating each second; on completion a ready notice appears and the plan section scrolls into view; on failure the area shows the exit status and the job's log directory. A window reopened during planning shows the same elapsed time. Trace: UC-37.

### QAS-39 Readable instructions

Given a goal, the planning prompt requires the five labelled sections on separate lines with `- ` bullets and the goal's language; the instruction editor opens with at least six rows and grows to the content (up to 30 rows) as text is typed. Trace: UC-38.

### QAS-40 Plans follow the operator's role rules

Given role rules, the planning prompt lists each role with its ordered agents; every generated task carries a role from the rule set and an agent from that role's list, otherwise planning fails naming the task and role and no proposal is shown. Unknown roles, unregistered agents or models outside the agent's observed catalog are refused on save; when a rule names a model, every task assigned through that entry runs with exactly that model. Without rules, planning behaves as before. The screen shows the headings 1 Workspace settings, 2 Agent roles and 3 Review the plan in that order. Trace: UC-39.

### QAS-41 Registered agents survive a restart

Given a saved workspace, **Use profile** for a new agent stores it before the dialog closes; reopening the workspace lists it, and the saved goal is unchanged by this action. Removing the coordinator agent sets the coordinator to the first remaining agent; role entries naming it are removed; an invalid profile or stale revision stores nothing. Trace: UC-40.

### QAS-42 Model lists are ready without manual queries

Given a newly added built-in agent, or a workspace just created or switched to, built-in agents without a catalog are discovered once per page session; a new workspace first copies catalogs observed in the previous workspace for the same parser and executable; **Models** refreshes an agent's catalog; existing agents are not queried on open; model options in the role table update in place without changing focus or the selected values; no discovery starts while a plan is being generated or started or edits are unsaved, and a reviewed plan remains approvable after models load. Saving a workspace whose submitted profiles omit a catalog keeps the stored catalog for the same agent ID, parser and executable. Trace: UC-41.

### QAS-43 A plan survives unrelated changes

Given a ready plan, saving a new workspace name or role rules, or importing/refreshing a catalog keeps the proposal and its approval path; changing the goal, repository, workspace mode or an agent's execution settings clears it. Trace: UC-41.

### QAS-44 Exported settings are safe to commit

Given a workspace with private environment values, catalogs and a repository, the export contains `format: agentconductor.workspace/v1`, the name, coordinator, mode, agents without `environment` or `catalog`, role rules and `repository: "."` only when exported into the repository root; output is stable for the same settings and ends with LF. Import of that file into another workspace reproduces agents, roles, coordinator, mode and name; keeps local environment values and catalogs for the same agent IDs; sets the repository to the file's folder; and imports `external_sandbox`/`read_only` capabilities as `unverified` unless confirmed. Unknown formats, invalid profiles or a stale revision change nothing. Trace: UC-42.

### QAS-45 Plan size and document import

Plans with 0 or more than 12 tasks are refused; 1 and 12 are accepted. The planning prompt no longer demands three tasks and asks for the smallest disjoint split. **Read plan…** sends the document text (at most 64 KiB, larger files refused) with the role rules to the coordinator running on the read-only snapshot; the generated plan passes the same validation, role checks and approval as a generated one; when the goal is empty the document's first heading becomes the goal. Trace: UC-43.

### QAS-46 Goals accumulate as slugs

Setting a second goal adds a second slug with its own tree; slugs are unique, lowercase `[a-z0-9_]`, at most 48 characters; the repository's `.agentconductor/` is excluded from Git status so planning and merges are unaffected. Trace: UC-44.

### QAS-47 Phases run in order

A phase cannot be approved for execution while the previous phase is not completed; each phase is executed as its own run and its tasks reach `completed/` only after the candidate is merged. Trace: UC-45.

### QAS-48 Progress equals the tree

Progress reported in the window and in the export is computed from file locations; moving a task file by hand changes the reported stage or, if impossible (for example a completed task in an incomplete phase), produces a mismatch report; no operation leaves a task in two stage folders. Trace: UC-46.

### QAS-49 Archives are verified on import

Export writes one store-only zip per slug with entries only under that slug; import refuses missing archives, unexpected names, entries escaping the slug folder, or a JSON/archive progress disagreement, and changes nothing in that case. Trace: UC-47.

### QAS-50 Plan rules reach the coordinator

The planning prompt contains the effective rules — the workspace override when set, otherwise the global file — and the global file is created with the default rules on first use and kept when edited. Trace: UC-48.

### QAS-51 Claude quota is evidence, not inferred capacity

A valid owned stream event updates the board before worker completion. Missing utilization, malformed values and unrelated costs do not become a percentage. A new event replaces prior window evidence rather than refreshing omitted windows. Refresh preserves observation timestamps and execution revision, creates no process and reads no credentials. Profile changes invalidate retained metadata. Trace: UC-49.

### QAS-52 — A reported 73% renders beside the agent name in both lists; multiple reported windows use the lowest remaining value. Missing values render an em dash; stale values render Stale. Names can truncate without hiding the badge. Workspace reads only its retained run and validates the current profile digest. Trace: UC-50.

### QAS-53 MCP registration compatibility

Local entries use `type`, argv-array `command`, optional `cwd`, `environment`, `enabled`, `timeout`; remote entries use `type`, `url`, optional `headers`, `oauth`, `enabled`, `timeout`. OpenCode-compatible JSON/JSONC imports preserve these objects without injecting provider labels. Goal-source bindings are separate application metadata. Unsupported configuration produces a clear validation error.

### QAS-54 Traceable import before planning

Only confirmed IDs on configured bindings are fetched. Capture source ID, server, URL, version/update time, retrieval time and content digest; the preview reports failures and inaccessible pages. A reviewed plan cites imported snapshots, not changing live records. Missing required Story/pages cannot silently produce a complete-context plan. Source changes require explicit refresh and re-review.

### QAS-55 Imported content has no execution authority

Read-only retrieval tools cannot create/update Jira issues or Confluence pages. Imported text cannot change local planning rules, approval gates or tool permissions. Secrets are redacted and excluded from settings export; document bodies remain local and are not automatically published into shared agentmemory. Trace: UC-51, UC-52.

### QAS-56 Independent quota refresh

A fresh workspace queries each supported provider without an execution run. Refreshing agent A does not reset B's two-minute deadline; failures do not remove B from the queue. Query processes are owned, bounded and reaped, and rendering never starts inference. Badge icons have provider-specific accessible labels and indicate the current refresh. Models/Edit/Remove keep their action handlers and accessible names. Trace: UC-53.

### QAS-58 Quiet idle window

Given an open workspace or Agent Room with no running job, unsaved edit or planning in progress, the page refreshes at most every 5 s, not at all while the window is hidden, and snapshots omit application icons unless the page asks for them once or after the agent kinds change; with a job running it refreshes every second. The release build idles at core RSS < 100 MiB and average CPU < 1 % over 30 s. Trace: R-65, NFR-05.

### QAS-57 Independent window readability

Given remaining values 73% and 36%, an operator sees both values and their window labels in the agent row. The row displays `73%/36% ↻` on one line. The tooltip maps Enterprise to Current/Total and a 300-minute plus plan to 5h/Weekly. A 1440-minute window is Daily; unknown durations retain provider labels. The same group contains the refresh icon. Trace: FR-50 / NFR-24.

### QAS-60 Quiet settings with preserved evidence

Given a reviewed plan and an existing agent, changing cadence from five to ten minutes survives reopening without changing its proposal. Bounds outside 1–60 are refused. Missing tools can be selected through a native file picker; no CLI is launched to locate them. Enabled controls remain usable during idle UI polling. Actual app inspection verifies uniform form sizing and AC identity. Trace: FR-51 / NFR-25.
### QAS-59 Empty agent list by default

Given a new workspace, the snapshot lists no agents, an empty coordinator and the provider templates; saving such a workspace succeeds; generating or writing a plan is refused until an agent is registered and chosen as coordinator. Trace: R-66.

### QAS-61 Clean provider switch

Given a Claude agent `claude` used as coordinator, role agent and Codex's fallback, switching it to OpenCode in Edit yields ID `opencode` (or `opencode-2` when taken), the OpenCode executable and arguments, no Claude models, and after saving the coordinator, role and fallback reference `opencode`. Trace: FR-52 / NFR-26.

### QAS-62 Reset time for exhausted quota

Given windows 0 % (reset today 14:30) and 40 %, the badge reads `0%/40% · 14:30`; a reset on another day includes the date; an exhausted window without a reset time reads `· reset unknown`; non-exhausted quota shows no time. Trace: FR-53.

### QAS-63 Korean interface on demand

Given a fresh machine, the page loads in English. After choosing Korean in Settings, the titlebar, steps and agent dialog read in Korean immediately, an unsaved goal draft remains, a reopened window starts in Korean, and an unknown language code is refused. A goal typed as "Testing" in a textarea is not translated. Release notes and pages declare UTF-8. Trace: FR-54 / NFR-27.

### QAS-64 Fallback model list

Given a fallback agent without models, choosing it shows `Loading models…`, the select is enabled, and after the fetch the models appear without closing the dialog or moving focus; with no fallback agent the select is disabled with `Choose an alternate agent first`. Trace: FR-55.

QAS-58 measurement amendment (NFR-05 clarification): verified 2026-10-11 on the release build, idle workspace window — main physical footprint 36 MB (target < 100 MB), whole window incl. WebKit helpers ≈ 94 MB, CPU 0.00 % over 60 s; RSS for reference 107–112 MiB main, 199 MiB for the four processes.

### QAS-66 Phase 1 notification delivery

Given a workspace with Slack enabled, a run that reaches `waiting_approval` sends exactly one POST whose body is `{"text": …}` with workspace name, event and task counts; a second save of the same state sends nothing; a company webhook receives the template with `{{text}}`, `{{event}}`, `{{workspace}}`, `{{state}}`, `{{goal}}` JSON-escaped; the goal text is absent unless allowed; an `http://` URL is refused; an unreachable endpoint leaves the run state and timing unchanged; the exported settings contain no webhook URL. Trace: FR-28 / NFR-17 / QAS-35.

### QAS-65 Windows metadata and artwork
Given extensionless and .cmd npm launchers, choose .cmd. Given no desktop apps, each built-in provider returns a distinct bounded PNG badge. Given a Windows terminal fixture, send /usage only after the ready prompt, parse the reported windows, reject trust prompts and reap the process tree. Verify the Windows build and tests in CI; actual enterprise subscription acceptance remains separately recorded. Trace: FR-56/NFR-28.


### QAS-67 CLI discovery absent
Given aipro has no /models command, Models explains the limitation and shows existing IDs. Duplicate IDs deduplicate; empty/invalid/oversized lists fail; observed JSON retains original provenance. No CLI command or HTTP request is sent. Trace: FR-57/NFR-29.

### QAS-68 Messenger decisions are exact and authorized

Given a linked run with one pending request `3FA9C21B…`, a Slack reply `allow 3fa9c21b` from a listed approver applies exactly one approval for that request and posts a confirmation; the same reply seen again, a reply from an unlisted user, a bot message, `allow 00000000`, or a reply after the request is no longer pending changes nothing and is logged; a relay decision with a wrong signature is refused; with no pending request no inbound HTTP request is made. Trace: FR-58 / NFR-30 / QAS-36.

### QAS-69 Readable automatic slugs

`Add export API` → `add_export_api`; `헬스체크 주기 조정` → `helseuchekeu_jugi_jojeong`; `결제 API 개선` → `gyeolje_api_gaeseon`; `日本語だけ` on 2026-10-11 → `goal_20261011`; a repeated goal → `_2`; a very long goal ≤ 48 characters without a trailing `_`; the startup page has no slug input. Trace: FR-59.

### QAS-70 Enterprise budgets without rolling windows
Given a CLI response containing individualLimit or credits and null primary/secondary windows, the refresh remains available and renders remaining/allocated. Expired metadata becomes stale; a balance-only response never implies a total allocation or percentage. Windows live-account acceptance stays separate. Trace: FR-60/NFR-31.


### QAS-71 Foreground selection and hidden background execution
Browse opens NSOpenPanel above the active macOS app; cancellation returns no path and retains the existing form. On Windows the executable PE subsystem is GUI and a console fixture launched through the background factory observes no console while stdout/stderr remain intact. Exact packaged workflow smoke still passes. Trace: FR-61/NFR-32.


### QAS-72 Fresh directory settings and metadata remain usable
A plain directory saves and reopens correctly; planning rejects it with actionable guidance before any slug or Git mutation. An unborn repository saves but requires an initial commit before planning. Invalid files preserve settings. Codex fixture quota reaches the fresh workspace with no Git and no run. Trace: FR-62/NFR-33.


### QAS-73 Slow metadata never monopolizes team configuration
A paused OpenCode catalog fixture allows profile registration and workspace Save; another catalog request queues without another collector. Editing the first profile prevents its late result from overwriting it, while the queued second catalog applies. Metadata scheduling preserves edit revision. Actual JavaScript tests prove the dialog opens before lookup, late executable results preserve typing and busy controls are scoped per operation/agent. Trace: FR-63/NFR-34.


### QAS-74 Git setup consent preserves project content
False approval leaves a plain folder without .git; accepted approval creates an empty committed tree and local audit; existing files remain untracked and staged files remain staged; repeated setup is rejected. Actual ensureBaseline JavaScript stops without a write or rechecks and continues after approval. Trace: FR-64/NFR-35.


### QAS-75 Folder planning, verification and approval
Given a plain folder, the actual planning subprocess returns three tasks from a read-only copy, and shared/worktree execution yields a tested candidate without changing the original. Unapproved application fails; exact approval applies files without creating .git. A later original edit causes refusal and preserves it. Excluded secret/build paths never enter the snapshot or application; aliases are refused. Trace: FR-65/NFR-36.

### QAS-76 Metadata resilience and bounded temporary history
An oversized CLI catalog becomes a persistent failed metadata observation, without blocking Snapshot/Save, including after restart; correcting the profile allows a successful retry. Replacing an unstarted plan removes its owned snapshot, while active and run-required history stays. Completed application removes private working files and retains Git objects, logs, tests, diffs and backups. Candidate stages are removed on failure. Trace: R-78/R-80, DD-86/DD-88, NFR-05.


### QAS-77 Agent tools guide is accurate and inert
The Settings dialog contains the guide button; the guide contains every verified command inside code elements (excluded from translation), contains no script, link or request, and every command also appears in the user manual. Korean strings exist for the guide's interface text. Trace: FR-66/NFR-37.


### QAS-78 Picked Korean path survives the picker
A Windows PowerShell run of the picker owner script writing C:\사다리\프로젝트 is parsed back to exactly that path on a runner whose OEM code page lacks Hangul; BOM, CRLF and trailing separator are removed on every platform; legacy code-page bytes are refused. Trace: FR-67/NFR-38.


### QAS-79 Planning reports real activity without false timeouts
Planning-entry actions pass the long limit; a real planning job snapshot carries output bytes; the progress text covers waiting, active and stale output; the reduced-motion style still blinks without transform. Trace: FR-68/NFR-39.


### QAS-80 Korean goal becomes an English folder
"사다리 게임을 만들자 그런데" yields goal_20261011 before planning; a real native planning run whose coordinator returns slug ladder_game leaves .agentconductor/ladder_game/plan/plan.json and active_slug ladder_game; non-placeholder slugs are never renamed. Trace: FR-69/NFR-40.


### QAS-81 Enterprise badge in percent
Budgets 640/1000 and remaining_percent 64 both show 64%/100%; 0/0 and credit-only 730/— show —; unlimited shows ∞; the tooltip still contains Individual allocation. Trace: FR-70/NFR-41.
