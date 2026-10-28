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

Given a restricted paid account and reviewed self-hosted fallback, no running attempt is steered or overlapped during transfer. A fresh target session receives bounded owned-path evidence; source quota remains blocked after transfer. Unknown targets/models and same-account fallbacks fail before spawn; agents may name each other, and a chain whose accounts are all restricted waits instead of looping. A stopped run never launches a fallback. Failed tests never become approval. Ordinary errors or silence do not trigger quota fallback. Trace: UC-18.

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

The planning prompt contains the workspace override exclusively when set. Otherwise it contains the existing global/default rules plus all Markdown files below `<repository>/.agentconductor/plan-rules/`, recursively and in relative lexical order. Files outside that root, non-Markdown files and symlinks are excluded. Trace: UC-48.

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
Given extensionless and .cmd npm launchers, choose .cmd. Given no desktop apps, each built-in provider returns a distinct bounded PNG badge. Given a Windows terminal fixture, send /usage only after the ready prompt, parse the reported windows, accept the folder trust prompt at most twice and reap the process tree. Verify the Windows build and tests in CI; actual enterprise subscription acceptance remains separately recorded. Trace: FR-56/NFR-28.


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


### QAS-82 Windows collector folder and first-run screen
On Windows, a .cmd launcher started with a canonical \\?\ folder runs in that folder and reports usage; the reported first-run screen ends the query with "first-run" in under 15 s; trust still reports trust. claude_blocker recognises both screens on every platform. Trace: FR-71/NFR-42.


### QAS-83 Readable Windows project paths
Trace: R-87 -> BG-42 -> SF-74 -> UC-77 -> FR-72 -> QAS-83 -> DD-95.
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity.


### QAS-84 Reliable named workspace creation
Trace: R-88 -> BG-43 -> SF-75 -> UC-78 -> FR-73 -> QAS-84 -> DD-96.
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable.


### QAS-85 Delete an unwanted workspace
Trace: R-89 -> BG-44 -> SF-76 -> UC-79 -> FR-74 -> QAS-85 -> DD-97.
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first.


### QAS-86 Register MCP servers in Settings
Trace: R-90 -> BG-45 -> SF-77 -> UC-80 -> FR-75 -> QAS-86 -> DD-98.
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration.


### QAS-87 Profile save success and failure feedback
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


### QAS-88 Signal delivery during optional-service failure
Source: operator or optional memory service. Stimulus: change transport while healthy, or stop/refuse/time out the agentmemory server or memory/signal MCP adapter during delivery. Environment: active local cooperative sessions; separate remote-recipient scenario. Artifact: Rust router, durable signal store and settings.
Response: persist preference, retain exact addressed signal, use Local Rust for reachable local recipients, show fallback cause, deduplicate late/repeated transport results, and preserve approvals. A remote recipient without a valid route stays pending.
Measures: no UI-thread blocking; at most one external signal operation and eight-second deadline; failure settles to fallback/pending/error within ten seconds; zero lost persisted messages and zero duplicate instruction acceptance in restart/late-reply fixtures; no read/acceptance state is fabricated. Validate healthy manual Local Rust selection, daemon refusal, MCP disconnect/error, late success after fallback, recipient replacement, expiry, restart and remote-route absence.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### QAS-89 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.
Measure: actual generated Workspace/Room documents show the exact build version with no template marker left; native header remains compact.


### QAS-90 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.
Measure: repeated successful refreshes retain at most eight eligible diagnostic/cache sets plus protected references; live/unknown jobs and path aliases are refused. Manual preview is side-effect-free and changed previews are rejected before deletion.


### QAS-91 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.
Measure: a blocking planning subprocess exits through its stop marker, no replacement writer starts early, and current-workspace deletion lands in an empty workspace with recoverable prior app data; project hash remains unchanged.

### QAS-92 Agents work inside the workspace sandbox; the coordinator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Work confined to the task workspace runs at once; a request reaching outside it (network, installs, history-changing Git, privilege, other processes, outside paths, secrets) is decided by the workspace coordinator , and a person is asked only when no coordinator can decide. The permission level is not chosen in the agent profile; it follows from the CLI. Time spent waiting for a decision does not count toward the task limit, and a task that times out can be resumed instead of failing.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.
Measure: Workspace-confined commands are classified Automatic and outside requests Human in the policy tests; a Human request is sent to the configured coordinator first and to the ledger only without a valid answer; a timed-out task becomes blocked and Resume runs it again; the profile dialog has no permission selector.

### QAS-93 Usage query accepts the Claude folder-trust prompt
Claude usage must load even when Claude first asks whether to trust the project folder. The usage query runs in a separate session with no tools, hooks or project settings, so it accepts the prompt automatically in both forms (Enter for the default choice, y for the screen-reader "Enter y/n:" form) at most twice, and stops with guidance instead of waiting for a timeout if Claude keeps asking.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.
Measure: ConPTY/PTY fixtures for the default, y/n and repeating prompts report usage, usage, and "kept asking to trust" within 15 s; a real Claude 2.1.296 usage query on Windows returned current-session and weekly usage.

