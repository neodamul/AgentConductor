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
