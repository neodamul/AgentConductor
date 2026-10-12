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
| FR-44 | Global editable plan rules with workspace override, passed to the coordinator | UC-48 | SF-49 |
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