### QAS-94 Planning start and workspace naming feedback
Generate plan shows progress with elapsed time from the moment it is pressed, including while the project folder is copied before the coordinator starts. Saving workspace settings updates the titlebar name at once. Deleting the current workspace asks for the name of the fresh workspace that replaces it instead of always naming it "New workspace".
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.
Measure: The plan progress box shows "Preparing the project folder for planning…" before the job exists; a delete with replacement " Fresh start " opens a workspace named "Fresh start"; the titlebar shows a renamed workspace after Save without reopening the menu.

### QAS-95 Run and task status at a glance
The Agent Room shows the run's current final state immediately: a run status banner with the state in bold colour, task progress and the last error, and a prominent Resume button whenever the run can continue. Task states (failed, completed, pending, waiting, running) are bold and colour-coded in the session heading and in the left session list. In the left list a failed task is red, a task waiting for a decision or tool request has a blinking border, pending tasks use a strong colour, and the Tool requests count turns red and blinks while requests wait.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.
Measure: For a failed run the banner reads Failed with "Tasks completed: n/m" and the last error, Resume is a primary button, the failed session row has a red border, a waiting_tool_approval row blinks (static border under reduced motion) and the tool count is red.

### QAS-96 Goal folders directly in the project folder
When a workspace has a project folder, each goal (slug) folder is created directly in it as <project>/<slug>/ (for example C:Templadderladder_and_priority_games) with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Goal folders in a former place, including the interim <project>/<workspace name>/<slug>/ of an earlier build that misread the request, move there when first used.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.
Measure: Creating a slug creates <project>/healthcheck_feature/{plan,phase,task,verification,completed}; Git status stays clean; a .agentconductor/<slug> or <project>/<workspace name>/<slug> folder moves to <project>/<slug> on first use; a plan whose task scope is inside a goal folder is refused.

### QAS-97 Agent CLI list is a master list
The registered agent CLI list is kept as a master list for the machine, not per workspace: deleting a workspace never removes agents, and every workspace opens with the master list. When a workspace refers to an agent the master list no longer has (planning coordinator or role assignments), the assignment is shown in red and can be reassigned; Generate plan stays disabled until every assignment refers to a listed agent.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.
Measure: After deleting a workspace the next workspace still lists the same agents; a workspace whose coordinator or role agent is missing from the master shows that selection in red with a reason, Generate plan and Draft plan are disabled, and become enabled after reassignment and Save.

### QAS-98 Reorder agent cards
Agent CLI cards can be moved up and down with the mouse (drag and drop), with keyboard-accessible move controls as an alternative. The order is saved, and every agent selection list (planning coordinator, roles, fallback) follows the same order.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.
Measure: Dragging the third card above the first saves the new order; the coordinator, role and fallback options list agents in that order after reload; Move up/Move down buttons give the same result without a mouse.

### QAS-99 Readable file paths everywhere
Screens, error messages and notifications never show Windows extended-length path prefixes such as \\?\C:\Work\Temp or //?/C:/Work/Temp; paths read as C:\Work\Temp.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.
Measure: Snapshot, error and notice text containing \\?\C:\…, \\?\UNC\server\share\… or //?/C:/… is shown as C:\…, \\server\share\… and C:/… in every desktop window and in messenger notifications; stored paths sent back to the engine are unchanged.

### QAS-100 Activity tab reads like the CLI
The Activity tab shows agent output the way the CLI shows it: assistant messages as text, tool calls as a one-line action with its key input, tool results as short summaries and errors highlighted. Raw JSON is never shown there; it remains in the Raw logs tab.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.
Measure: Claude stream-json, Codex JSONL and OpenCode events render as text, "● Bash(cargo test)"-style tool lines and result summaries; an unrecognised event shows its type label, never a JSON object.

### QAS-101 Coordinator final step in the task list
The Agent Room task list shows the step the coordinator runs after the tasks, integration verification of the combined result, as its own entry with its status (pending, running, completed, failed) and which task tests failed, so a run that fails after every task completed shows why without opening other views.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.
Measure: With all tasks completed and t01's test failing on the combined result, the list ends with Coordinator · Integration verification · Failed: t01-requirements-doc in red; selecting it opens Diff & tests; with all tests passing it shows Completed.

### QAS-102 Decide a live tool request from its window
When a tool request window opens while the run is working, Allow once and Deny are available at once, without closing the window with Decide later and deciding again from the Tool requests list.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.
Measure: While the run's job is running and a request is pending, both buttons in the window are enabled; one click records the decision; the buttons stay disabled only while that decision is being sent or after the request settles.

### QAS-103 Coordinator decides routine requests and records each decision
Simple decisions are made by the coordinator so the user is not asked often: it allows routine development requests, denies only publishing, outside-project changes, secret exposure or system changes, also for runs started before orchestration existed, and every decision is recorded with the request, the result, the reason and the time, visible in the task's Activity. Commands inside a workspace that itself lives under an app folder (such as ~/.agentconductor) are workspace commands.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.
Measure: A resumed pre-v0.1.13 run's outside request is decided by the workspace coordinator; Activity shows Coordinator allowed/denied with the command and reason; a cd into the workspace path under ~/.agentconductor runs without a decision; a person is asked only after two attempts give no valid answer.

### QAS-104 Task list without account usage
The Agent Room task list does not show account usage badges or refresh buttons; account limits are shown in the inspector's Account capacity section.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.
Measure: No quota badge or refresh control appears in session rows; Account capacity still shows and refreshes the selected agent's limits.

