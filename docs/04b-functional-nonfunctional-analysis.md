# Functional and non-functional analysis

Method step 5 of: requirements -> business goals -> system features -> use cases ->
**functional / non-functional analysis** -> QAS -> design drivers -> architecture.
Each use case is decomposed into functional requirements (FR, what the system does) and
non-functional requirements (NFR, how well or under which constraint). NFRs are refined
into measurable QAS in `05-qas.md`; FR/NFR/constraints are prioritized into design drivers
in `05b-design-drivers.md`.

## Functional requirements

| ID | Function | Use cases | Features |
|---|---|---|---|
| FR-01 | Register CLI profiles (argv, parser, input mode, sessions, capability) without collecting credentials | UC-01, UC-15 | SF-01, SF-18 |
| FR-02 | Query and import observed model catalogs | UC-11, UC-15 | SF-11 |
| FR-03 | Generate a read-only plan (1–12 tasks, R-53) from a goal | UC-02, UC-15, UC-23 | SF-02, SF-18, SF-26 |
| FR-04 | Validate plan schema, DAG, disjoint scopes and test argv; bind approval to its digest | UC-02, UC-15 | SF-02 |
| FR-05 | Execute up to three tasks in parallel in a shared workspace or optional worktrees | UC-02 | SF-03 |
| FR-06 | Capture raw stdout/stderr and normalize provider JSONL events | UC-02, UC-03 | SF-04 |
| FR-07 | Gate sensitive tool calls with exact human decisions before execution | UC-04, UC-22 | SF-05, SF-25 |
| FR-08 | Stop, resume, revise, recover and schedule attempts with retained evidence | UC-03, UC-09 | SF-06, SF-09 |
| FR-09 | Aggregate diffs and coordinator-run tests into a candidate; merge only on exact approval | UC-04 | SF-05 |
| FR-10 | Reassign roles and apply reviewed fallback after account restriction | UC-10, UC-18 | SF-10, SF-21 |
| FR-11 | Record reports, lessons and wiki assets with source hashes; sync to agentmemory via outbox | UC-05, UC-21 | SF-07, SF-24 |
| FR-12 | Exchange addressed checkpoint directives with non-steerable CLIs | UC-21 | SF-24 |
| FR-13 | Show quota windows, status lights and a factual team briefing | UC-16, UC-19, UC-20 | SF-19, SF-22, SF-23 |
| FR-14 | Provide the native desktop setup and Agent Room for all operator actions | UC-15, UC-22, UC-24 | SF-18, SF-27 |
| FR-15 | Build, verify and publish macOS and Windows release artifacts | UC-17 | SF-20 |
| FR-16 | Plan in a disposable tracked-files snapshot and verify the target is unchanged | UC-23 | SF-26 |
| FR-17 | Hold a paused tool call for a live decision and decide it while the run is active | UC-22 | SF-25 |
| FR-18 | Accept input-method (Korean) composition in all native text fields | UC-24 | SF-27 |
| FR-19 | Return from the workspace to the parent screen and discard unsaved edits after confirmation | UC-25 | SF-28 |
| FR-20 | Show installed application icons for built-in agent kinds, letters for custom agents | UC-26 | SF-29 |
| FR-21 | Provide a product README and a complete user manual covering every CLI command | UC-27 | SF-30 |
| FR-22 | Create an editable plan draft without running a coordinator; approve it through the same validation and digest | UC-28 | SF-31 |
| FR-23 | Group workspace settings, order goal → coordinator → repository/worktrees, pick the repository with a native folder dialog, save without a repository | UC-29 | SF-32 |
| FR-24 | Show the current workspace and switch to recent, existing or new workspace folders in place | UC-30 | SF-33 |
| FR-25 | Name workspaces; create by name, add folders, list by name, remove from the registry without deleting files | UC-31 | SF-34 |
| FR-26 | Publish each release with its changelog section; refuse publication without one | UC-32 | SF-35 |
| FR-27 | Report repository cleanliness with changed files; commit all changes as baseline only on explicit choice | UC-33 | SF-36 |
| FR-29 | Detect untracked OS metadata files and exclude them locally on explicit choice, then re-check the baseline | UC-36 | SF-36 |
| FR-30 | Show planning progress (coordinator, elapsed time), ready notice and failure reason with log location | UC-37 | SF-38 |
| FR-31 | Require sectioned, line-broken task instructions in the goal's language; auto-growing instruction editor of at least six rows | UC-38 | SF-39 |
| FR-32 | Store per-workspace role rules, pass them to the planner, require a role per task and an allowed agent, reject violating plans | UC-39 | SF-40 |
| FR-33 | Persist agent additions, edits and removals immediately and only them; keep coordinator and role rules consistent | UC-40 | SF-41 |
| FR-34 | Fetch model catalogs automatically for built-in agents without one; keep observed catalogs across saves | UC-41 | SF-42 |
| FR-35 | Keep a reviewed plan on saves and catalog changes unless plan-relevant settings change | UC-41 | SF-42 |
| FR-36 | Compact agent cards with provider, sessions, model count, capability and fallback | UC-40 | SF-41 |
| FR-37 | Export workspace settings to a versioned repository file and import them with preview, local-state preservation and capability re-verification | UC-42 | SF-43 |
| FR-38 | Accept 1–12 tasks; add and remove tasks during review | UC-43 | SF-44 |
| FR-39 | Convert a chosen Markdown plan document (≤64 KiB) into a plan through the read-only coordinator, excluding completed items and following role rules | UC-43 | SF-44 |
| FR-40 | Create an accumulating slug per goal with its folder tree in the repository or workspace data folder | UC-44 | SF-45 |
| FR-41 | Plan analysis plus ordered phases; run phases sequentially; move task files through task/verification/completed | UC-45 | SF-46 |
| FR-42 | Derive and display slug, phase and task progress from the tree; detect mismatches | UC-46 | SF-47 |
| FR-43 | Export/import settings JSON with one listed `<slug>.zip` per slug from the same folder | UC-47 | SF-48 |
| FR-44 | Global editable plan rules with workspace override, plus recursively discovered repository Markdown rules when no workspace override exists | UC-48 | SF-49 |
| FR-28 (planned) | Send redacted decision/outcome notifications to Slack or a company messenger; accept exact, signed, linked-approver decisions for pending tool requests | UC-34, UC-35 | SF-37 |

## Non-functional requirements