### QAS-105 The coordinator is selectable and shows its work; one name
The interface and documents use one name, coordinator, for the agent that plans, verifies the combined result and decides tool requests (previously also called orchestrator). In the Agent Room task list the coordinator entry is selected like a task: the center shows its heading, status and an updating Activity of its run steps, tool decisions and integration test results, and Raw logs show the integration test output, following the Agent room design (select on the left, progress in the center, evidence on the right).
Trace: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117. Source: user request, 2026-10-13.
Measure: Selecting the coordinator entry highlights it, shows Coordinator with the verification status, and the Activity lists run steps, Coordinator allowed/denied decisions and failed integration test output; it refreshes on every poll; no interface text says Coordinator.

### QAS-106 Failure cause and the next action on a failed run
When a run fails, the status banner shows why (for an integration verification failure, the failed task and the first lines of its test output) and offers Fix with feedback, which selects the failed task and drafts a direction with the failure output for the user to review and send; sending it reruns the task and repeats verification.
Trace: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118. Source: user request, 2026-10-13.
Measure: For a run failed by t01's integration test, the banner shows the failed task and output lines and Fix with feedback; pressing it selects t01 and fills Your direction with the output; Send feedback continues the run.

### QAS-107 The next step is always shown
The Agent Room status banner always says what to do next and offers that action: review the changes when the run waits for approval (Approve stays unavailable until the latest evidence is reviewed), approve after review, apply or merge after approval, and return to the workspace to plan the next phase when the run is complete.
Trace: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119. Source: user request, 2026-10-13.
Measure: In waiting_approval the banner reads Next: review the changes with Review changes; after the review loads it offers Approve; after approval it offers Apply changes; when completed it offers Back to workspace.

### QAS-108 Plan review and run completion are organised by phase
A phased goal shows its phases in Review the plan: which phase is under review (phase n of m), which are completed, which comes next, and a Plan phase n action when the next phase is ready to load. When a run finishes, the Agent Room says which phase completed and which phase is next, and that it is planned from the workspace; a completed phase is recorded even if the goal folder moved after the run started.
Trace: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121. Source: user request, 2026-10-13.
Measure: After run-52dd3147 the Agent Room reads Phase 1 of 4 (ladder core) is complete. Phase 2 (race core) is next; the workspace's Review the plan lists 1 completed, 2 next with Plan phase 2; the phase-1 task files are in completed/ even though the run linked the former .agentconductor folder.

### QAS-109 A blocking verification finding fails integration verification
When a verification report changed in the run records a blocking issue (a Blocking/차단 section with items, or a positive blocking-issue count), integration verification fails like a failing test, naming the report and the task responsible for the files the issue names, so the coordinator does not accept a candidate its verifier rejected and Fix with feedback drafts the correction for that task.
Trace: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120. Source: user request, 2026-10-13.
Measure: A candidate whose docs/verification/phase-01.md lists B-01 under 차단 (naming src/core/ladder.js) fails with integration verification failed: docs/verification/phase-01.md records a blocking issue for t03-ladder-impl; reports with only non-blocking notes, None/없음, or a zero count pass.

### QAS-110 The run's phase beside the workspace name
The Agent Room titlebar shows, beside the workspace name, the phase the run executes (phase n of m and its name) and what it covers (the phase description, or its task list when the phase file has no description); the full text is in its tooltip.
Trace: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122. Source: user request, 2026-10-13.
Measure: Opening run-52dd3147 shows Phase 1/4 · ladder core with Tasks: t01-requirements-doc, … beside the workspace menu; returning to the workspace hides it; runs without a phase show nothing.

### QAS-111 Readable Agent Room text and agent icons sized to their names
The Agent Room's text is at least 11 px and scaled up in proportion (body text 14 px instead of 12 px) for readability, workspace screen text is at least 11 px, and the agent icons in the task list are the height of the name beside them instead of a large tile.
Trace: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123. Source: user request, 2026-10-13.
Measure: No room.css or startup.css font size is below 11 px; task-list icons are 22 px and the heading icon 28 px; the Python UI contract suite passes.

### QAS-112 Task cards in Review the plan fold to one line
In Review the plan each task is folded by default, showing only its number, name and a one-line description (the first line of its instructions that is not a label); an arrow unfolds it to show and edit every field.
Trace: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124. Source: user request, 2026-10-13.
Measure: A generated plan shows each task as one summary line with ▸; clicking it opens the task's fields; Remove still works from the summary line without toggling it; saving plan edits reads the same fields as before.