| ID | Attribute | Requirement | Source | QAS |
|---|---|---|---|---|
| NFR-01 | Security / safety | No merge, push, deployment, external transmission or secret access without an exact human decision; approvals bind exact inputs and context | R-07, R-08, R-30 | QAS-02, QAS-23 |
| NFR-02 | Integrity | Planning and verification never change the target repository; scope escapes are detected | R-05, R-31 | QAS-06, QAS-24 |
| NFR-03 | Recoverability | Supervisor crash, stop and restart preserve artifacts and never duplicate writers | R-09, R-18 | QAS-03, QAS-04, QAS-09 |
| NFR-04 | Availability | Work continues when agentmemory or a paid account is unavailable | R-10, R-25, R-26 | QAS-07, QAS-19 |
| NFR-05 | Performance / resources | Three overlapping workers; core RSS < 100 MiB, idle CPU < 1 % (targets) | R-04, R-12 | QAS-01, QAS-08 |
| NFR-06 | Reliability of I/O | No lost/duplicated events under split UTF-8, large stderr or malformed JSON | R-02 | QAS-05 |
| NFR-07 | Modifiability | New CLIs need only a profile/parser, no scheduler change | R-03 | QAS-12 |
| NFR-08 | Correctness of ownership | One active process per provider session; disjoint ownership | R-19 | QAS-10 |
| NFR-09 | Truthfulness | Quota, status and briefings never fabricate progress or capacity | R-22, R-28, R-29 | QAS-17, QAS-20, QAS-21 |
| NFR-10 | Usability / accessibility | Status readable without colour alone; reduced motion; dialogs reachable by assistive technology; Korean input; every screen has a way back without side effects | R-17, R-29, R-32, R-33 | QAS-21, QAS-25, QAS-26 |
| NFR-11 | Portability | macOS and Windows native builds with equivalent safety primitives | R-23, R-24 | QAS-18 |
| NFR-12 | Distribution compliance | Reviewed licenses, notices and checksums for every released artifact; no bundled third-party trademarks | R-15, R-24, R-34 | QAS-14, QAS-27 |
| NFR-14 | Learnability | A first-time operator can start the desktop workflow from the README without CLI knowledge | R-36 | QAS-28 |
| NFR-15 | Independence | Planning, execution and approval work with no LLM coordinator; a draft launches no process | R-37 | QAS-29 |
| NFR-16 | Discoverability | Every workspace setting is reachable from visible controls without typing paths or using the CLI | R-38, R-39 | QAS-30, QAS-31 |
| NFR-18 | Shareability without leakage | Exported settings are deterministic, LF-terminated and free of secrets and machine-specific state | R-52 | QAS-44 |
| NFR-19 | Consistency of work records | One source of truth (the tree); atomic moves; verifiable archives | R-57, R-58 | QAS-48, QAS-49 |
| NFR-17 (planned) | Confidentiality and integrity of remote channels | Outbound-only by default, secrets in the OS credential store, redaction before sending, signed and replay-protected inbound decisions | R-43 | QAS-35, QAS-36 |
| NFR-13 | Operability | Human decisions do not force regenerated duplicate requests; waits are bounded | R-30 | QAS-23 |

## Constraints

| ID | Constraint | Source |
|---|---|---|
| C-01 | Reuse existing CLI subscriptions; no orchestrator API keys | R-01 |
| C-02 | Single Rust executable with embedded SQLite; no required daemon, Docker or web server | R-12 |
| C-03 | English documents, Korean operator communication, public README without internal plans | R-16 |
| C-04 | TDD with traceable Red/Green evidence; RTK, Ponytail, Graphify, CocoIndex in the workflow | R-13, R-14 |
| C-05 | Provider CLIs may be unverifiable (company CLIs); trust is established by the product, not labels | R-27, R-31 |

### Claude quota collection extension

| ID | Requirement | Use case | Feature / QAS |
| --- | --- | --- | --- |
| FR-45 | Collect Claude quota from owned streams and display the provider-reported windows | UC-49 | SF-50 |
| NFR-20 | Quota checking adds no inference, credentials access or execution authority; missing and stale evidence remain unknown | UC-49 | QAS-51 |

### FR-46 / NFR-21 — Render quota next to names in both lists, select the lowest remaining reported window, explain it in a tooltip, and never start quota queries while rendering. Trace: UC-50.

### Goal-source import extension (planned)

FR-47: register OpenCode-shaped MCP server configuration and source bindings in workspace settings. FR-48: resolve explicit references, preview/import records and feed source-linked evidence into planning. NFR-22: read-only tool allowlists, local provenance, bounded results, no implicit external writes, redacted credentials and independently reviewed imported content. Trace: UC-51, UC-52.

FR-49: manually and periodically refresh each supported agent independently. NFR-23: metadata processes have bounded duration/output and owned process cleanup; scheduling serializes queries to limit host resources and skips active coordinator work. Trace: UC-53.

NFR-05 refinement (R-65, QAS-58, DD-71): idle pages poll every 5 s, stop while hidden and omit icon payloads after the first load.

### FR-50 / NFR-24 Plan-aware compact capacity

FR-50: display slash-separated quota percentages on one line; show provider durations and explicitly reported Enterprise Current/Total aliases in the tooltip. Keep the refresh control adjacent to values. NFR-24: preserve old cache compatibility, accessible refresh names and no invented plan or percentage. Trace: UC-54.

FR-51: persist and validate refresh minutes, suggest executable paths and offer Browse. Apply consistent control sizing and AC native identity. NFR-25: no resident CLI for metadata, no model calls for executable lookup, backward-compatible settings, no execution approval invalidation for cadence alone, no quota loading text. Trace: UC-55.

### FR-52 / FR-53 / NFR-26 Provider switch and reset time

FR-52: provider change replaces all template-derived fields with a unique ID and a located executable; `save_profiles` accepts `renames {old: new}` and rewrites coordinator, role rules and fallbacks. FR-53: exhausted windows (remaining ≤ 0 %) show their reset time in the compact control and every reset appears in the tooltip. NFR-26: no CLI is launched to fill the form (filesystem lookup only); no extra polling for reset times. Trace: UC-56, UC-57.

### FR-54 / NFR-27 Interface language

FR-54: persist `language` (`en` default, `ko`), validate it, expose `set_language`, embed it in both pages (`<html lang>`), translate interface strings and dynamically rendered nodes, and leave textareas, inputs' values, logs and agent output unchanged. NFR-27: UTF-8 everywhere; no per-poll work when the language is English (observer disconnected); translation applies only to changed nodes; no new dependency; files under 500 lines. Trace: UC-58.

FR-55 (R-71): fallback model options come from the fallback agent's catalog; a missing catalog is queued once through the existing discovery queue; the open dialog refreshes without moving focus. NFR-26 applies (no extra polling; one CLI metadata query).

NFR-05 measurement clarification (2026-10-11): "core memory" is measured as macOS **physical footprint** (`footprint <pid>`, the value Activity Monitor shows as Memory) of the main process, with the whole window (main + WebKit GPU/WebContent/Networking helpers) also reported. RSS is recorded for reference only: it counts shared system-framework pages (dyld shared cache `__TEXT`, ~480 MB virtual / tens of MB resident) once per process, so it overstates what the app adds to the host. Windows uses the working-set *private* bytes equivalent.

FR-28 phase 1 (UC-61): per-workspace `messenger` settings (`kind` slack|webhook, `events`, `template`, `include_goal`, `enabled`) travel with the workspace export; the webhook URL is stored only in `~/.agentconductor/messenger-secrets.json` (owner-only permissions) keyed by workspace. Run state transitions and plan completion produce at most one message per state change. NFR-17 phase 1: outbound only, HTTPS only, metadata-only payload unless `include_goal`, the URL never appears in a command line, files under version control or logs; delivery is bounded (10 s per attempt, 2 retries) and never blocks or fails the workflow.

### FR-56 / NFR-28 Windows metadata parity
FR-56: prefer .exe/.cmd/.bat/.com over extensionless npm wrappers, resolve existing profile paths before quota collection, supply installed-artwork fallback badges and a ConPTY-backed Claude collector. NFR-28: bounded output/deadline, process-tree containment, no idle worker or new dependency, UTF-8 evidence, no automatic permission approval. Trace: UC-60.


### FR-57 / NFR-29 Custom catalog registration
FR-57: accept newline-separated unique IDs or observed-catalog JSON; retain configuration timestamp and explicit operator provenance, without claiming CLI discovery/version. NFR-29: no endpoint, API key, unsupported slash command, process or resident worker; bound IDs to 1024 and 256 characters each and reject whitespace/control characters. Trace: UC-62.

FR-58 (UC-63): announce each new pending tool request of a linked running run once with its code (first 8 hex characters of the request SHA-256, case-insensitive); read Slack replies or relay decisions only while a request is pending; accept `allow|deny <code>` only from `approvers`; apply through `tool-approve`/`tool-reject` (300 s approval lifetime, exact binding and single consumption of DD-32); post the outcome; keep a processed-message cursor. NFR-30: polling every 15 s and only while requests are pending, inside the existing run process (no daemon, no inbound port); Slack token and relay secret stored with the webhook URL (local, owner-only, never exported); relay signatures verified with constant-time comparison; messages carry task and tool name only, never the command text.

FR-59 (UC-64): `slugs::suggest` romanizes Hangul syllables (initial/medial/final jamo tables of the Revised Romanization, without assimilation rules), keeps ASCII words, drops other characters, cuts at 48 characters on a word boundary and falls back to `goal_<yyyymmdd>`; `unique` adds `_n`. The page sends no slug. NFR: deterministic, no dictionary or network, no new dependency.

FR-60: normalize Codex individualLimit and credits in addition to primary/secondary windows; show remaining/allocated with conservative freshness and an unknown denominator when missing. NFR-31: no admin API keys, inference prompts or new resident processes; bound numeric strings and do exact decimal arithmetic for allocations. Trace: UC-65.


FR-61: own native pickers in the desktop process; suppress Windows automatic console allocation for the app and all background subprocess paths. NFR-32: reuse OS facilities with no new dependency; retain pipes, job containment and parent-terminal CLI behavior. Trace: UC-66.


FR-62: save existing directories independently of Git, reject files/missing paths without mutating saved settings, and preflight planning before creating slugs. NFR-33: no implicit Git initialization, commit or model call during save; quota remains metadata-only and project-independent. Trace: UC-67.


FR-63: open profile forms before executable lookup; keep unrelated controls active during model/quota collection; queue up to 32 distinct metadata requests for at most 16 profiles; isolate profile snapshot files and apply catalogs only to matching current execution settings. NFR-34: at most one metadata collector at a time, no resident service/new dependencies, no metadata scheduling or observation changes in edit revision binding. Trace: UC-68.


FR-64: baseline status reports missing Git/initial commit; initialize_git requires explicit approved=true and current workspace revision; commit-tree creates an empty tree without reading/changing the index; update-ref requires an absent prior HEAD; record consent/applied commit in workspace-actions.jsonl. NFR-35: no silent commit of existing files, no nested/inaccessible repository replacement, no Git hooks or external transfer during empty setup. Trace: UC-69.


FR-65: support planning, execution, phase continuation, reviewed diffs and approved file application for projects without a committed Git repository. NFR-36: no original Git initialization; snapshot limits of 10,000 files, 16 MiB/file and 256 MiB total; no resident service/new dependency; aliases refused and common build/secret directories excluded; fingerprints prevent overwriting concurrent edits. Trace: UC-70.


FR-66: Settings opens a static Agent tools dialog listing agentmemory, ponytail, rtk, graphify and cocoindex-code with exact commands for Claude Code and Codex and the uv prerequisite; the manual lists the same commands; interface text is translated, commands are not. NFR-37: no network request, subprocess, link navigation, dependency or background work is added; the dialog is plain markup with one open handler. Trace: UC-71.


FR-67: the Windows picker script sets UTF-8 console output without BOM; picker output parsing drops a leading BOM, line endings and trailing separators and refuses non-UTF-8 bytes. NFR-38: no new dependency or process; the picker stays a short-lived OS-provided PowerShell process. Trace: UC-72.


FR-68: prepare, draft, read_plan, next_phase and start requests wait up to 300 s; snapshot job.output reports coordinator stdout/stderr bytes and last write; progress text distinguishes no output, recent output and more than 120 s of silence; reduced motion uses an opacity-only blink. NFR-39: activity costs two file metadata reads per poll and only while a plan job is shown; no extra timer, thread or file read. Trace: UC-73.


FR-69: non-English goals map to goal_<yyyymmdd>; the planner prompt requests an English slug; a valid slug is stored as slug.txt; settle renames only date placeholders, keeps user-requested slugs and avoids existing names and folders. NFR-40: no translation service, model call or dependency is added; the coordinator already reads the goal. Trace: UC-74.


FR-70: Enterprise budgets display Math.round(remaining_percent or remaining/allocated*100)%/100%; zero or missing allocation and credit-only balances display —; unlimited displays ∞; tooltips keep raw amounts. NFR-41: identical function in startup.js and room.js; no extra query. Trace: UC-75.


FR-71: the Windows collector converts \\?\ working directories with git_path before CreateProcessW; claude_blocker stops on trust or first-run screens on macOS and Windows. NFR-42: nothing is typed into those screens; no new dependency. Trace: UC-76.


### FR-72 Readable Windows project paths
Trace: R-87 -> BG-42 -> SF-74 -> UC-77 -> FR-72 -> QAS-83 -> DD-95.
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity.


### FR-73 Reliable named workspace creation
Trace: R-88 -> BG-43 -> SF-75 -> UC-78 -> FR-73 -> QAS-84 -> DD-96.
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable.


### FR-74 Delete an unwanted workspace
Trace: R-89 -> BG-44 -> SF-76 -> UC-79 -> FR-74 -> QAS-85 -> DD-97.
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first.


### FR-75 Register MCP servers in Settings
Trace: R-90 -> BG-45 -> SF-77 -> UC-80 -> FR-75 -> QAS-86 -> DD-98.
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration.


### FR-76 / NFR-47 Reliable and responsive profile confirmation
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


### FR-77 / NFR-48 Durable selectable signal delivery
FR: persist per-workspace preference and expose it regardless of service health; provide a self-managed Rust inbox/outbox; detect failed agentmemory REST or memory/signal MCP delivery; preserve message ID, content binding, recipient session, sequence, correlation and expiry across fallback; reconcile late replies and duplicate delivery; expose effective mode and pending remote delivery.
NFR: Rust owns authoritative state/approval; bounded async I/O and queues do not block UI; no extra daemon, model turn or API credential is required for local delivery. Default new/unconfigured workspaces to Local Rust; memory use is explicitly selected. Use at most an eight-second preferred transport deadline and one in-flight external signal operation per workspace; a failed delivery must settle/fallback or surface pending/error within ten seconds. Reuse actual operation outcomes for health, with bounded backoff rather than continuous probing. Persisted pending messages survive restart; a restored transport must not replay an acknowledged instruction. Other MCP functionality does not acquire an invented local substitute.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### FR-78 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.


### FR-79 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.


### FR-80 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.