### QAS-113 Review the whole phased plan up front
The coordinator plans every phase of a goal when the plan is generated, so Review the plan briefs each phase (what it will do, from the coordinator's analysis) and lists each phase's planned tasks, with the coordinator's analysis of the whole goal available; the phase about to run is open and its tasks are edited below, completed phases are marked, and later phases can be unfolded to review their tasks before any of them runs.
Trace: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125. Source: user request, 2026-10-13.
Measure: After Generate plan for the ladder goal, Review the plan lists 4 phases with their briefing lines (e.g. 2. 02-race-core: marble race priority logic (TDD)) and task lists (t05, t06, t07); phase 1 is open as under review; the analysis expands; the goal_briefing test passes.

### QAS-114 A run survives closing or losing the desktop app
Closing the desktop app (or the app process ending) must not end a run: the run coordinator and its agent workers keep working and the app reattaches when reopened. Closing the window while a run is active asks: Keep running in the background (default), Stop the run and close, or Cancel.
Trace: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127. Source: user request, 2026-10-13.
Measure: On Windows and macOS, ending the app process while a task runs leaves the coordinator and worker alive; reopening shows the same run progressing (DD-126 routing opens the Agent Room); the close dialog's Stop path stops through the normal Stop request and records it.

### QAS-115 Interrupted runs recover and resume automatically
When a run is recorded as running but its coordinator and workers are gone (app killed, crash, reboot), opening the workspace or the Agent Room recovers it automatically (verifying every recorded process is absent, as Recover does) and, with automatic resume on by default, resumes it: completed tasks are kept, the interrupted task continues in its own native session (or a new one if the session cannot resume) with a note that its previous attempt was interrupted and its owned files must be inspected. After two consecutive automatic resumes that are interrupted again, it stops and asks the operator.
Trace: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128. Source: user request, 2026-10-13.
Measure: Killing the app tree during t08 and reopening shows Interrupted at hh:mm → recovered → resumed automatically; t08 continues with --resume of its session; t01..t07 stay completed; a third interruption within the cap leaves the run waiting for the operator; the setting can turn automatic resume off.

### QAS-116 Interruptions and silent workers are visible
The Agent Room shows when a run was interrupted and recovered (banner history and coordinator Activity), collapses a worker's consecutive thinking_tokens events into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows, and warns when a running task has produced no output for several minutes or its process is gone, suggesting Recover.
Trace: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129. Source: user request, 2026-10-13.
Measure: t08's transcript renders one Thinking line with its count and time; a running task silent for over 5 minutes shows No output for n minutes; a task whose process is gone shows Not responding — Recover; the banner lists Interrupted 14:38 → resumed 14:41.

### QAS-117 Reassign tasks that have not started while the run is working
While a run is working, the operator can change the agent (and model) of a task that has not started yet, for example to move work from a busy Claude account to Codex; the change applies before that task starts. Running and finished tasks still need the run to be idle.
Trace: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130. Source: user request, 2026-10-13.
Measure: With t08 running, assigning t09 (pending) to codex is accepted at once and recorded; when t09 starts it runs on codex; assigning t08 (running) is refused with a reason.

### QAS-118 Agents run with the highest permission by default
Stimulus: a phase verification task must check a web UI. Response: the worker opens it through the operator's Playwright MCP server; the coordinator still runs read-only. Measure: no blocker reports a missing browser connection.
Trace: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131. Source: user request, 2026-10-13.

### QAS-119 The coordinator resolves verification failures
Stimulus: all tasks completed and a verification report records blocking issues for another task. Response: within the same run process the coordinator revises the owning tasks and the run resumes, at most three rounds. Measure: the run reaches waiting_approval without operator action, or stays Failed with the reason after three rounds.
Trace: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132. Source: user request, 2026-10-13.

### QAS-120 Selected-folder outputs and baseline restoration
Given an ordinary project with existing files and no Git metadata, start shared execution. Verify outputs appear only in the selected project, `.git` remains absent, and baseline evidence precedes task launch. Accept preserves outputs; Reject restores modified/deleted files and removes newly created owned files. Manual edits after the worker checkpoint produce a warning and refuse automatic restoration without overwriting those edits. Confirm existing Git and legacy private-folder runs still work.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user workspace-path requirement and Claude review, 2026-10-13.

### QAS-120 Deleting workspaces needs no replacement name
Stimulus: the operator deletes the current workspace three times in a row. Response: each deletion asks only for confirmation and opens the most recently used remaining workspace; deleting the last leaves one empty workspace. Measure: no name prompt appears.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user request, 2026-10-13.

### QAS-121 Documentation language and release consistency
Given a Korean UI preference and a release update, run documentation and native UI checks. Both manuals cover every executable command, all screenshot links resolve, README stays under 120 lines, and no language-dependent error assertion fails.
Trace: R-125 -> BG-80 -> SF-112 -> UC-115 -> FR-110 -> QAS-121 -> DD-134. Source: user release/manual requests, 2026-10-14.

### QAS-122 The app updates itself from GitHub releases
Stimulus: a newer release is published while the app is idle. Response: within 10 minutes the update icon appears; Update and restart installs the verified archive and the app restarts on the new version. Measure: a tampered archive or an active run never replaces the executable; a failed replacement restores the original.
Stimulus: the operator saves a different update repository in Settings. Response: invalid values (non-HTTPS, credentials, wrong path shape) are rejected with a localized message and nothing is saved; a valid GitHub `owner/name` or HTTPS URL on any GitHub-compatible host is used for the next check, release notes, archive and SHA256SUMS (#130).
Trace: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135. Source: user request, 2026-10-14.


### QAS-123 Recover timers without duplicate or stale-assignment execution
Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

### QAS-124 Theme switch without information loss

Given the workspace and Agent Room in Light, choosing Dark in Settings recolors every surface, text, line and status immediately; reopening keeps Dark; an unknown theme value is refused; component CSS contains no color literals outside the two token blocks; a session row shows its icon and its status tag on its own wrapping line, never wider than the rail. Trace: FR-113 / NFR-49.

### QAS-125 Plans know whether the project uses Git, and verification commands that cannot work are replaced
Stimulus: a plan for a folder without Git contains git diff --check, or an agent reports the planned test command cannot run. Response: the test runs against the external history, or the coordinator revision sets a replacement test_command; integration verification passes. Measure: no run loops on a command that cannot work; every replacement appears as test_command_revised with the old and new command.
Trace: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138. Source: user request, 2026-10-14.

### QAS-126 Compact metadata without information loss

Given an exhausted window with a known or unknown reset, the card remains compact and its tooltip preserves the deadline or unknown state. On light/dark Korean/English screens, step introductions and action descriptions use the available width without clipping controls.
Trace: R-130 -> BG-85 -> SF-117 -> UC-120 -> FR-115 -> QAS-126 -> DD-139. Source: user, 2026-10-14.

### QAS-127 Settings readability and retained server values

Given a saved agentmemory port, changing transport to local and back hides/shows the field without data loss. The guide is absent from Settings and available from the workspace menu. Native light/dark labels do not overlap inputs.
Trace: R-131 -> BG-86 -> SF-118 -> UC-121 -> FR-116 -> QAS-127 -> DD-140. Source: user, 2026-10-14.

### QAS-128 Custom CLIs that approve their own tools can run
Stimulus: an operator runs a plan with a custom CLI that auto-approves its tools. Response: unmarked, execution is refused with a message that points to the agent setting; marked, the run starts. Measure: an imported profile never arrives marked unless the importing operator keeps verified claims.
Trace: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141. Source: user request, 2026-10-14.

### QAS-129 Scrollable untrusted release notes

Given a multiline body containing markup, the dialog shows the literal characters without executing HTML. A long body scrolls inside 35vh; an absent body clears/hides the panel. Installation behavior is unchanged.
Trace: R-133 -> BG-88 -> SF-120 -> UC-123 -> FR-118 -> QAS-129 -> DD-142. Source: user, 2026-10-14; next release after v0.1.18.

### QAS-130 Large-document and multi-server correctness

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.


### QAS-131 Planning retry and CLI card clarity

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.


### QAS-132 No stale edit flags or lost API restrictions

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.


### QAS-133 / QAS-134 Five reconnects then final success, safe manual display

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

### QAS-135 Workspace and Agent Room feedback refinements
Stimulus: the operator previews MCP sources, hovers a goal reference, or opens a finished run. Response: the preview window appears immediately with progress; the hover shows the source title and summary; the heading shows the run state; usage shows percentages; the selected task is outlined. Measure: no step leaves the operator without visible progress or with a stale status.
Trace: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148. Source: user requests, 2026-10-14.

### QAS-136 Runs keep going and stay visible
Stimulus: a model is momentarily at capacity, an agent writes outside its scope, the operator presses Approve & run, or leaves the run screen during a run. Response: the task or coordinator call retries up to three times; stray files leave the project; the pressed button shows progress until its job ends; the run screen keeps updating while the workspace page is shown. Measure: no run stops on a single capacity reply; no unowned agent file remains in the project; the operator can always switch views.
Trace: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149. Source: user requests, 2026-10-14.


### QAS-137 Optional memory absent / QAS-138 Unicode native selection

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

### QAS-140 Offline table integrity and startup memory

Given command tables and escaped pipe characters, HTML preserves columns and code. Raw scripts render as text and javascript links cannot execute. Generated copies match Markdown and images. Startup omits manual bodies; opening help fetches the chosen embedded document.
Trace: R-144 -> BG-99 -> SF-131 -> UC-134 -> FR-129 -> QAS-140 -> DD-153. Source: user, 2026-10-14.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

### QAS-143 Completed-room navigation regression

Given a retained completed room, workspace creation succeeds without restart; a running report is refused and remains attached.
Trace: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156. GitHub #4, user2026-10-14.

### QAS-144 Real PTY prompt variants

Run a fixture CLI through the native collector for all three prompts and verify usage plus quiescent ownership. A mere banner or cost line must not authorize command input.
Trace: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157. GitHub #5, user2026-10-14.

DD-157 implementation note: the operator confirmed Windows Claude2.1.296 uses a > prompt and a Yes-I-trust-folder choice, with an explicit spent/allocated Enterprise line. Parse that declared allocation using existing exact-decimal subtraction; the display percentage is derived from remaining/allocated amounts. Date-only resets remain unscheduled. Committed fixtures use synthetic amounts; no operator account values are published.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

### QAS-146 Completed goal to new goal to prior history

A native regression completes real fixture work, starts a new goal, confirms no old approval is reusable, switches goals with preserved run evidence and refuses duplicate folder binding without mutating its form.
Trace: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159. GitHub #6, user2026-10-15.

### QAS-147 Rust-only application and release verification
Source and artifact inspection finds no project-authored JavaScript or Python in the installed application or required release path; Rust tests cover UI actions and release/manual generation. macOS and Windows artifacts start without a Node.js or Python runtime, preserve current interactions/accessibility, and meet the existing idle-resource budget. Third-party implementation source is not copied into the packages.
Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160.

### QAS-148 Slow plan retry has accurate progress and recovery
Given plan startup is still saving workspace state, checking a baseline, or waiting for an owned snapshot, requests remain within the bounded planning timeout and the review screen remains in preparation state. If that bound is exceeded, automatic progress polling continues and localized guidance offers reopening the workspace or run as recovery; it never asks for a refresh action absent from the UI. Retrying after an old failed plan clears its failure before starting, and only the new job controls subsequent progress and failure display.
Trace: R-152 → BG-107 → SF-139 → UC-142 → FR-137 → QAS-148 → DD-162. GitHub #36, 2026-10-19.

### QAS-151 Company suffix is visible locally and excluded from releases
Given an empty suffix file, both titlebars show the numeric Cargo version. Given `company_name`, both show `<numeric-version>.company_name`, while update comparison still uses numeric components. Package assembly fails before writing an archive for any non-empty suffix. Invalid labels do not alter the displayed version.
Trace: R-155 → BG-110 → SF-142 → UC-145 → FR-140/NFR-52 → QAS-151 → DD-165. GitHub #13.
### QAS-150 Custom CLI profile extension is bounded and backward compatible
Given an existing workspace profile without custom event or quota settings, deserialize and execute it with the previous wire behavior. Given a custom profile, map configured event names and session pointers, invoke only its configured argv quota command within 1–300 seconds, retain process logs, reject output over 1 MiB or outside the normalized Snapshot bounds, and preserve the configuration through workspace export/import. A profile without a quota command is not queried. Verify with Rust tests on macOS and Windows; built-in Claude/Codex/OpenCode fixtures must remain unchanged.
Trace: R-154 → BG-109 → SF-141 → UC-144 → FR-139 → QAS-150 → DDR-17 → DD-164. GitHub #12, 2026-10-19.
### QAS-149 Timeout recovery preserves evidence, honors Stop, and informs operators
Given a worker times out once then succeeds, the run retries after a bounded delay, reaches review, and retains the first attempt's timeout logs and audit record. Given a stop request during the retry delay, no new attempt starts and the run ends stopped. When the final state is blocked, the report's top-level error identifies the cause. Regression tests cover automatic retry, Stop during backoff, and the existing JavaScript run-banner contract consuming `Report.error` without suppressing it for blocked state.
Trace: R-153 → BG-108 → SF-140 → UC-143 → FR-138 → QAS-149 → DD-163. GitHub #44, 2026-10-19.

### QAS-152 Approval and merge progress remains continuous and visible
Measure: the run board shows ordered Plan, Tasks, Verify, Merge, and Workspace stages. `waiting_approval` remains on Verify; `approved` and a running merge/apply job show Merge; a running merge appears in the primary banner and its action stays visibly busy; only `completed` reaches Workspace and offers return navigation. Latest-evidence review and separate human approval and merge gates are unchanged. Verify English/Korean labels and reduced-motion behavior.
Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166. GitHub #50, user, 2026-10-20.

### QAS-153 Concurrent state writes survive a transient lock
Given one process holds a SQLite writer lock for 2.5 seconds while another run connection records task state, the write completes after the lock clears and the persisted record can be read. Given a lock that outlasts the ten-second bound, the operation returns an error and does not claim success. Run the regression locally and verify the packaged parallel workflow on macOS and Windows.
Trace: R-157 → BG-112 → SF-144 → UC-147 → FR-142 → QAS-153 → DDR-18 → DD-167. GitHub #56.

### QAS-154 Custom CLI contracts round-trip and validate at every boundary
Given a custom profile with a manual catalog, explicitly configured discovery mapping, templated option groups, per-model restrictions, result pointer, and error condition, serializing and reloading the profile preserves each field and provenance. Discovery does not spawn unless configured. Given an unset optional group, argv contains none of its tokens; configured defaults and valid task overrides produce the expected argv, and resume uses the exact session ID. Given reassignment to a profile/model that rejects a persisted option, validation refuses the handoff before it changes ownership. Given JSONL result events with matching and nonmatching error conditions, the selected payload is extracted and only matching events become errors. The native editor and task review expose these settings; workspace import/export preserves them; built-in adapter tests remain unchanged. Enforce the 300-line Rust source/test limit and run focused local tests plus the all-target native check.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62.

### QAS-155 A quiet running task shows tool, last-output age, and process state
Given a running task whose last event was a tool start 3 minutes ago and which has printed nothing since, the task header names the tool with about 3 minutes elapsed and the last output age. Given a tool result, the tool line disappears. Given an absent owned process the header says the process is not present. Given a CLI with no tool events, only the last-output age and process state appear. Under reduced motion the running icon does not animate.
Trace: R-159 → BG-114 → SF-146 → UC-149 → FR-144/NFR-54 → QAS-155 → DDR-20 → DD-169. GitHub #100, user, 2026-10-21.

### QAS-157 A question yields an answer or a visible error, never a file change
Given a coordinator that answers `{"answer": ...}`, the chat shows that answer once; given a failing coordinator it shows the error and keeps the conversation usable; invalid conversations (empty, ending with the assistant, over-long, unknown role) are refused before any job starts; the prompt stays under the coordinator limit.
Trace: R-161 → BG-116 → SF-148 → UC-151 → FR-146/NFR-55 → QAS-157 → DDR-22 → DD-171. GitHub #115, user, 2026-10-21.
### QAS-156 The selection follows a completed session to the next running one
Given sessions a (running) and b (pending), when a completes and b starts, the selection moves to b; when a completes and the run waits for approval, the integration entry opens; a finished session selected for review stays selected.
Trace: R-160 → BG-115 → SF-147 → UC-150 → FR-145 → QAS-156 → DDR-21 → DD-170. GitHub #109, user, 2026-10-21.

### QAS-158 A scope failure recovers through the app with evidence kept
Given a task that wrote a file outside its path, the failure lists that file and the task's paths; feedback leaves the paths unchanged; a preview with a wrong digest is refused; an approved change leaves the run stopped with no candidate or error, resume keeps the task's attempt count, and the run reaches review; overlapping ownership is refused; a quarantined file restores to its original path unless a different file is there, and its recovery copy remains.
Trace: R-162 → BG-117 → SF-149 → UC-152 → FR-147/NFR-56 → QAS-158 → DDR-23 → DD-172. GitHub #122, user, 2026-10-22.

### QAS-159 A failed task is repaired by the coordinator without the operator
Given a task that exits with an error and a recorded coordinator that revises it, the run resumes, the task's attempt count increases and the run reaches waiting_approval with one review round; without a recorded coordinator the failure stays unchanged; an out-of-scope failure and a run waiting for availability are never triaged; after three rounds the run keeps the failure with an explanation for the operator.
Trace: R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173. GitHub #135, user, 2026-10-22.

### QAS-160 A lesson reaches later tasks and the operator
Given a coordinator repair of a failed task, the run has one lesson naming the task, the error and the feedback; repeating it records nothing new; the 51st distinct lesson drops the oldest; the next task prompt contains the lessons under an evidence-only heading and stays under 4 KiB; the task detail returns them; the memory outbox holds an entry for each; an unreachable agentmemory does not fail the step.
Trace: R-164 → BG-119 → SF-151 → UC-154 → FR-149/NFR-58 → QAS-160 → DDR-25 → DD-174. GitHub #138, user, 2026-10-22.

### QAS-161 A folded detail column opens for a decision and returns afterward
Given a folded column, a run that enters waiting_approval opens it once; folding it again keeps it folded with the dot blinking; a second distinct decision opens it again; when the decision is gone the column follows the saved preference; a saved preference survives a new page load; a failing local storage leaves the column open; a decision with the column open shows no dot.
Trace: R-165 → BG-120 → SF-152 → UC-155 → FR-150/NFR-59 → QAS-161 → DDR-26 → DD-175. GitHub #146, user, 2026-10-22.

### QAS-162 A routine block is decided by the coordinator within limits
Given a violation of each code, the coordinator's amendment is validated, applied or escalated by tier, recorded, shown and undoable; the budget stops repetition.
Trace: R-166 → BG-121 → SF-153 → UC-156 → FR-151/NFR-60 → QAS-162 → DDR-27 → DD-176. GitHub #149, user, 2026-10-22.

### QAS-163 Every judgment is listed
Given recorded tool decisions, reviews, test-command replacements, time-out resumes, recoveries, hand-offs and lessons, each appears once in order with its kind, judgment and task; unknown records are ignored; the snapshot count matches the list; the task view lists only its own task's judgments.
Trace: R-167 → BG-122 → SF-154 → UC-157 → FR-152/NFR-61 → QAS-163 → DDR-28 → DD-177. GitHub #149, user, 2026-10-22.

### QAS-164 The launch gate blocks before anything is created
Given a test command or an agent executable that does not exist, the run is refused with one message listing every violation by code and task, no run folder exists afterwards, and with everything present the run starts, records a launch_gate record and lists a gate judgment; a project-relative script resolves.
Trace: R-168 → BG-121 → SF-155 → UC-158 → FR-153/NFR-62 → QAS-164 → DDR-27 → DD-178. GitHub #149, user, 2026-10-22.

### QAS-165 A low-risk amendment continues the run and can be undone; others wait for a person
Given an out-of-scope failure and a coordinator that adds the offending file, the run reaches waiting_approval with the path added and one applied amendment listed; undoing it restores the paths and stops the run; a whole top-level directory, a secrets path, an overlapping path, an unknown dependency and the sixth amendment are not applied and stop the run with a reason.
Trace: R-169 → BG-121 → SF-156 → UC-159 → FR-154/NFR-63 → QAS-165 → DDR-29 → DD-179. GitHub #149, user, 2026-10-22.

### QAS-166 The charter is recorded, shared with the coordinator and enforced
Given a started run, a charter record lists the goal and the original scopes; the review evidence contains it; amendments within eight added paths apply and show their drift; one that would exceed eight escalates with goal_drift and the run is blocked; an operator-approved scope change records an updated charter.
Trace: R-170 → BG-121 → SF-157 → UC-160 → FR-155/NFR-64 → QAS-166 → DDR-30 → DD-180. GitHub #149, user, 2026-10-22.

### QAS-167 Lifecycle tests wait for recorded readiness
Given an asynchronous worker and a process tree under load, tests wait until all checkpoint markers and active session records exist before directing the worker, and wait for the child PID marker before testing containment. When a process exits or a bounded deadline expires first, the failure reports that outcome with retained evidence rather than panicking on a missing file.
Trace: R-171 → BG-123 → SF-158 → UC-161 → FR-156/NFR-65 → QAS-167 → DDR-31 → DD-181. GitHub #156, 2026-10-22.

### QAS-168 Feedback to the coordinator is acted on and visible
Given a run with a recorded coordinator, a request is queued, processing and completed in the judgment list with its summary, completed tasks keep their attempt count on an answer-only verdict, a revise verdict restarts only the revised task and the run resumes, a failing coordinator records failed with the error, empty or oversize messages are refused, a run without a coordinator refuses the request, and the room sends coordinator feedback through its own action with a separate draft.
Trace: R-172 → BG-124 → SF-159 → UC-162 → FR-157/NFR-66 → QAS-168 → DDR-32 → DD-182. GitHub #165, user, 2026-10-23.

### QAS-171 The policy is applied in review and is checked against the repository
Given a pull request that adds an `unsafe` block, a glob import or a `&Vec<T>` parameter, a reviewer finds the rule and its exception wording in `docs/13-rust-review-policy.md`. Given the repository at the baseline commit, the format, Clippy and source-layout checks pass unchanged, no test or source file is modified by the policy, and every exclusion names its reason.
Trace: R-174 → BG-129 → SF-162 → UC-165 → FR-160/NFR-70/C-06 → QAS-171 → DDR-34 → DD-185. GitHub #169, user, 2026-10-23.

### QAS-172 An undocumented unsafe block fails the build
Given the repository after the change, `cargo clippy --lib --bins --locked --features native-ui-probe -- -D clippy::undocumented_unsafe_blocks` passes on macOS and Windows; given a new `unsafe` block without a `// SAFETY:` comment in `src/`, the same command fails and names the file and line. Format, tests and the source-layout check pass unchanged and no runtime behavior differs.
Trace: R-175 → BG-130 → SF-163 → UC-166 → FR-161/NFR-71 → QAS-172 → DDR-35 → DD-186. GitHub #172, user, 2026-10-24.

### QAS-173 Test targets pass the unsafe gate
Given the repository after the change, `cargo clippy --lib --bins --tests --locked --features native-ui-probe -- -D clippy::undocumented_unsafe_blocks` passes on macOS and Windows; `grep -rE 'env::(set_var|remove_var)' tests` finds calls only inside the helper; the full suite passes unchanged and a repeated run shows no new flakiness.
Trace: R-176 → BG-131 → SF-164 → UC-167 → FR-162/NFR-72 → QAS-173 → DDR-36 → DD-187. GitHub #176, user, 2026-10-24.

### QAS-174 The proposal comes from the skill and stays read-only
Given a goal and a stub coordinator, the prompt the coordinator receives contains the skill's text, the four reference files and the goal; the stub's JSON reply is returned as the proposal, changes and questions with the bounds of NFR-73; a reply without `optimized` is an error; an empty or over-long goal is refused before any process starts; the coordinator profile is run read-only.

### QAS-175 The operator decides and the package carries the license
Given the workspace page with a proposal, **Use this goal** replaces the goal text and leaves the workspace unsaved, **Keep mine** leaves it unchanged, and an error leaves it unchanged with the reason shown; both interface languages have every new text. Given the release package, the Apache-2.0 license text and the notice naming getsentry/skills and its commit are present, and the vendored files match the recorded SHA-256 sums.
Trace: R-177 → BG-132 → SF-165 → UC-168 → FR-163/FR-164/NFR-73 → QAS-174/QAS-175 → DDR-37 → DD-188. GitHub #180, user, 2026-10-27.

### QAS-176 The entries live in Settings only
The Settings dialog contains both buttons and their handlers; the workspace menu script has neither entry; the guide stays static and its commands match the manual.
Trace: R-178 → BG-133 → SF-166 → UC-169 → FR-165 → QAS-176 → DD-189. GitHub #184, user, 2026-10-27.

### QAS-177 The conversation survives and follow-ups work
Given a stored question and answer, `history` returns both and an unanswered question alone; a 30 KiB earlier answer does not block the next question; the page restores turns, adopts a running chat, recalls earlier questions with Up and clears on New conversation or a workspace change.
Trace: R-179 → BG-134 → SF-167 → UC-170 → FR-166/FR-167 → QAS-177 → DD-190. GitHub #185, user, 2026-10-27.

### QAS-178 No dead texts remain
Given the embedded pages and both message catalogs, none of the removed element classes or keys appears, and every `data-i18n` id used by a page still exists in the catalogs.
Trace: R-180 → BG-135 → SF-168 → UC-171 → FR-168 → QAS-178 → DD-191. GitHub #190, user, 2026-10-27.

### QAS-179 A late reply is not lost
Given a coordinator that answers after two seconds, the chat request returns before it finishes, a second question is refused meanwhile, the answer arrives in a later snapshot and the next prompt contains the earlier exchange; given a send that times out after the application accepted it, the question stays and the late answer is shown, and given a send the application never saw, the text is restored.
Trace: R-181 → BG-136 → SF-169 → UC-172 → FR-169 → QAS-179 → DD-192. GitHub #197, user, 2026-10-28.

### QAS-180 The record exists and the guide ships
Given a chat request, the log has a start and a done line (with the reason on failure); a fast snapshot leaves no line; a log above the limit restarts and keeps one previous file; both shipped manual pages contain the file name, the line formats and the decision table terms.
Trace: R-182 → BG-137 → SF-170 → UC-173 → FR-170 → QAS-180 → DD-193. GitHub #200, user, 2026-10-28.