### FR-81 Agents work inside the workspace sandbox; the coordinator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Work confined to the task workspace runs at once; a request reaching outside it (network, installs, history-changing Git, privilege, other processes, outside paths, secrets) is decided by the workspace coordinator , and a person is asked only when no coordinator can decide. The permission level is not chosen in the agent profile; it follows from the CLI. Time spent waiting for a decision does not count toward the task limit, and a task that times out can be resumed instead of failing.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.

### FR-82 Usage query accepts the Claude folder-trust prompt
Claude usage must load even when Claude first asks whether to trust the project folder. The usage query runs in a separate session with no tools, hooks or project settings, so it accepts the prompt automatically in both forms (Enter for the default choice, y for the screen-reader "Enter y/n:" form) at most twice, and stops with guidance instead of waiting for a timeout if Claude keeps asking.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.

### FR-83 Planning start and workspace naming feedback
Generate plan shows progress with elapsed time from the moment it is pressed, including while the project folder is copied before the coordinator starts. Saving workspace settings updates the titlebar name at once. Deleting the current workspace asks for the name of the fresh workspace that replaces it instead of always naming it "New workspace".
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.

### FR-84 Run and task status at a glance
The Agent Room shows the run's current final state immediately: a run status banner with the state in bold colour, task progress and the last error, and a prominent Resume button whenever the run can continue. Task states (failed, completed, pending, waiting, running) are bold and colour-coded in the session heading and in the left session list. In the left list a failed task is red, a task waiting for a decision or tool request has a blinking border, pending tasks use a strong colour, and the Tool requests count turns red and blinks while requests wait.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.

### FR-85 Goal folders directly in the project folder
When a workspace has a project folder, each goal (slug) folder is created directly in it as <project>/<slug>/ (for example C:Templadderladder_and_priority_games) with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Goal folders in a former place, including the interim <project>/<workspace name>/<slug>/ of an earlier build that misread the request, move there when first used.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.

### FR-86 Agent CLI list is a master list
The registered agent CLI list is kept as a master list for the machine, not per workspace: deleting a workspace never removes agents, and every workspace opens with the master list. When a workspace refers to an agent the master list no longer has (planning coordinator or role assignments), the assignment is shown in red and can be reassigned; Generate plan stays disabled until every assignment refers to a listed agent.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.

### FR-87 Reorder agent cards
Agent CLI cards can be moved up and down with the mouse (drag and drop), with keyboard-accessible move controls as an alternative. The order is saved, and every agent selection list (planning coordinator, roles, fallback) follows the same order.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.

### FR-88 Readable file paths everywhere
Screens, error messages and notifications never show Windows extended-length path prefixes such as \\?\C:\Work\Temp or //?/C:/Work/Temp; paths read as C:\Work\Temp.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.

### FR-89 Activity tab reads like the CLI
The Activity tab shows agent output the way the CLI shows it: assistant messages as text, tool calls as a one-line action with its key input, tool results as short summaries and errors highlighted. Raw JSON is never shown there; it remains in the Raw logs tab.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.

### FR-90 Coordinator final step in the task list
The Agent Room task list shows the step the coordinator runs after the tasks, integration verification of the combined result, as its own entry with its status (pending, running, completed, failed) and which task tests failed, so a run that fails after every task completed shows why without opening other views.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.

### FR-91 Decide a live tool request from its window
When a tool request window opens while the run is working, Allow once and Deny are available at once, without closing the window with Decide later and deciding again from the Tool requests list.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.

### FR-92 Coordinator decides routine requests and records each decision
Simple decisions are made by the coordinator so the user is not asked often: it allows routine development requests, denies only publishing, outside-project changes, secret exposure or system changes, also for runs started before orchestration existed, and every decision is recorded with the request, the result, the reason and the time, visible in the task's Activity. Commands inside a workspace that itself lives under an app folder (such as ~/.agentconductor) are workspace commands.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.

### FR-93 Task list without account usage
The Agent Room task list does not show account usage badges or refresh buttons; account limits are shown in the inspector's Account capacity section.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.

### FR-94 The coordinator is selectable and shows its work; one name
The interface and documents use one name, coordinator, for the agent that plans, verifies the combined result and decides tool requests (previously also called orchestrator). In the Agent Room task list the coordinator entry is selected like a task: the center shows its heading, status and an updating Activity of its run steps, tool decisions and integration test results, and Raw logs show the integration test output, following the Agent room design (select on the left, progress in the center, evidence on the right).
Trace: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117. Source: user request, 2026-10-13.

### FR-95 Failure cause and the next action on a failed run
When a run fails, the status banner shows why (for an integration verification failure, the failed task and the first lines of its test output) and offers Fix with feedback, which selects the failed task and drafts a direction with the failure output for the user to review and send; sending it reruns the task and repeats verification.
Trace: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118. Source: user request, 2026-10-13.

### FR-96 The next step is always shown
The Agent Room status banner always says what to do next and offers that action: review the changes when the run waits for approval (Approve stays unavailable until the latest evidence is reviewed), approve after review, apply or merge after approval, and return to the workspace to plan the next phase when the run is complete.
Trace: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119. Source: user request, 2026-10-13.

### FR-97 Plan review and run completion are organised by phase
A phased goal shows its phases in Review the plan: which phase is under review (phase n of m), which are completed, which comes next, and a Plan phase n action when the next phase is ready to load. When a run finishes, the Agent Room says which phase completed and which phase is next, and that it is planned from the workspace; a completed phase is recorded even if the goal folder moved after the run started.
Trace: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121. Source: user request, 2026-10-13.

### FR-98 A blocking verification finding fails integration verification
When a verification report changed in the run records a blocking issue (a Blocking/차단 section with items, or a positive blocking-issue count), integration verification fails like a failing test, naming the report and the task responsible for the files the issue names, so the coordinator does not accept a candidate its verifier rejected and Fix with feedback drafts the correction for that task.
Trace: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120. Source: user request, 2026-10-13.

### FR-99 The run's phase beside the workspace name
The Agent Room titlebar shows, beside the workspace name, the phase the run executes (phase n of m and its name) and what it covers (the phase description, or its task list when the phase file has no description); the full text is in its tooltip.
Trace: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122. Source: user request, 2026-10-13.

### FR-100 Readable Agent Room text and agent icons sized to their names
The Agent Room's text is at least 11 px and scaled up in proportion (body text 14 px instead of 12 px) for readability, workspace screen text is at least 11 px, and the agent icons in the task list are the height of the name beside them instead of a large tile.
Trace: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123. Source: user request, 2026-10-13.

### FR-101 Task cards in Review the plan fold to one line
In Review the plan each task is folded by default, showing only its number, name and a one-line description (the first line of its instructions that is not a label); an arrow unfolds it to show and edit every field.
Trace: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124. Source: user request, 2026-10-13.

### FR-102 Review the whole phased plan up front
The coordinator plans every phase of a goal when the plan is generated, so Review the plan briefs each phase (what it will do, from the coordinator's analysis) and lists each phase's planned tasks, with the coordinator's analysis of the whole goal available; the phase about to run is open and its tasks are edited below, completed phases are marked, and later phases can be unfolded to review their tasks before any of them runs.
Trace: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125. Source: user request, 2026-10-13.

### FR-103 A run survives closing or losing the desktop app
Closing the desktop app (or the app process ending) must not end a run: the run coordinator and its agent workers keep working and the app reattaches when reopened. Closing the window while a run is active asks: Keep running in the background (default), Stop the run and close, or Cancel.
Trace: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127. Source: user request, 2026-10-13.

### FR-104 Interrupted runs recover and resume automatically
When a run is recorded as running but its coordinator and workers are gone (app killed, crash, reboot), opening the workspace or the Agent Room recovers it automatically (verifying every recorded process is absent, as Recover does) and, with automatic resume on by default, resumes it: completed tasks are kept, the interrupted task continues in its own native session (or a new one if the session cannot resume) with a note that its previous attempt was interrupted and its owned files must be inspected. After two consecutive automatic resumes that are interrupted again, it stops and asks the operator.
Trace: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128. Source: user request, 2026-10-13.

### FR-105 Interruptions and silent workers are visible
The Agent Room shows when a run was interrupted and recovered (banner history and coordinator Activity), collapses a worker's consecutive thinking_tokens events into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows, and warns when a running task has produced no output for several minutes or its process is gone, suggesting Recover.
Trace: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129. Source: user request, 2026-10-13.

### FR-106 Reassign tasks that have not started while the run is working
While a run is working, the operator can change the agent (and model) of a task that has not started yet, for example to move work from a busy Claude account to Codex; the change applies before that task starts. Running and finished tasks still need the run to be idle.
Trace: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130. Source: user request, 2026-10-13.

### FR-107 Agents run with the highest permission by default
Agent workers run with the highest permission their CLI offers by default (Claude without permission prompts and with the operator's MCP servers such as Playwright, so it can open and inspect a browser UI; Codex without approvals or sandbox), while the planning and review coordinator stays read-only. The operator can lower a profile's permission.
Trace: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131. Source: user request, 2026-10-13.

### FR-108 The coordinator resolves verification failures
When every task of a run has completed but integration verification fails (a failing test command or a verification report that records a blocking issue), the run does not stop at Failed waiting for the operator: the coordinator reviews the evidence, revises the tasks whose files must change and the verifying task, and the run resumes. A finding the run cannot resolve (outside every task scope, or needing an environment the agents lack) is recorded by the verifying task as a deferred non-blocking follow-up with its reason and owner. After three repair rounds without passing, the run stays Failed with the reason, for the operator to revise or reassign. Resume on such a failed run starts this repair.
Trace: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132. Source: user request, 2026-10-13.

### FR-109 Selected-folder execution with conflict-safe restoration
Resolve ordinary-folder Git operations through an external project binding, capture the baseline before launch, keep provider cwd and test cwd at the selected root in shared mode, and recheck exact files before Accept or Reject. Add persistent completed-worker fingerprints and an active-owned-path warning. Treat process attribution as unknown: fingerprints indicate a changed checkpoint, not who edited it.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user workspace-path requirement and Claude review, 2026-10-13.

### FR-109 Deleting workspaces needs no replacement name
Deleting a workspace asks for no new workspace name. When the current workspace is deleted, the most recently used other workspace opens, so several workspaces can be deleted in a row; only when none is left does one empty workspace start.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user request, 2026-10-13.

### FR-110 Validate both language references
Check each manual against executable help; require real linked screenshot files and README packaging references. Assert runtime errors through current message catalog entries rather than English literals.
Trace: R-125 -> BG-80 -> SF-112 -> UC-115 -> FR-110 -> QAS-121 -> DD-134. Source: user release/manual requests, 2026-10-14.

### FR-111 The app updates itself from GitHub releases
Every 10 minutes the app checks the latest GitHub release in the background. When a newer version exists, an update icon, a downward arrow, appears left of Settings; the icon and the workspace menu item Check for updates open the same dialog with the installed and latest versions. Update and restart downloads the archive for this platform, verifies it against the release SHA256SUMS, replaces the installed executable (or macOS app bundle) and restarts the app. It is refused while any run or app job is active, because restarting ends their agents. The update repository defaults to the project's GitHub repository. Settings can change it to a GitHub `owner/name` or an HTTPS repository URL on any GitHub-compatible host, such as an in-house GitHub Enterprise server; release notes, the archive and `SHA256SUMS` come from that repository's latest release, and only the repository is configured, not the download path (#130).
Trace: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135. Source: user request, 2026-10-14.


### FR-112 Availability deadline and fallback lifecycle
Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

FR-113 (UC-118): persist `theme` (`light` default, `dark`) in `preferences.json`, embed it as `data-theme`, expose `set_theme`, apply it without reload; no hard-coded colors outside the token blocks; session status tags wrap within the rail. NFR-49: no new fonts, images or animation; contrast ≥ WCAG AA for body text in both themes; no extra polling.

### FR-114 Plans know whether the project uses Git, and verification commands that cannot work are replaced
The planning coordinator is told whether the project folder is a Git repository and does not plan git test commands when it is not. If a planned test command cannot work in the project (for example git in a folder without Git) or an agent reports that its test command must change, the coordinator review replaces it with a command that verifies the same work instead of deferring the failure, and the replacement is recorded. Test commands of a folder without Git run against the history AgentConductor keeps outside it.
Trace: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138. Source: user request, 2026-10-14.

### FR-115 Compact quota and step text

Badge text remains percentages only, including exhausted windows. The tooltip retains reset times, observation time and source; refresh retains an accessible name. Text is 11px with a matching 11px icon. Descriptions wrap responsively without forced no-wrap overflow. No extra query, polling or dependency.
Trace: R-130 -> BG-85 -> SF-117 -> UC-120 -> FR-115 -> QAS-126 -> DD-139. Source: user, 2026-10-14.

### FR-116 Conditional server controls

Synchronize port-label visibility when preferences load and transport changes, without clearing its value. Keep Settings scrollable on smaller windows. Move the guide entry without adding another dialog or runtime dependency.
Trace: R-131 -> BG-86 -> SF-118 -> UC-121 -> FR-116 -> QAS-127 -> DD-140. Source: user, 2026-10-14.

### FR-117 Custom CLIs that approve their own tools can run
A custom agent CLI that approves its own tool use (for example one set to auto-approve tools) can be marked so in its agent settings; it then runs after plan approval instead of being refused as unverified. The mark is the operator's claim for this machine and is not trusted when settings are imported from another machine.
Trace: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141. Source: user request, 2026-10-14.

### FR-118 Safe release text

Use the existing asynchronous GitHub response; no extra requests or Markdown dependency. Render remote body as literal text, retaining newlines and wrapping long lines. Clear stale content when no body exists.
Trace: R-133 -> BG-88 -> SF-120 -> UC-123 -> FR-118 -> QAS-129 -> DD-142. Source: user, 2026-10-14; next release after v0.1.18.

### FR-119 Automatic read routing and retained Markdown sources

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.


### FR-120 Immediate failure reset and active CLI visibility

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.


### FR-121 Normalized approval readiness and API restriction persistence

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.


### FR-122 / FR-123 Narrow recovery and offline localized guidance

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

### FR-124 Workspace and Agent Room feedback refinements
Step 1 shows the planning coordinator right of the workspace name and the project folder above the goal, without a divider. MCP lookups show that they are running; Preview opens its window at once with a progress pulse and fills it when the lookup ends or shows the failure. jira:KEY and confluence:ID references in the goal are shown as tags that stay plain, deletable text, and hovering one shows its title and summary. The goal heading status follows the started run instead of staying Started. In the Agent Room, account usage shows Enterprise budgets as a remaining percentage like the agent cards, and the task shown on the right has a strong border in the list.
Trace: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148. Source: user requests, 2026-10-14.

### FR-125 Runs keep going and stay visible
A model reply such as "Selected model is at capacity" is retried after a short wait, at most three times, for tasks and for the coordinator. Files an agent creates outside its owned paths in a project without its own Git (for example a CLI saving .aipro/plans) are moved out of the project into the run folder and recorded. Agent Room task rows lead with the reviewed task summary. A pressed button shows that its work is running, distinct from a disabled button, until the request and any background job it started end. The workspace page and the Agent Room stay open side by side: the operator switches freely, the room keeps its live progress, and the workspace shows the run state on its back-to-run button.
Trace: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149. Source: user requests, 2026-10-14.


### FR-126 Authoritative routing guidance / FR-127 no per-click shell startup

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

### FR-129 Semantic and lazy manual display

Generate tables/headings/code/links and embedded local PNGs. Escape raw markup and reject executable link schemes or image paths outside assets. Only en/ko are accepted by manual_document. No runtime parser dependency, web server or network load; cache after first manual request.
Trace: R-144 -> BG-99 -> SF-131 -> UC-134 -> FR-129 -> QAS-140 -> DD-153. Source: user, 2026-10-14.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

### FR-132 Preserve live jobs and completed evidence across navigation

Only quiescent retained rooms are detached after successful workspace transition; navigation failure keeps the old room.
Trace: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156. GitHub #4, user2026-10-14.

### FR-133 Prompt-aware bounded metadata collection

Keep the existing 20-second and 1-MiB bounds, generation-free command and owned-process cleanup. Report whether timeout happened before command submission or while waiting for usage.
Trace: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157. GitHub #5, user2026-10-14.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

### FR-135 Isolate goal context and preserve history

New/selected goals revoke old proposals and source bindings without deleting trees, diffs or logs. Active/nonterminal runs must finish or stop before goal changes. New project assignment cannot create a second owner; existing duplicate data is retained and new execution uses the deterministic primary owner.
Trace: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159. GitHub #6, user2026-10-15.

### FR-136 Keep application, UI interaction, and release tooling in Rust
Replace project-authored JavaScript UI logic and Python release/manual/verification scripts with Rust implementations. Preserve the existing user workflows, keyboard/accessibility behavior, offline documentation, cross-platform package contents, and low-resource target. No interpreter for project-authored JavaScript or Python may be needed to run or build the release. External agent CLIs and operating-system runtimes are explicit integration boundaries.
Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160.

### FR-137 Keep planning startup requests and recovery coherent
Pass the configured bounded planning timeout through workspace save, baseline inspection, approved baseline actions, and automatic snapshots while planning is starting. Keep recurring snapshots active after a timeout. Localized timeout text must not refer to a manual refresh control that the screen does not provide.
Trace: R-152 → BG-107 → SF-139 → UC-142 → FR-137 → QAS-148 → DD-162. GitHub #36, 2026-10-19.

### FR-140 / NFR-52 Keep company labels local
FR-138: read the validated optional suffix from `company-suffix.txt` for both application titlebars while preserving Cargo's numeric version for release/update comparison. NFR-50: refuse package assembly before archive creation when the suffix contains a non-whitespace value.
Trace: R-155 → BG-110 → SF-142 → UC-145 → FR-140/NFR-52 → QAS-151 → DD-165. GitHub #13.
### FR-139 Configure custom CLI event and quota adapters per profile
For a custom profile, map raw JSONL event names to the standard event kinds and optionally identify session IDs with a JSON Pointer. Optionally invoke an argv-only quota command with a bounded timeout and extract a normalized Snapshot from stdout using a JSON Pointer. Preserve these fields in workspace JSON export/import, retain process logs, and keep existing built-in provider behavior unchanged.
Trace: R-154 → BG-109 → SF-141 → UC-144 → FR-139 → QAS-150 → DDR-17 → DD-164. GitHub #12, 2026-10-19.
### FR-138 Keep task timeout recovery visible and stoppable
On a wall-timeout, retain attempt artifacts, increment timeout evidence, keep the task pending, and set a retry time using a bounded backoff no longer than 60 seconds. Automatically resume due tasks while the run is active; honor `stop.request` before launching again. Persist an audit event and expose timeout/error details in the report during retry and at blocked completion.
Trace: R-153 → BG-108 → SF-140 → UC-143 → FR-138 → QAS-149 → DD-163. GitHub #44, 2026-10-19.

### FR-141 Show the final apply stage and its progress
Expose Verify, Merge, and Workspace as consecutive steps after task execution. Keep latest-evidence review and human approval separate from final application. While applying an approved candidate, present a persistent localized busy status and tell the operator completion returns to the workspace.
Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166.

### FR-142 Wait briefly for concurrent SQLite writers
Run database connections use the same finite ten-second busy timeout. On transient contention, SQLite retries the blocked operation within that bound; if the bound expires, return the database error without discarding already committed state or silently replaying a non-idempotent write.
Trace: R-157 → BG-112 → SF-144 → UC-147 → FR-142 → QAS-153 → DDR-18 → DD-167. GitHub #56.

### FR-143 Describe custom CLI model, argument, and result behavior
Custom profiles shall retain manual model catalogs and run model discovery only when configured with direct argv and response JSON Pointers. Execution option groups shall support templates, allowed values, optional defaults, and per-model restrictions. Resolve them from profile defaults or per-task overrides, omit unresolved optional groups, preserve task values across session resume, and reject reassignment when the destination profile or selected model does not accept them. Extract result payloads by configured JSON Pointer and classify matching pointer/value conditions as errors before event normalization. Editing and workspace JSON export/import shall round-trip these declarations.

### NFR-53 Keep CLI extensions local and provider-neutral
Custom behavior shall run through the registered executable and its existing authentication, with no API key requirement, HTTP model service, shell interpolation, new provider enum variant, or provider-specific application branch. Unconfigured custom discovery shall not spawn a process. Existing built-in CLI behavior and older profiles without these fields shall remain compatible.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62.

### FR-144 Report activity, current tool, last output age, and process state
For each running task the snapshot carries the last event kind and observation time, the current tool name and start time (Claude tool_use/tool_result, Codex command, MCP and file-change items, custom tool events), and the owned-process state. The page renders elapsed times from those timestamps once per second.

### NFR-54 Activity display costs no extra polling and never asserts a cause
Reuse the existing snapshot poll and recorded task activity; add no new background loop or process probe beyond the existing per-poll owned-process check. Silence is reported as elapsed time only. Animation respects reduced-motion settings.
Trace: R-159 → BG-114 → SF-146 → UC-149 → FR-144/NFR-54 → QAS-155 → DDR-20 → DD-169. GitHub #100, user, 2026-10-21.

### FR-146 Answer operator questions from project files and the manual
Accept 1..8192-byte operator turns, send at most the 12 most recent turns, require the conversation to end with an operator turn, and return one bounded Markdown answer (32 KiB). Show failures as an error turn that is not sent back.

### NFR-55 Keep the chat read-only and bounded
The coordinator runs only through its verified read-only template, never changes files or runs commands, shares the single job slot (a running operation refuses a new question), and times out after five minutes. The page renders answers as plain text.
Trace: R-161 → BG-116 → SF-148 → UC-151 → FR-146/NFR-55 → QAS-157 → DDR-22 → DD-171. GitHub #115, user, 2026-10-21.
### FR-145 Move the selection when the watched session stops running
Only a selected session that was running and no longer is triggers the move; a session selected after it finished, and the integration entry, are never taken away. The first snapshot never moves the selection.
Trace: R-160 → BG-115 → SF-147 → UC-150 → FR-145 → QAS-156 → DDR-21 → DD-170. GitHub #109, user, 2026-10-21.

### FR-147 Preview, approve and apply a scope change
The preview validates the plan (overlap and unsafe-path rules), lists previous and new paths and test command, and classifies each quarantined file within the new scope as restore, already present or conflict. Apply requires the preview's digest, copies only restorable files (the recovery copies stay), records the change in the audit log, and revokes integration approval.

### NFR-56 Never overwrite or silently widen
A conflicting destination is never overwritten, feedback never edits owned paths, an unreviewed or changed proposal is refused, and no step edits internal state by hand.
Trace: R-162 → BG-117 → SF-149 → UC-152 → FR-147/NFR-56 → QAS-158 → DDR-23 → DD-172. GitHub #122, user, 2026-10-22.

### FR-148 Triage a failed task through the existing review
When a run is failed, has a failed task, no task waiting for availability, and its error has no waiting task, the recorded coordinator reviews the frozen evidence (task status, error, stdout and stderr excerpts) and a revise verdict resets the task and its dependents and resumes the run. A blocked verdict or an unusable review leaves the failure for the operator.

### NFR-57 Bounded and fail-closed triage
Triage uses the same three review rounds as integration repair, never runs without a recorded coordinator, never starts after a Stop, never changes owned paths, and never approves a tool request or a merge.
Trace: R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173. GitHub #135, user, 2026-10-22.

### FR-149 Record, show and share step lessons
A lesson has a task, a cause and a response, each clipped to 1 KiB; the same task and response are recorded once; the run keeps the newest 50; the prompt receives at most 4 KiB, newest first, for every task; the task detail lists them all; each lesson is one memory outbox entry.

### NFR-58 Evidence only and failure-tolerant
Lessons never change scope, tool permission or merge approval; a failed agentmemory delivery never fails the step and the local record stays authoritative.
Trace: R-164 → BG-119 → SF-151 → UC-154 → FR-149/NFR-58 → QAS-160 → DDR-25 → DD-174. GitHub #138, user, 2026-10-22.

### FR-150 Fold, remember and open on a decision
A decision is open when the run state is waiting_approval, waiting_tool_approval or blocked, a tool request is pending, or the failure is an out-of-scope change. A decision key (state plus candidate and pending tool request identities) opens the column once when it appears or changes; the toggle sets the remembered preference; when the key disappears the column follows the preference. While a decision is open the candidate review is the first section.

### NFR-59 Quiet, accessible and failure-tolerant
The warning dot uses the existing warning token, blinks only while a decision is open and the column is folded, does not blink when reduced motion is requested, and has an accessible name; the toggle exposes aria-expanded; an unavailable local storage leaves the column open without errors.
Trace: R-165 → BG-120 → SF-152 → UC-155 → FR-150/NFR-59 → QAS-161 → DDR-26 → DD-175. GitHub #146, user, 2026-10-22.

### FR-151 Gates, violation codes and amendments
Violation codes: scope_overlap, scope_escape, test_unrunnable, tool_missing, timeout, plan_unsatisfiable, goal_drift. Amendments are typed, validated against the charter, bounded by a budget per run, applied by tier and reversible.

### NFR-60 Deterministic checks, bounded loops
Validation never depends on a model, every loop has a budget, and a run never repeats the same block without a recorded decision.
Trace: R-166 → BG-121 → SF-153 → UC-156 → FR-151/NFR-60 → QAS-162 → DDR-27 → DD-176. GitHub #149, user, 2026-10-22.

### FR-152 One decision list from existing records
A judgment row has a kind, a judgment, the task it concerns (if any), a detail and the record's summary. Rows come from the run's audit records of tool decisions, reviews, replaced test commands, time-out resumes, automatic recoveries, hand-offs and lessons, and later launch-gate and amendment records. Unknown records are ignored.

### NFR-61 Cannot be hidden
There is no setting that hides judgments, the list is complete up to a documented limit, and the count is cheap enough for the snapshot.
Trace: R-167 → BG-122 → SF-154 → UC-157 → FR-152/NFR-61 → QAS-163 → DDR-28 → DD-177. GitHub #149, user, 2026-10-22.

### FR-153 Resolve every executable the run will start
Distinct agents of the tasks and each task's test command program are resolved with the app's executable search (PATH, per-user tool folders), an absolute path, or a path relative to the project root. Empty commands are violations.

### NFR-62 Side-effect free
The gate runs nothing, writes nothing and creates no run folder when it blocks; it adds no dependency and costs a file-system lookup per executable.
Trace: R-168 → BG-121 → SF-155 → UC-158 → FR-153/NFR-62 → QAS-164 → DDR-27 → DD-178. GitHub #149, user, 2026-10-22.

### FR-154 Amendments, risk and budget
Valid amendments pass `Plan::validate` (safe, non-overlapping scopes; known, acyclic dependencies). High risk: a single-component path without an extension (a top-level directory) or a sensitive path (.env, .ssh, .aws, keys, secrets, credentials). The budget is five applied amendments per run. Applied, escalated and undone amendments are audit records listed as judgments with their reason; applied ones carry the previous values.

### NFR-63 Reversible and bounded
Undo restores exactly the recorded previous paths and dependencies, revokes approvals and leaves the run stopped for an explicit Resume; an escalation never changes the plan.
Trace: R-169 → BG-121 → SF-156 → UC-159 → FR-154/NFR-63 → QAS-165 → DDR-29 → DD-179. GitHub #149, user, 2026-10-22.

### FR-155 Charter content and drift
The charter holds the goal and, per task, id, owned paths, test command and dependencies, with a status (recorded or updated). Drift is the number of task-owned paths not in the charter entry of that task. The limit is eight; an amendment that would exceed it escalates with goal_drift. Applied amendments carry the drift after the change; undoing one lowers it.

### NFR-64 Cheap and tamper-evident
The charter is one audit record written once per run (and once per approved scope change); drift needs no model and no file access; the coordinator cannot edit the charter.
Trace: R-170 → BG-121 → SF-157 → UC-160 → FR-155/NFR-64 → QAS-166 → DDR-30 → DD-180. GitHub #149, user, 2026-10-22.

### FR-156 Assert readiness before lifecycle actions
Asynchronous worker tests shall observe the requested task's ready marker and active attempt/session state before issuing a directive. Process-tree tests shall observe a descendant marker before verifying termination and direct-child reaping.

### NFR-65 Bounded readiness with actionable diagnostics
Every readiness wait has a generous explicit deadline, polls the required condition, and reports retained process/workflow evidence on failure. Tests must not rely on fixed startup sleeps or require a busy wait.
Trace: R-171 → BG-123 → SF-158 → UC-161 → FR-156/NFR-65 → QAS-167 → DDR-31 → DD-181. GitHub #156, 2026-10-22.

### FR-157 Route, run and show a coordinator request
Messages are 1 byte to 8 KiB without NUL; a coordinator must be recorded for the run; the request waits for the run lock instead of failing; the review prompt carries the message as operator text, not evidence; an accept verdict with no revisions is allowed for a request even without a verified candidate; the three-round automatic limit does not block an operator request; a revise verdict resumes the run unless it escalated. Each state is an audit record listed as a judgment.

### NFR-66 No silent restarts and separate drafts
A request never restarts a completed task by itself, never grants tool or merge permission, and a failure is recorded with its error.
Trace: R-172 → BG-124 → SF-159 → UC-162 → FR-157/NFR-66 → QAS-168 → DDR-32 → DD-182. GitHub #165, user, 2026-10-23.

### FR-160 Review rules and exclusions are written down
The policy states sixteen review rules for new and changed code, the exclusions with their reasons, the measured baseline with the commit it was taken at, and the checks that remain authoritative (`cargo fmt --check`, warning-denied Clippy, `tests/source_layout.rs`).

### NFR-70 No churn and no new tooling without a demonstrated gap
The policy rewrites no existing code, adds no unstable rustfmt option, no `rust-toolchain` pin and no lint that fails on current code. A new check needs a measured gap, red-first tests and its own issue.

### C-06 Stable toolchain and unchanged line limits
The policy works on the stable toolchain with the default rustfmt and Clippy settings. `main` keeps the 500-line limit enforced by `tests/source_layout.rs`; the Rust migration line keeps 300 lines for changed Rust files. Test bodies stay under `tests/`.
Trace: R-174 → BG-129 → SF-162 → UC-165 → FR-160/NFR-70/C-06 → QAS-171 → DDR-34 → DD-185. GitHub #169, user, 2026-10-23.

### FR-161 Safety comments and the gate
Each comment names the invariant (valid pointer and lifetime, handle ownership, thread affinity, why the call cannot violate the contract), not a restatement of the call. The lint step is added in the same change that brings the violation count to zero on macOS and Windows.

### NFR-71 No behavior change and no new tooling
Comments and one CI step only; no refactoring of unsafe code beyond removing two avoidable `unsafe` sites (a repeated `geteuid` call and a struct-update `zeroed`), no new dependency and no other lint. Each changed Rust file stays within its line limit (500 on `main`, 300 on the Rust line).
Trace: R-175 → BG-130 → SF-163 → UC-166 → FR-161/NFR-71 → QAS-172 → DDR-35 → DD-186. GitHub #172, user, 2026-10-24.

### FR-162 Helper and gate
The helper offers `lock()`, `set` and `remove`, keeps the first original value of each variable and restores it on drop even when the test panics. A poisoned lock is recovered because the mutex guards no data. The CI gate runs `--lib --bins --tests` on macOS and Windows.

### NFR-72 No behavior change
Test assertions and the code under test stay as they are; only how variables are set changes. Each changed Rust file stays within its line limit (500 on `main`, 300 on the Rust line). The helper serializes mutating tests only: it cannot stop a non-mutating thread of the same binary from reading the environment, which is recorded as a documented limit.
Trace: R-176 → BG-131 → SF-164 → UC-167 → FR-162/NFR-72 → QAS-173 → DDR-36 → DD-187. GitHub #176, user, 2026-10-24.

### FR-163 Skill prompt and result
The coordinator prompt holds the unmodified skill text and the references it points to for an existing prompt, the operator's goal and the interface language, and asks for one JSON object with `optimized` (the rewritten goal, in the language of the original), `changes` (short statements of what changed) and `questions` (what the operator should confirm). The coordinator is read-only (Read, Grep, Glob): it can inventory the project files the skill asks it to list but cannot change anything. The skill's eval loop is not run, because no extra model calls are made; the skill is applied as a single pass.

### FR-164 Review and apply
The workspace screen shows Optimize prompt beside the goal field, disabled while the goal is empty or another job runs. The dialog shows the proposal, the changes and the questions in the interface language. **Use this goal** puts the proposal in the goal field and marks the workspace as changed, **Keep mine** closes the dialog, and neither saves or runs anything.

### NFR-73 Bounded, attributed, one job at a time
The goal is limited to 8 KiB, the proposal to 16 KiB, at most 12 changes and 8 questions of 400 characters each, and the job times out after five minutes. It uses the existing single job slot, so it cannot overlap planning, a run or an MCP preview. The vendored files are byte-identical to the pinned upstream commit, and the Apache-2.0 license text and a notice naming the origin ship in the release package. Each changed Rust file stays within its line limit (500 on `main`, 300 on the Rust line).
Trace: R-177 → BG-132 → SF-165 → UC-168 → FR-163/FR-164/NFR-73 → QAS-174/QAS-175 → DDR-37 → DD-188. GitHub #180, user, 2026-10-27.

### FR-165 Settings opens the guide and the manual
`start-tools-guide` and `start-manual-open` in the Settings dialog close it with `cancel` (nothing is saved) and open the existing `start-tools` dialog and `AC_MANUAL`. The workspace menu builds neither entry.
Trace: R-178 → BG-133 → SF-166 → UC-169 → FR-165 → QAS-176 → DD-189. GitHub #184, user, 2026-10-27.

### FR-166 Stored conversation
The `chat` request carries only the new question; the server rebuilds the earlier turns from the stored artifacts of the latest chat (`chat-input-<id>.json` plus its answer), keeps the last 40 turns, and `chat_history` and `chat_reset` read and clear them. An earlier long answer is clipped in the prompt instead of refusing the next question.

### FR-167 Transcript behavior
The panel loads the history when the workspace opens, adopts a chat that is still running, shows the answer once, keeps typed input when a request is refused, shows elapsed time while waiting, and is cleared when another workspace opens.
Trace: R-179 → BG-134 → SF-167 → UC-170 → FR-166/FR-167 → QAS-177 → DD-190. GitHub #185, user, 2026-10-27.

### FR-168 Removal
`startup.html` and `room.html` have no `local-dot`, `local-label`, `startup-note` or the footer default-mode span; the keys `startup.local_existing_cli_login`, `startup.local_by_default`, `startup.one_application_independent_agent_sessions_your_approval_controls`, `startup.shared_by_default_worktrees_optional` and `room.local_cli_authentication` are gone from both catalogs. The workspace mode stays visible in its checkbox.
Trace: R-180 → BG-135 → SF-168 → UC-171 → FR-168 → QAS-178 → DD-191. GitHub #190, user, 2026-10-27.

### FR-169 Reconcile on failure
`send` in `chat.js` calls `resync` on any rejection: it reloads `chat_history`; if the last turn is the operator text that was sent, the turn stays and `settle` adopts the running or completed job; otherwise the optimistic turn is removed and the text and error are shown. If the history cannot be read either, the optimistic turn is removed as before.
Trace: R-181 → BG-136 → SF-169 → UC-172 → FR-169 → QAS-179 → DD-192. GitHub #197, user, 2026-10-28.
