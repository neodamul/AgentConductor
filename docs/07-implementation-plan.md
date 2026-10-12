# Implementation plan and TDD

## Reconciled current baseline (2026-10-11)

- [x] Plan: review Claude source `318f930`, added/amended R-36–59 and trace through DD-65; preserve original constraints and latest task/release policy.
- [x] Task: fetch and fast-forward the clean canonical checkout from `36d3420`; prior Codex corrections are already in history, not replayed.
- [x] Verify: 16 focused local tests (plan size, roles, slugs, settings sharing, 500-line layout) passed.
- [x] Verify: preview.6 has actual macOS/Windows ZIP assets and checksums; latest source CI 38112177633 jobs were not started because of GitHub account billing/spending limits.
- [ ] Complete: R-55–59 remaining increments and native acceptance; refer to the slug checklist below.

Detailed review: [requirements reconciliation](../knowledge/wiki/requirements-reconciliation-2026-10-11.md).
The historical checkpoints below retain their original scope and do not describe current implementation status.


Historical DD-41 checkpoint: modular source/test separation, 2026-10-10. Full macOS
regression: 213 passed, 0 failed, 9 ignored; 57 suites. Files are bounded at 500 lines.
Canonical development checkout is `/Users/neodamul/Development/AgentConductor`.
Platform CI `38038926444` on `0ab97d8` is terminal: macOS passed; Windows tests and
formatting passed, but Clippy rejected two `&PathBuf` fixture parameters. These are
corrected to `&Path`; native Windows verification of this checkpoint remains pending.
Previous checkpoint details below retain their original source scope.

Current status: full product under development. Cross-agent reconciliation on 2026-10-10
confirmed remote `2211ba3`, 19 commits after the preceding Codex checkpoint `3eb669c`.
CI `38037224504` on `bbabefe` completed: macOS passed; Windows built successfully,
including passing Job Object containment and WebView2 smoke tests, but failed
`gates::simultaneous_callbacks_cannot_consume_one_human_approval_twice` during directory
cleanup with Windows error 32. The reopened SQLite store must close before cleanup.
The following historical checkpoint remains evidence for its original source only.

Latest earlier verified slice: Claude Code CLI
integration (subscribed native sessions, guarded three-worker workflow and a read-only
Claude planning coordinator), based on continuation checkpoint `cff4646`.
Reviewed on 2026-10-10. The latest full regression passed 201 tests, with 8 ignored and
0 failures across 51 suites. The ignored live Claude tests (`native_claude`,
`native_claude_workflow`) passed separately against a claude.ai-subscribed CLI.
The existing-agentmemory-daemon test passed separately at the preceding transport checkpoint;
it was not repeated in this slice.
Clippy with warnings denied, formatting and whitespace checks passed at that implementation checkpoint.
Evidence: `.agentconductor/verification/cooperative-continuation/`,
`.agentconductor/verification/cooperative-recovery/` and
`.agentconductor/verification/cooperative-exit/`, with preceding worker/transport
checkpoints retained separately.

The native macOS interface has been exercised with controlled CLI fixtures, including
human review, team briefing and status lights. This does not establish authenticated
all-provider acceptance or native Windows support. The latest observed CI run,
`38027108692` on `c4f6291`, completed with macOS success and a Windows build failure
(E0425: Unix-only `process::run_controlled_async`, plus four platform warnings).

### Current checkpoint and remaining acceptance

### Cross-agent continuity and Windows cleanup (2026-10-10)

- [x] Plan: reconcile Claude handoff with committed source, agentmemory and actual CI.
- [x] Task: preserve the preceding Codex working changes in a named Git stash; fast-forward to `2211ba3` without replaying the already integrated continuation code.
- [x] Task / Red: Windows CI failure at `tests/gates.rs:172`; simultaneous approval assertions passed before cleanup failed.
- [x] Task / Green: explicitly drop the reopened SQLite store before cleanup.
- [x] Verify: focused macOS gate regression (4 passed, 0 failed, 0 ignored).
- [x] Verify: native Windows CI for the cleanup correction (windows-latest CI 38056911056 and Release run 38069707257; `gates` passes on Windows).
- [x] Complete: preserve source/CI evidence and shared agentmemory handoff (`mem_mv25cgtz_2a8a07bee3cc`).
- [x] Verify: Rust formatting and whitespace checks.

Cross-agent handoff procedure: [development continuity](12-development-continuity.md).
The Windows CI result above supersedes the older Windows build-failure observations below.

| Area | Verified checkpoint | Remaining work |
|---|---|---|
| Rust execution and review | Parallel subprocess fixtures, shared workspace default, optional worktrees, durable SQLite state, diff/test evidence and exact candidate approval | Authenticated multi-provider end-to-end acceptance and platform containment |
| Sessions and account limits | Session controls, restriction/reset scheduling, reviewed per-agent fallback contracts, actual Codex quota observation and claude.ai-subscribed Claude CLI generation, exact-session resume and concurrent-session isolation (`tests/native_claude.rs`), and a three-worker guarded Claude validate/run/approve/merge workflow (`tests/native_claude_workflow.rs`); missing-file Read/Edit is a per-call deny, not a worker abort | Native fallback registration acceptance, Claude headless quota capture and complete provider model/session coverage |
| Native macOS interaction | Setup-to-merge fixture, read-only three-session briefing, green/yellow/red indicators and running pulse | Live provider interaction, expired-approval presentation, OS reduced-motion and sustained resource checks |
| Cooperative agentmemory | Typed ledger/transport, real daemon round-trip, custom bootstrap, checkpoint polling, quiescent exit and completed pause/handoff crash recovery with explicit resume, and pending/accepted rework continuation after interruption | Automatic memory collection, worker authentication, company CLI acceptance, continuation across reassignment and automatic reviewed-target handoff |
| Knowledge assets | English source-linked wiki, development observations and local memory fallback | Live worker lesson/wiki ingestion, source revalidation and native knowledge interaction |
| Windows and releases | Committed development handoff and actual Windows CI failure evidence | Portable runtime, native Windows UI, verified packaged binaries, license closure and private macOS/Windows Release |
| Overall supervision | Deterministic Rust coordination and factual team briefing | Fine-grained idle/unresponsive policy and optional designated AI coordinator lifecycle |

### Claude CLI integration and native app acceptance (2026-10-10)

- [x] Verify: `claude` CLI subscribed login; live stdin/stream-json session, exact `--resume` and concurrent-session isolation (`tests/native_claude.rs`).
- [x] Task / Red/Green: guarded workers aborted when Claude read a not-yet-created file; missing-path Read/Edit is now a per-call deny after ancestor inspection.
- [x] Verify: live three-worker guarded Claude validate/run/diff/approve/merge workflow (`tests/native_claude_workflow.rs`).
- [x] Task / Red/Green: Claude can be the planning coordinator through the pinned guarded template restricted to `Read,Grep,Glob` with MCP/slash commands disabled; altered or unverified profiles stay refused.
- [x] Verify (native app, accessibility automation): workspace setup, Claude profile capability edit, Claude-coordinated plan, plan edits (reassignment), exact plan consent, Agent Room auto-open and per-request tool approval.
- [x] Plan: R-30 → BG-13 → SF-25 → UC-22 → FR-17/NFR-13 → QAS-23 → DDR-01/09 → DD-32 (live tool decisions); functional/non-functional analysis (`04b`) and design drivers (`05b`) added for the full trace.
- [x] Task / Red/Green: guarded workers hold a human-gated callback open (`AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS`, default 600 s); `tool-approve`/`tool-reject` decide through the ledger while the run is active; approval runs the exact input once in attempt 1, rejection denies only that call (`tests/live_tool_decision.rs`).
- [x] Task / Red/Green: native acceptance found the Agent Room did not list ledger-only live requests (Red: empty `tool_requests`); the snapshot now includes pending requests of running tasks for the current attempt.
- [x] Task / Red/Green: identical commands in later tool calls (or resumed attempts) collided with consumed/approved ledger entries and waited until timeout (Red reproduced with a read-only repeated command); live requests now carry the CLI tool-call identifier.
- [x] Task / Red/Green: Room live approvals were disabled and refused while the Room-resumed coordinator occupied the job slot; live approve/reject now decide synchronously with the Room executable (`tests/live_tool_decision.rs`, 5 tests).
- [x] Verify: native macOS app with three Claude Code workers — Claude-coordinated plan, plan edits, exact consent, live approvals from the Agent Room consumed in the same attempt, coordinator loss recovered (absence verified) and resumed, candidate `917074f` reviewed (3 tests exit 0), approved and merged; repository clean.
- [x] Plan: R-31 → BG-13 → SF-26 → UC-23 → FR-16/NFR-02 → QAS-24 → DDR-02 → DD-33.
- [x] Task / Red/Green: planning runs in a disposable tracked-files snapshot (`Repository::export_tree`, no symlinks/submodules/untracked/ignored files) outside the repository; the snapshot is removed and HEAD, status and the ignored-file listing must be unchanged. Unverifiable coordinators are accepted (`coordinator::planning_args`); verified templates keep their restrictions. A hostile mock planner that deletes and writes files leaves the target untouched (`tests/planner.rs`).
- [x] Verify: modal dialogs are exposed to accessibility — with the discard dialog open, `AXFocusedUIElement` is the "Keep editing" button inside the labelled dialog group within the web area; the earlier "page disappears" observation was a traversal-tool artifact over inert background content. Approved tool requests no longer offer "Review & allow" (6fc77b6). Duplicate values reported for dynamic task cards remain a tooling observation, not a confirmed defect.
- [x] Plan: R-32 → BG-13 → SF-27 → UC-24 → FR-18/NFR-10 → QAS-25 → DDR-07 → DD-34.
- [x] Task / Red/Green: Korean goals round-trip into the planning prompt and plan; polling and the Cmd+Enter shortcut never act during input-method composition (`tests/korean_input.rs`).
- [x] Verify: Korean (IME) text entry in the native macOS app with the system 390 Sebeolsik input method — virtual key codes typed 300 ms apart across 1-second polls composed exactly `안녕 하세요` in the goal field, and Save stored the identical UTF-8 (`ec9588eb859520ed9598ec84b8ec9a94`). Synthetic `keystroke` text bypasses the input method and is not a valid IME test.
- [x] Verify: complete native-app run through review, approval and merge with Claude workers (see live decision verification above).

### Workspace back navigation (DD-39)

- [x] Plan: R-33 → BG-09 → SF-28 → UC-25 → FR-19/NFR-10 → QAS-26 → DDR-07 → DD-39.
- [x] Task / Red/Green: titlebar "Back to run" and "Discard changes" with confirmation; view-only discard (`tests/workspace_navigation.rs`).
- [x] Task / Red/Green: persistent `last_run` link so saving the workspace never orphans the current run (`tests/startup.rs`); sticky, legible titlebar actions.
- [x] Verify: native app — Korean goal edit, Discard changes → confirmation (focus on Keep editing) → saved goal restored, `startup.json` byte-identical; Back to run covered by the startup flow test. Regression 208 passed.
- [x] Complete: wiki asset, log and agentmemory.

### Windows default desktop launch (R-23 / DD-38)

- [x] Plan: R-23/BG-10/SF-20/UC-17/NFR-10/NFR-11/QAS-18/DDR-03/DDR-08/DD-38; no-argument native startup must work without HOME.
- [x] Task / Red: selection regression failed with the former HOME-only error (1 passed, 1 failed).
- [x] Task / Green: pure nonempty HOME selection, Windows-only USERPROFILE fallback and no-argument CLI wiring.
- [x] Verify: macOS selection tests (2), CLI/startup/layout regression (8) and all-target Clippy; precedence and empty/missing values covered.
- [x] Verify: native Windows startup without HOME and with empty HOME; sequential window cleanup enforced (`windows_shell`, windows-latest CI 38056911056 and Release run 38069707257).
- [x] Complete: source-linked wiki and shared agentmemory checkpoint prepared; native Windows acceptance remains open.
- [x] Complete: implementation committed as 0308003 and pushed with 6ccd4bf.
- [x] Verify: native Windows results superseded by green windows-latest CI 38056911056 and Release run 38069707257.

### Native package prerequisites (R-24 / DD-27)

- [x] Plan: R-24/BG-10/SF-20/UC-17/QAS-18/DDR-03/DDR-08/DD-27; README support notes and manifest must disclose the native platform runtime.
- [x] Task / Red: both package target cases failed because their desktop runtime was absent from external_requirements.
- [x] Task / Green: Windows WebView2/macOS WebKit requirements in manifest and corrected public README support notes.
- [x] Verify: seven Python package/smoke contracts pass; README wording matches the native shell branches and initialization error. Native Windows execution remains separately gated.
- [x] Complete: wiki (`native-release-packaging`), agentmemory and commit/push.

### Missing upstream license texts (R-24 / DD-27)

- [x] Plan: R-24/BG-10/SF-20/UC-17/QAS-18/DDR-03/DDR-08/DD-27; native Windows CI 38040929721 failed on missing webview2-com license text after release compilation succeeded.
- [x] Task / Red: omitted WebView2 crate text regression failed before package output with the same CI error.
- [x] Task / Green: retained official MIT copies bound to exact package/version/SPDX, VCS commit and SHA-256; no packaging-time network or generic substitution.
- [x] Verify: seven Python packaging/smoke contracts and source-layout enforcement passed; actual offline closure resolves 42 macOS and 108 Windows dependencies.
- [x] Task / Red: CI 38041844312 Windows Rust tests passed, but packaging contract refused the checkout-converted license hash.
- [x] Task / Red/Green: actual Git autocrlf checkout preserves retained upstream license bytes (`.gitattributes` `-text`; contract fails without it, passes with it).
- [x] Verify: native Windows package execution after the byte-preserving checkout fix (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)).
- [x] Complete: source-linked upstream-license wiki, shared memory and 6ccd4bf committed/pushed; CI 38041844312 is in progress.

### Packaged native workflow acceptance (R-24 / DD-27)

- [x] Plan: existing R-24/BG-10/SF-20/UC-17/QAS-14/QAS-18/DDR-03/DDR-08/DD-27; exact packaged executable and native platform execution evidence.
- [x] Task / Red: copied-binary integrity/refusal tests failed on the absent packaged workflow runner.
- [x] Task / Green: three scoped CLI workers, shared/worktree parallel runs, diff/tests, unapproved merge denial and exact approved merge through the package binary.
- [x] Task / Red/Green: actual ZIP failed on pre-1980 registry timestamps; date clamp preserves contents/execute bits and overwrite refusal. Five Python contracts passed.
- [x] Verify: actual macOS ZIP entry CRCs and archived binary checksum; 2,978,551 bytes.
- [x] Verify: exact da746b9 macOS package completed both parallel fixture modes with diff/tests, unapproved merge refusal and approved candidate merge.
- [x] Verify: native Windows packaged execution and CI archives (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)).
- [x] Complete: source-linked wiki and agentmemory (`mem_mv26gi9j_1f9a63545a04`); unit commit/push follows local checks.
- [x] Verify: retained native CI archives; published as v0.1.0-preview.1 … preview.3 with verified `SHA256SUMS`.

### Native release package contract (R-24 / DD-27)

- [x] Plan: R-24 → BG-10 → SF-20 → UC-17 → NFR-03/NFR-11 → QAS-14/QAS-18 → DDR-03/DDR-08 → DD-27.
- [x] Task / Red: missing isolated assembly API; existing script selects no Windows .exe and carries stale mockup metadata.
- [x] Task / Green: native executable name, bounded public asset selection, target-locked reviewed license closure, notices, SBOM and exact binary checksum; no wiki mutation.
- [x] Verify: two Python contract tests cover both platform names, exact binary/evidence, overwrite refusal and pre-output license rejection; native CI 38040271132 passed both platforms on da746b9.
- [x] Verify: actual macOS release package from da746b9; 5,115,648 bytes, 42 dependency notices, exact checksum and packaged --help exit 0.
- [x] Verify: Windows package and exact packaged workflow/review/merge smoke on both platforms (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)); reviewed artifacts published.
- [x] Complete: source-linked wiki and agentmemory; unit commit/push follows successful contract checks.

### Source layout limit (DD-41)

- [x] Plan: R-35 → BG-05 → SF-12 → C-04/NFR-07 → DDR-04/DDR-06 → DD-41.
- [x] Task / Red: compiled CI layout check failed on eight oversized files and three inline test bodies.
- [x] Task / Green: responsibility modules and private tests under tests/unit/; maximum source/test file is 485 lines.
- [x] Verify: full macOS regression 213 passed, 0 failed, 9 ignored across 57 suites before checkout migration; focused layout and formatting passed again in the new checkout.
- [x] Verify: CI 38039813985 on 47eb2d7 completed successfully on native macOS and Windows (build, layout, all applicable tests, formatting, Clippy).
- [x] Complete: source-layout wiki, local evidence and agentmemory (`mem_mv25wmpa_f63ae0e2a551`); Windows completion remains open.

### Application icons (DD-40)

- [x] Plan: R-34 → BG-14 → SF-29 → UC-26 → FR-20/NFR-10/NFR-12 → QAS-27 → DDR-10 → DD-40.
- [x] Task / Red/Green: dependency-free PNG data-URL encoder with size bounds (`tests/icons.rs`).
- [x] Task / Red/Green: snapshot icons for built-in providers only; custom stays letters.
- [x] Task / Red/Green: macOS NSWorkspace icon resolution (live test passed); Windows executable icon extraction implemented, unverified.
- [x] Task / Red/Green: workspace and Agent Room render icons with accessible labels.
- [x] Verify: native macOS app shows Claude, ChatGPT (Codex) and OpenCode icons in workspace and Agent Room.
- [x] Complete: runtime-icons wiki asset, index/log and agentmemory; five macOS icon tests passed on 2026-10-10.
- [x] Verify: Windows per-user install-path discovery, Win32 icon extraction and PNG encoding (`tests/windows_icons.rs`, CI 38056051105).
- [x] Task / Red/Green (Windows CI 38056911056): Start Menu shortcut discovery through `SHGetFileInfoW` for installers that do not use the fixed per-user paths; Store packages fall back to letters.

### Windows native port (DD-35 … DD-38)

- [x] Plan: R-23/R-24 → BG-10 → SF-20 → UC-17 → NFR-03/NFR-11 → QAS-18 → DDR-03/DDR-08 → DD-35–DD-38.
- [x] Task / Red/Green: cross-platform kernel leases (DD-35); lease tests run on macOS and Windows runners (faae6f9).
- [x] Task / Red/Green: Windows Job Object process supervisor, ownership and quiescence (DD-36); `windows_containment` passed on windows-latest (CI 38036749822).
- [x] Task / Red/Green: Windows file identity and session randomness (DD-37, f79d3a5).
- [x] Task / Red/Green: Windows WebView2 desktop shell (DD-38); `windows_shell` smoke passed on windows-latest (CI 38036749822).
- [x] Verify: Windows CI green for build, tests, fmt and clippy; packaged Windows binary smoke test (windows-latest CI 38056911056 and Release run 38069707257).
- [x] Task / Red/Green: tag-triggered `release.yml` reuses platform validation and package smoke, then only the publish job (`contents: write`) creates a pre-release with both archives and combined `SHA256SUMS` (`tests/test_release_workflow.py`).
- [x] Verify: CI 38051732098 on b1e404e passed all four jobs (both platforms' validation and packaged workflow smoke).
- [x] Complete: `v0.1.0-preview.1` published by release run 38052572665 (macOS arm64 and Windows x86_64 archives + SHA256SUMS); downloaded macOS archive and inner binary checksums verified.
- [x] Task / Red/Green: the Windows archive checksum line was CRLF, so `shasum -c` could not find it; generated package text is now written as LF bytes (contract checks no CRLF).
- [x] Complete: `v0.1.0-preview.2` (release run 38054751920, source d84b876) published; downloaded archives verify with `shasum -c SHA256SUMS`, and each package's own `SHA256SUMS` verifies `agentconductor` / `agentconductor.exe`.

### Next implementation slice — cooperative worker integration

- [x] Plan: DD-29 defines addressed checkpoint directives and local execution authority.
- [x] Inspect: identify missing worker capability/bootstrap wiring and live custom-session address binding.
- [x] Task / Red: missing capability/bootstrap fails compiled-worker tests.
- [x] Task / Green: explicit custom capability, persisted owned session, addressed worker context and checkpoint commands; retain Rust approval authority.
- [x] Verify: three compiled-worker tests cover before/during-work polling in both workspace modes, monotonic reports, addressed rework, conflicting session rejection and mandatory merge approval.
- [x] Task / Red/Green: completed pause previously created a merge candidate; reconcile clean quiescent exits to stopped/blocked instead of completed.
- [x] Verify: live worker after completed acknowledgment refuses reassignment; partial-work stop, unchanged due scheduling, explicit attempt-2 resume and stale-address rejection tested.
- [x] Verify: pending/accepted directives at clean exit block completion; relinquished attempts reject further directives.
- [x] Verify: killed coordinator after completed pause/handoff acknowledgment; live child refuses recovery/reassignment, quiescence permits stopped recovery and exact-session attempt-2 resume.
- [x] Task / Red/Green: explicit resume preserves unexpired pending/accepted rework as provenance-bound context; supersession and receipt commit in one transaction; pause/handoff is superseded without replay.
- [x] Verify: killed coordinator after accepted rework; resumed attempt 2 receives the original instruction, inspects the partial file and continues; full regression 199 passed.
- [x] Task / Red/Green: automatic reviewed-target handoff transitions with no competing writer (DD-43). Red: completed handoff left the run `stopped`; Green: `tests/cooperative_handoff.rs` — the company worker stays the owner while alive, then the reviewed fallback continues `partial` work in a new session to `waiting_approval`; the no-fallback case stays stopped (`cooperative_worker`). Full regression 224 passed / 10 ignored / 0 failed. Shipped in v0.1.0-preview.4 (CI 38071626681, Release run 38072717471 on macOS and Windows; `SHA256SUMS` verified).
- [ ] Verify: company LLM CLI acceptance, continuation across reassignment and automatic handoff reassignment.
- [x] Complete: update traced design/wiki and retain verified bootstrap evidence; full cooperative integration remains open.

No whole-product completion or release readiness is claimed. Historical checkpoints below
retain their original evidence; later verified slices refine their remaining scope.

Each task: **plan** (linked R/BG/SF/UC/QAS/DD and acceptance criteria) → **task**
(Red → Green → Refactor) → **verify** (counterexamples, integration, review) → **complete**
(evidence, report, lessons). QAS defines quality goals; tests verify them.

| Task | Dependency | DD | Tests to fail first | Completion evidence |
|---|---|---|---|---|
| T-01 Profile/plan schema | Design documents | 01/03 | Wrong task count, cycles, overlapping scopes, argv injection | Schema/boundary tests |
| T-02 SQLite state/audit | T-01 | 02 | Illegal transitions, duplicate events, stale lease recovery | Restart tests |
| T-03 Subprocess supervisor | T-01 | 04/07 | stdin EOF, stderr flood, split UTF-8, stop grandchildren | Mock process contract |
| T-04 Three parsers/resume | T-03 | 03/04/07 | Fixtures to normalized events, errors, exact sessions | Version-pinned fixtures; separate live smoke |
| T-05 Workspace/scheduler | T-02/03 | 05 | Both workspace modes, three parallel executions, fixed base, dependency blocks, scope violations | Temporary Git integration |
| T-06 Diff/test/approval/merge | T-04/05 | 05/06 | Untracked files, failing tests, unapproved/changed candidate rejection | Protected-branch before/after SHA |
| T-07 Memory/wiki | T-02 | 08/09 | Outage outbox, duplicate delivery, stale sources | Offline fallback/source checks |
| T-08 CLI E2E/release | T-01–07 | 10 | Register → three tasks → verify → approve → merge; rejection/revision | Measured QAS, license/SBOM |

Verification: rtk cargo test; rtk cargo clippy --all-targets -- -D warnings;
rtk cargo fmt --check; lockfile-based license/text checks and SBOM generation through
the packaging script. `cargo deny` is an optional additional check and is not installed
on the current host.
Unit tests use mocks/captured fixtures, not paid model calls. Record live CLI smoke separately
using existing authentication. Completion requires passing tests, tested integration candidates,
and human approval matching exact SHAs.

Trace example: R-07 → BG-03 → SF-05 → UC-04 → QAS-02 → DD-06 → T-06 →
approval_changed_head_rejected. Add actual test names and evidence during implementation.


## Open acceptance that needs external resources (2026-10-11)

The checklist below distinguishes verified implementation from external acceptance.
Each external item names the resource it needs; newly active work is tracked separately.

| Need | Items |
|---|---|
| A real company/non-steerable CLI | Live checkpoint polling and handoff acceptance with a real company CLI, company fallback chains and consent interaction |
| Codex quota (reset 2026-10-16) or a second Codex account | Codex 0.162.1 live re-run, multi-provider briefing query |
| A Windows machine with a real subscribed CLI login | Windows account-capacity acceptance; fixture CI does not prove subscription access |
| Operator OS settings changes | Live reduced-motion preference |
| Product decisions requested from the user | Desktop planning without an LLM coordinator; redistribution beyond the configured fallback |

## Progress checklist

Checked items have current evidence. A foundation is not completion of the entire task.
Update this checklist after verification, not merely after writing code.

### T-01 — Profile and plan validation

- [x] Plan: requirements and acceptance criteria linked to DD-01/03.
- [x] Task: three-task schema, explicit workspace mode, DAG and scope validation.
- [x] Verify: invalid plans, path escapes, overlapping scopes, and custom profiles tested.
- [x] Complete: foundation committed; TDD evidence recorded.

### T-02 — Persistent state and audit

- [x] Plan: lifecycle, audit, and recovery requirements defined.
- [x] Task: SQLite transitions and event persistence implemented.
- [x] Verify: illegal transitions, duplicate events, and database reopen tested.
- [x] Task: workflow document, audit, and memory-outbox transactions integrated.
- [x] Verify: interrupted coordinator recovery and stale process ownership (`crash_recovery`: killed coordinator, live children, incomplete ownership evidence).
- [x] Complete: recovery scope verified and documented (manual §2.3, wiki `session-recovery`).

### T-03 — Subprocess supervisor

- [x] Plan: process contract and timeout policy defined.
- [x] Task: direct argv, cwd, stdin EOF, concurrent stdout/stderr spool, Unix process groups.
- [x] Verify: six cases cover large I/O, nonzero exit, spawn failure, timeout descendants,
  unread stdin, and forced-kill escalation.
- [x] Task/verify: explicit user stop and restart recovery (`stop_resume`, `crash_recovery`).
- [x] Complete: lifecycle controls verified; Windows Job Object containment (`windows_containment`) passes on windows-latest CI 38056911056 and Release run 38069707257.

### T-04 — CLI protocols and resume

- [x] Plan: native headless and resume commands researched.
- [x] Task: Claude/Codex/OpenCode/custom JSONL normalization and native argv builders.
- [x] Verify: synthetic contracts, malformed output, exact session selection, resume flag differences.
- [x] Task: environment/input-mode/permission profiles connected to workflow execution.
- [ ] Verify: installed-version fixtures and live CLI smoke. Claude 2.1.287 passed (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)); Codex passed on 0.162.0, and the 0.162.1 re-run is blocked by the account usage limit until 2026-10-16 (classified correctly, `availability`); OpenCode 2.0.25 passed live (`native_opencode`: task-file turn and exact session resume, free hosted model, 2026-10-11).
- [x] Complete: supported CLI/version contracts documented with evidence (user manual Part 3: verified versions, live evidence and what is not yet verified per CLI).

### T-05 — Workspace and scheduler

- [x] Plan: shared default, optional worktrees, disjoint ownership, dependency semantics.
- [x] Task: Git candidate snapshots with a temporary index and optional worktree creation.
- [x] Verify: three Git tests cover new files, unchanged HEAD/index, scope violations, exact base.
- [x] Task: three-agent parallel scheduler and dependency propagation.
- [x] Verify: shared/worktree E2E, overlapping mock attempt intervals, failed-dependency blocking.
- [x] Complete: shared and worktree modes verified (`workflow`, packaged smoke on both platforms) and documented (manual §1.4).

### T-06 — Evidence, approval, and merge

- [x] Plan: bind approval to candidate, base, diff, and test evidence.
- [x] Task: integration candidate, coordinator tests, review artifacts, approve/reject/merge commands.
- [x] Verify: failing tests, modified files/evidence, changed target branch (`target_branch_advanced_after_approval_blocks_merge`), missing approval (`workflow`, packaged smoke).
- [x] Complete: reviewed candidate merged in mock E2E test with exact before/after SHAs.

### T-07 — Memory and knowledge

- [x] Plan: SQLite authority, memory outbox, local fallback, wiki rules defined.
- [x] Task: English source/wiki/schema assets and development progress in agentmemory.
- [x] Task/verify: outbox delivery, offline continuity, local FTS recall, HTTP mock delivery.
- [x] Task/verify: runtime lessons and source revalidation (`knowledge`).
- [x] Verify: offline continuity, delivery retries/deduplication, changed sources (`memory`, `memory_transport`, `knowledge`).
- [x] Complete: runtime memory and knowledge workflow verified.

### T-08 — End-to-end verification and binary

- [x] Plan: MVP and release evidence defined.
- [x] Verify: initial native validate/normalize binary built and smoke-tested; 35 baseline tests passed.
- [x] Task: host dependency license declaration and source-text inventory (42 packages).
- [x] Verify: registered-profile planner tests and mock CLI run → parallel tasks → evidence → approval → merge E2E.
- [x] Verify: 42 dependency license texts, notices, CycloneDX inventory, and startup resource measurement; live agent resources remain unmeasured.
- [x] Task: macOS ARM64 binary, checksum, usage examples, and release manifest.
- [ ] Complete: full user-requested product acceptance audit and final binary. Prior mock CLI release is a verified checkpoint only.

## Distribution requirements

Distribute one native executable with rusqlite bundled SQLite, without required Python/Node
sidecars. Apply release optimizations. Track OS/architecture artifacts, checksums, dependency
and license manifests as release evidence.

## Commit and push cadence

Keep commits small and reviewable. Push accumulated commits at meaningful checkpoints
or when remote review is needed; pushing after every change is not required.
This updates the earlier request to commit and push each small unit.

## T-09 — Availability, multi-session roles, and models

- [x] Plan: R-18–20 → BG-07–09 → SF-09–11 → UC-09–11 → QAS-09–11 → DD-11–13.
- [x] Task: Red tests for restriction parsing, persisted scheduling, reassignment, and model approval invalidation.
- [x] Verify: three restriction parser tests pass (explicit offsets, relative delays, ambiguous messages, multiple windows).
- [x] Task: persisted due-work execution and bounded automatic retries.
- [x] Task: multiple sessions, per-profile concurrency, account groups, and audited reassignment.
- [x] Task: native catalog query/import and session model selection/change.
- [x] Verify: five library recovery tests, binary session-control E2E, two catalog tests; native Claude/Codex metadata smoke.
- [x] Complete: CLI controls documented, 59 tests and native metadata evidence retained, agentmemory updated, release binary produced.

## Current verification evidence

- `tests/availability.rs`: explicit/relative reset, multiple windows, offset parsing, ambiguous messages.
- `tests/recovery.rs`: restart scheduling, role transfer, model approval invalidation, concurrency cap, retry ceiling.
- `tests/models.rs`: metadata-only CLI protocol mocks; malformed catalogs rejected.
- `tests/cli_workflow.rs`: exact plan consent and merge approval; CLI schedule/reassign/model/resume commands.
- `tests/workflow.rs`: shared/worktree integration, retained failures and changed evidence.
- `tests/planner.rs`, `tests/git.rs`, `tests/memory.rs`: three-task planning, temporary-index snapshots, outbox and offline recall.

The full product backlog is larger than the deliverable CLI MVP. Unchecked scope must not
be described as verified or shipped. Native metadata smoke is not live paid execution.

## T-03 follow-up — Explicit stop and resume

- [x] Plan: preserve process containment (DD-04/07), ownership, artifacts, and human review gates.
- [x] Task: Red tests for process-tree stop, stop-before-launch, and workflow stop/resume.
- [x] Task: durable stop request, interrupted attempts, and explicit resume commands.
- [x] Verify: two supervisor stop tests and two workflow stop/resume tests, including interruption during integration verification.
- [x] Complete: documented CLI controls, binary-command verification, and passing regression suite.

A human stop must terminate the owned process group, retain raw I/O and partial files,
prevent automatic restart, and revoke any candidate approval. Resume must verify the target
Git baseline and permission configuration, append a new attempt, and repeat integration
verification. Unknown/crashed ownership is a separate recovery operation.

Full completion is governed by [the requirement-by-requirement audit](09-completion-audit.md).

## T-02 follow-up — Crash ownership and leases

- [x] Plan: recover only after kernel ownership is idle and all recorded processes/groups are terminal; preserve unknown ownership for intervention.
- [x] Task: Red tests for persistent exclusive leases and symlink rejection.
- [x] Task: kernel-backed coordinator lease and durable process ownership records.
- [x] Task: explicit crash recovery and cross-run repository ownership claims.
- [x] Task: cross-run native session leases and paused-workspace checkpoint validation (DD-14).
- [x] Verify: live-owner rejection, SIGKILL lease release, absent/ambiguous process ownership refusal, and resumed evidence preservation.
- [x] Verify: killed coordinator's live children also prevent a different run; dirty-repository rejection does not create an unknown ownership claim.
- [x] Verify: native session reuse refuses live/unknown previous ownership; workflow does not spawn while leased elsewhere; shared/worktree edits block resume.
- [x] Complete: compiled CLI recovery/control verification and documented limits. Native provider generation remains a separate full-product acceptance item.

A PID or leftover lock file alone is not proof of live ownership. Keep the lock inode stable;
use an OS advisory exclusive lease rather than deleting a possibly active owner's file.
Record owned process/group identity before starting I/O collection. Recovery cannot assume
that an absent process record means the subprocess did not start.

## T-05 follow-up — Review feedback and dependency rework

- [x] Plan: R-04–07/09 → BG-02/03 → SF-02–06 → UC-03/04 → QAS-02/06/10 → DD-15.
- [x] Task: Red tests for human feedback, transitive invalidation, preserved prior attempts and refreshed dependency contents.
- [x] Task: bounded review feedback, audited rework, updated dependency context and safe worktree refresh.
- [x] Verify: shared/worktree dependency chains, failure feedback, conflicts, stop/resume and exact fresh approval (82-test regression suite).
- [x] Complete: compiled CLI controls, evidence, documentation and agentmemory update.

Changed predecessor results must be visible before a dependent retries. Preserve old
worktrees and evidence rather than resetting them in place. A refresh conflict blocks
that task with retained evidence; it must not launch a worker against ambiguous contents.

Structured coordinator review is verified below with mock subprocesses. Native provider
review contracts and enforced pre-execution permissions remain separate acceptance items.

## T-06 follow-up — Structured coordinator review

- [x] Plan: R-04/06/07/09 → BG-02/03 → SF-02/04/05/06 → UC-03/04 → QAS-02/03/05/06 → DD-16.
- [x] Task: Red tests for typed final JSON, unknown task/evidence rejection, bounded context and coordinator read-only preservation.
- [x] Task: registered subprocess review, strict proposal validation, bounded revision rounds, audited scoped feedback and retained ownership.
- [x] Verify: failure → reviewer → revised tasks → new tests → exact human approval; malformed proposals, altered evidence, stale feedback, stopped reviews and write violations.
- [x] Complete: compiled `review`/`apply-review` controls, documentation, agentmemory and audit evidence for the mock CLI contract.

Verification: `rtk cargo test` passed 96 tests (one intentionally ignored subprocess helper),
27 suites in 37.42 seconds; Clippy and formatting checks passed. Graphify extracted
234 nodes/737 edges without an LLM; CocoIndex located saved-proposal application in
CLI and workflow modules. No new runtime dependency was added. Native execution,
pre-execution permission gates, runtime knowledge and functional UI remain open.

## T-03/06 follow-up — Pre-execution approval transport

- [x] Plan: DD-17, native callback failure semantics and retained process containment.
- [x] Task: Red tests for permission request/reply, stdin lifetime, malformed output and failed callback containment.
- [x] Task: bounded bidirectional JSONL transport integrated with the existing supervisor.
- [x] Verify: mock denial creates no side effects; exact human ledger approval permits one execution; protocol failure kills/reaps owned processes and preserves raw evidence.
- [x] Complete: transport/controller/ledger component evidence and documentation; this does not complete native permission enforcement.
- [x] Task/verify: SQLite exact-request/context approvals, maximum 900-second expiry, single consumption under concurrent callbacks, central audit/outbox.
- [x] Verify: installed Claude 2.1.286 accepts SDK PreToolUse registration in metadata-only initialization; no generation prompt sent.
- [x] Task/verify: native controller preserves a permission decision owed by active work after a result frame (Red → Green).
- [x] Task/verify: durable typed human decisions, provider capability integration and actual native denial/approval (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)).

Passing transport mocks does not prove complete R-08 enforcement. CLI profile labels,
ordinary shell hooks and post-execution integrity checks cannot substitute for native
pre-execution negative tests.

Current component regression: 107 tests passed, two opt-in/helper tests ignored,
30 suites in 37.64 seconds; native metadata test also passed separately
with `rtk cargo test --test native_control -- --ignored`. The native test is opt-in;
ordinary test discovery skips it and the existing subprocess helper. Clippy and formatting
checks pass. Graphify: 284 nodes/869 edges; CocoIndex locates the atomic consumption path.

Native metadata evidence is retained locally under
`.agentconductor/verification/claude-control-metadata-3042/process/`; stdout SHA-256
`935b6d736d8562ab968ac4a6fa440f34f46f0a00f3d3753e09e488a84857f5f1`.
Initialization succeeded, no user frames were sent, and ownership is terminal/quiescent.
`claude auth status` currently reports no login; actual generation/denial tests await
existing-subscription login. API keys are not a fallback.

## T-03/06 follow-up — Bounded worker policy and callback lifecycle

- [x] Plan: DD-18, file ownership and explicit approval boundaries.
- [x] Task: Red → Green for duplicate tool-use identities before approval consumption and requests after input closure.
- [x] Task: Red → Green for bounded tool inputs, owned edits, sensitive paths and filesystem alias rejection.
- [x] Verify: policy connected to bidirectional callback component, with pre-execution denial and retained process evidence.
- [x] Complete: component verification, local knowledge asset and agentmemory decision record.
- [x] Task/verify: production worker context binding, human CLI controls and live native denial/approval (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)).

Regression: 116 tests passed, two opt-in/helper tests ignored, 32 suites in 38.76
seconds. Clippy with warnings denied, formatting and whitespace checks passed.
Graphify extracted 308 nodes/941 edges without an LLM; CocoIndex found the classifier
and its evaluator bridge. No runtime dependency was added. Ponytail remains guidance;
no callable Ponytail tool or installed CLI was available in this environment.

Policy tests include replacement of the workspace directory at the same path,
directory/file symlinks, hardlinks, FIFO, ambiguous paths, unsupported input fields,
bounded input, secret filenames and task ownership. The controlled subprocess test
verifies no cross-task write, no pending-command side effect, one approval consumption
and denial after consumption. It uses a mock native protocol and fixed test context;
it does not verify production workspace fingerprints or native CLI enforcement.

## T-03/06 follow-up — Worker-bound native decisions

- [x] Plan: DD-19, bounded asynchronous verification and immutable request binding.
- [x] Task: Red → Green for asynchronous control cancellation and verifier containment.
- [x] Task: authoritative context verifier derives session/workspace/plan/Git binding from stored state; ignored sensitive targets use metadata without reading contents.
- [x] Task: opt-in Claude native callback worker integration with preassigned owned sessions.
- [x] Task: operator list/refresh/approve/reject controls and distinct waiting state.
- [x] Verify: compiled mock workflow controls in shared/worktree modes, stale/expired/denied decisions, native-session retention and separate merge approval.
- [x] Complete: worker/operator implementation checkpoint evidence, wiki and agentmemory; full-product completion remains open.
- [x] Verify: actual authenticated Claude tool denial/approval and containment: scope escape terminates the owned process group before writing; human rejection denies only that call (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)).

Verifier/transport checkpoint: 121 tests passed, two ignored, 34 suites in 43.06
seconds. Clippy, formatting and whitespace checks passed. Compiled verifier binding
tests cover both workspace modes, changed working files, unknown tasks and foreign
write scopes. Owned async tests cover stop, callback cancellation without a cooperative
stop listener, pending callback timeout and normal-exit final-frame draining.
Actual Claude metadata-only initialization passed separately in 0.90 seconds; no
generation prompt was sent. This does not prove native tool enforcement.
Graphify extracted 321 nodes/987 edges without an LLM; CocoIndex located the verifier.
No runtime dependency was added. Worker/operator mock-contract implementation is now
present; live provider enforcement remains open. The follow-up adds durable decision
epochs, exact-ID receipt lookup, exclusive approved retries, session identity checks,
stopped-state preservation and explicit rejection handling. Red → Green covers old
receipts beyond the listing limit and tampered payloads. Compiled CLI tests cover both
workspace modes, pending/consumed permissions, expiry/refresh, rejection/revision,
explicit stopped resume, changed paused workspaces and separate final merge approval.

Worker/operator checkpoint: 127 tests passed, two ignored, 34 suites in 49.61 seconds;
Clippy, formatting and whitespace checks passed. Actual Claude metadata-only control
initialization passed separately in 0.90 seconds. Graphify extracted 341 nodes/1,083
edges without an LLM; CocoIndex located exact-ID receipt reads. No runtime dependency
was added. Native tools, provider containment, runtime knowledge and functional UI
remain open in the full-product audit.

## T-03/05/07 follow-up — Live intermediate reports

- [x] Plan: DD-20, owned spool observation, independent attempt evidence and bounded local summaries.
- [x] Task: Red → Green tests observe normalized evidence before worker completion, reject malformed streams and verify final draining.
- [x] Task: bounded observer and SQLite attempt event ledger connected to both worker transports.
- [x] Verify: regression, cancellation, final records, retained raw evidence and bounded queries.
- [x] Complete: evidence, wiki and agentmemory checkpoint; native runtime proof remains separate.

Live-progress checkpoint: 131 tests passed, two ignored, 35 suites in 50.14 seconds.
Clippy, formatting and whitespace checks pass. Graphify extracted 352 nodes/1,110
edges without an LLM; CocoIndex confirmed both worker transports use the observer.
No runtime dependency was added. A deferred SQLite read-to-write upgrade conflict
was reproduced in native-worker mock rework and fixed with an immediate transaction.

## T-07 follow-up — Runtime reviewed lessons

- [x] Plan: DD-21, bounded typed promotion, repository source hashes and task-scoped handoff.
- [x] Task: Red → Green for exact promotion consent, stale source exclusion and scope/alias refusal.
- [x] Task: authoritative local assets, derived wiki/index/log, operator controls and worker context.
- [x] Verify: compiled digest/accept controls, offline next-attempt task-scoped context, changed sources, mirror alias refusal and recovery.
- [x] Complete: checkpoint evidence and wiki/agentmemory; full-product acceptance remains open.

Runtime lesson checkpoint: 134 passed, two ignored, 36 suites in 51.90 seconds.
Clippy, formatting and whitespace checks pass. Graphify: 377 nodes/1,199 edges, no LLM;
CocoIndex located worker context injection. No runtime dependency added. Staleness
uses source hashes; bounded metadata checks do not freeze filesystem paths. Native
lesson service/confidence-aware retrieval, cross-run context and UI remain open.

## T-04/09 follow-up — Native Codex subscription/session contracts

- [x] Plan: existing ChatGPT authentication, explicit session IDs, no API-key fallback and retained process evidence.
- [x] Verify: installed Codex 0.162.0 stdin/JSONL initial turn and exact session resume preserving previous context.
- [x] Verify: two actual concurrent sessions retain distinct IDs and recall their own context on resume.
- [x] Task: Red → Green for nonterminal Codex error items classified as visible warnings; terminal outcomes remain separate.
- [x] Complete: regression evidence, wiki and agentmemory checkpoint; full-product goal remains active.
- [ ] Verify: native scoped implementation, dangerous-tool containment/approval and all-provider model/session behavior.

Native Codex checkpoint: 135 passed, four ignored, 37 suites in 52.91 seconds.
The two generation tests were also run explicitly: exact context resume passed in
7.74 seconds; concurrent independent contexts passed in 7.13 seconds. Installed
version: 0.162.0. Clippy/format/whitespace checks pass. Graphify: 377 nodes/1,199
edges, no LLM; CocoIndex located the nonterminal item normalization. No runtime
dependency added. Actual risky tool/native implementation tests and functional UI
remain open; these text-turn proofs do not close R-08 or the full product.

## T-08/09 follow-up — Functional intervention room

- [x] Plan: DD-22, approved visual direction, embedded native surface and exact-state control broker.
- [x] Task: Red → Green for newer-stop preservation, stale mutation refusal, local capability enforcement, cookie collisions and artifact/env exposure.
- [x] Task: embedded Agent Room with real task/evidence/approval/session controls.
- [x] Verify: backend integration and real rendered fixture feedback/reassignment/model/resume/diff/tests/approval/merge.
- [x] Verify: native Cocoa/WebKit review rendering and Cmd-W close with process exit 0 and endpoint shutdown, using a local metadata wrapper.
- [x] Task: Red → Green for same-run native-window lease, child CLI duplicate refusal and reopen after release.
- [ ] Verify: unbundled identity and remaining availability/tool-gate interactions.
- [x] Complete: actual browser UI screenshot, public usage, wiki and agentmemory checkpoint.
- [x] Verify: UI runtime resources and final distribution/license evidence. Released v0.1.0-preview.5 macOS binary (5,313,920 bytes) running `ui-new`: main process RSS 103.2–103.3 MiB and 0.7–1.0% CPU over six idle samples (2026-10-11); WebKit WebContent/GPU helpers are shared system XPC services and are not attributed. Every release archive carries license notices, CycloneDX SBOM, binary checksum and `SHA256SUMS` (release package contracts).
- [ ] Verify: goal/profile setup and all native provider/model/permission acceptance.

Intervention room checkpoint: 142 tests passed, four ignored, 39 suites in 55.78
seconds. Clippy passes. Graphify extracted 482 nodes/1,508 edges without an LLM;
CocoIndex located the local and native control implementations. No Cargo dependency
was added. The first whole-suite attempt exposed a retained PID-based temporary-run
collision; UUID fixture paths preserve failed evidence and prevent reuse. Real browser
verification uses a named local mock fixture, not authenticated provider execution.
Native Cocoa/WebKit review rendering and Cmd-W close were verified using a local app
metadata wrapper around the same binary. The first close shortcut failed; standard
application/edit/window menus fixed it, and the updated process exited 0 with its endpoint
stopped. Bare-executable computer-use identity, quota/tool-gate UI and resource acceptance
remain open. User correction requires closing prior owned verification windows/processes
and checking their exit before replacement; stored lesson lsn_5169ce1ac0c2a86d.

### Direct native transport (DD-23)

- [x] Plan: replace loopback HTTP with an embedded native message bridge.
- [x] Task / Red: missing Room compilation failure; additional fields exposed unit-variant deserialization weakness.
- [x] Task / Green: Rust Room controller, bounded typed requests, WebKit reply handler and embedded document.
- [x] Verify: full suite, targeted final-source tests, Clippy/rustfmt; actual native feedback/resume/review/approve/merge, zero TCP listeners and exit-0 cleanup.
- [x] Complete: README and actual native screenshot refreshed; direct-native wiki evidence and agentmemory records saved; checkpoint prepared.

### Unix termination evidence (DD-24)

- [x] Plan: retain numeric exit status and termination signal as separate outcomes.
- [x] Task / Red: missing signal fields failed compilation; coordinator incorrectly reported synthetic exit -1.
- [x] Task / Green: separate optional signal in process/ownership/test/coordinator evidence; completed coordinator flag prevents signal failure appearing running.
- [x] Verify: 33 targeted tests; complete suite 144 passed / 4 ignored / 0 failed; Clippy/rustfmt. Fresh older-fixture run succeeded, so historical failure remains unexplained.
- [x] Complete: source-hashed process-outcomes wiki, completion audit and agentmemory updated; checkpoint prepared.

### Native startup workspace (DD-25)

- [x] Plan: trace R-21 through business goal, feature, use case, QAS and DD-25.
- [x] Task / Red: registry persistence/redaction/stale revisions; modified profile consent bypass, late repository baseline change and oversized settings temporary-file leak reproduced.
- [x] Task / Green: local startup controller, owned catalog/planning jobs and exact consent; repository baseline rechecked under execution lease; settings bounds checked before temporary-file creation.
- [x] Task / UI: profile cards, goal setup, editable plan review and same-window run opening; macOS JSON smart quote substitution disabled and verified.
- [x] Verify: full regression 148 passed / 4 ignored / 0 failed; Clippy/rustfmt. Actual native empty startup → custom profile registration → planning → task edits → exact plan consent → same-window run → diff/tests → candidate approval → merge → home; exit 0 and kernel absence confirmed. Final text-input correction separately verified in the rebuilt native app.
- [x] Complete: public documentation and actual screenshot, source-hashed wiki and agentmemory checkpoint (`mem_mv1ukwwx_de6b3718e639`). Graphify: 548 nodes / 1,752 edges; CocoIndex located reviewed execution. Provider-authenticated planning, dialog accessibility and OS containment remain part of the full-product audit.

### Remaining subscription quotas (DD-26)

- [x] Plan: R-22 / BG-10 / SF-19 / UC-16 / QAS-17 traced; official Codex stdio and Claude status-line metadata reviewed.
- [x] Task / Red: independent quota windows, 27% used -> 73% remaining, unknown/malformed/future/stale evidence and no context/cost inference.
- [x] Task / Green: normalized local observations, bounded Codex existing-auth metadata query and native account summary refresh.
- [x] Verify: mocked metadata lifecycle and actual Codex quota query; actual native account selection/refresh and source/reset display.
- [ ] Verify: Claude headless capture, malformed-query/timeout cleanup and measured resource bounds.
- [x] Complete: public usage docs, wiki/memory evidence and quota checkpoint; Claude/resource acceptance remains open.

### Windows continuity and GitHub Releases (DD-27)

- [x] Plan: R-23/R-24 / BG-10 / SF-20 / UC-17 / QAS-18 traced; current Unix-only runtime gaps identified.
- [x] Task / Red: actual Windows CI build failure captured; latest observed run 38027108692.
- [x] Task / Red: platform-specific cancellation, lease, recovery and path-boundary tests (`windows_containment`, `leases`, `windows_shell`).
- [x] Task / Green: portable entropy, native Windows process containment/leases/filesystem checks and desktop shell (DD-35–DD-38; windows-latest CI 38056911056 and Release run 38069707257).
- [x] Verify: macOS + Windows runners, exact packaged-binary smoke tests, licenses, checksums and support notes (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)).
- [x] Complete: committed Windows development handoff.
- [x] Complete: private GitHub Release with verified macOS and Windows assets (v0.1.0-preview.3).

### Subscription-independent supervision and per-agent fallback (DD-28)

- [x] Plan: R-25/R-26 through BG-11 / SF-21 / UC-18 / QAS-19; non-steerable company CLI and optional AI coordinator boundaries recorded.
- [x] Task / Red: restricted paid worker transfers to configured company CLI; no policy means waiting; cycle/account/model/permission rejection; ordinary errors and stops never fail over.
- [x] Task / Green: per-profile fallback, durable account restrictions, drained attempt handoff and fresh target session.
- [x] Task / UI: fallback profile/model registration, company preset action and literal policy shown in execution consent.
- [ ] Verify: actual native fallback profile/model registration and consent interaction.
- [x] Verify: local fallback contracts cover retained partial work, unavailable targets, ordinary-error/stop rejection and lost process ownership.
- [ ] Verify: native shared/worktree fallback chains, restart/reset, exact approval and consent interaction.
- [x] Complete: source-hashed wiki, memory and verified component checkpoint; full fallback acceptance remains open.
- [ ] Verify: fine-grained idle detection and optional designated AI coordinator session lifecycle.

Windows CI evidence: historical run 38022076235 on d9590d7 and latest observed run 38027108692 on c4f6291 completed with macOS success and Windows build failure. The identified Windows gap includes E0425: process::run_controlled_async is Unix-only. Native Windows execution/release remains unverified.

### Cooperative agentmemory steering (DD-29)

- [x] Plan: R-27 / BG-11/BG-12 / SF-24 / UC-21 / QAS-22; company agents publish intermediate status and pull addressed instructions before work/at checkpoints alongside native steering.
- [x] Task / Red: addressed directive validation, duplicate/expiry handling and acknowledgment transitions; tests exposed expiry rollback and missing CLI commands.
- [x] Task / Red: completed pause incorrectly produced a merge candidate; live process after handoff acknowledgment must retain ownership.
- [x] Task / Green: owned process/group absence plus exact directive reconciliation; pause/handoff exits stop partial work, unresolved directives block completion and explicit resume uses a new attempt.
- [x] Verify: five worker tests, including live acknowledgment versus process presence, same-attempt relinquishment, unchanged due scheduling and explicit resume/stale-address refusal.
- [x] Verify: sixth worker test covers actual killed-coordinator pause/handoff recovery; targeted worker/crash/stop suite passes 11 tests.
- [x] Task / Green: transactional typed local ledger, checkpoint CLI polling/acknowledgment and bounded intermediate reports consumed by team briefing.
- [x] Task / Red/Green: bounded loopback REST publish/exact-ID read/import, shared outbox transport, receipt validation, EOF handling and capped pending batches.
- [x] Task / Green: explicit custom profile capability, owned session before launch and worker prompt/environment bootstrap.
- [x] Verify: compiled non-steerable workers voluntarily poll before/during work in shared/worktree modes; conflicting returned sessions fail and merge still requires approval.
- [x] Task / Green: memory discovery/import (DD-44: `memory-discover`, scoped to each task's current address via the agentmemory 0.9.29 `GET /agentmemory/memories?agentId=…` contract; fixture test plus real local daemon on 127.0.0.1:3113) and handoff-driven scheduler transitions (DD-43). Full regression 225 passed / 10 ignored / 0 failed. Shipped in v0.1.0-preview.5 (CI 38074563237 after a Windows cold-start timeout fix in the containment test; Release run 38075480798; `SHA256SUMS` verified).
- [x] Verify: eleven regular cooperative tests and five transport tests; local ledger/compiled CLI, restart/idempotency, contention, expiry, plan supersession, progress/briefing and namespace separation.
- [x] Verify: real configured local agentmemory daemon; typed directive/ack/progress round-trip, duplicate import, briefing source, confirmed outbox delivery and unchanged workflow authority on a controlled fixture.
- [ ] Verify: live company CLI checkpoint polling, individual worker authentication and human permission boundaries.
- [x] Complete: local checkpoint protocol documentation, source-hashed wiki and architecture memory evidence. Full cooperative steering acceptance remains open.

### On-demand team briefing (DD-30)

- [x] Plan: R-28 / BG-12 / SF-22 / UC-19 / QAS-20; factual bundle separate from optional LLM prose.
- [x] Task / Red: missing briefing/read-only API failed compilation; all-session coverage, stale/conflicting reports, missing-run refusal and state invariance tested.
- [x] Task / Green: read-only SQLite snapshot, event observation timestamps, separately observed process presence, bounded source reports and shared CLI/native request.
- [x] Verify: actual native three-session briefing, restriction/error/stop evidence, CLI/native parity, foreign/future report handling and concurrent snapshot consistency.
- [x] Verify: exact-ID agentmemory progress import appears in briefing with its real source on the configured-daemon fixture.
- [ ] Verify: active multi-provider session query, tool approval expiration and automatic live worker report collection.
- [x] Complete: public usage, source-hashed wiki and checkpoint memory evidence; automatic cooperative worker integration remains open.

### Agent status lights (DD-31)

- [x] Plan: R-29 / BG-12 / SF-23 / UC-20 / QAS-21 mapped.
- [x] Task: green/yellow/red indicator, running pulse, textual labels and stable list rendering.
- [x] Verify: actual simultaneous waiting/failed/running states and two-frame native pulse; static mapping/reduced-motion checks; UI stop and owned-process cleanup.
- [ ] Verify: live reduced-motion OS preference and pulse continuity during quota observation changes.
- [x] Complete: public usage and source-hashed checkpoint evidence; remaining acceptance stays open.

Quota/fallback/status checkpoint: final full regression 162 passed / 4 ignored / 0 failed across 43 suites in 96.16 seconds; Clippy, formatting, whitespace and JavaScript syntax checks passed. Local evidence: `.agentconductor/verification/quota-fallback/`. Actual Codex quota refresh used existing CLI authentication and no generation; private account observations remain local. Native status fixture reached stopped through UI and all five owned PIDs were kernel-absent after exit-0 cleanup. Graphify: 1,129 nodes / 2,601 edges without an LLM; CocoIndex structural grep located apply_fallbacks. No runtime dependency was added. Ponytail capabilities are not exposed in this session; no execution claimed.

Team briefing checkpoint: full regression 169 passed / 4 ignored / 0 failed, 44 suites, 99.32 seconds; Clippy/format/JS syntax passed. Native Brief team returned three real retained fixture sessions; revision and SQLite SHA-256 unchanged; owned app PID 49618 exited 0 and kernel absence confirmed. Graphify 1,160 nodes / 2,664 edges without LLM; CocoIndex located read_only_snapshot. No Cargo dependency added. CI run 38023847399 completed: macOS all checks passed, Windows build failed. Live agentmemory collection/directive acknowledgment and native Windows remain unfinished.

Cooperative checkpoint verification: final full regression 177 passed / 4 ignored / 0 failed, 46 suites, 107.05 seconds. Eight new local contract tests; Clippy all-targets with warnings denied, formatting and diff checks passed. Graphify rebuilt 1,216 nodes / 2,847 edges without LLM; CocoIndex located report_progress. No Cargo dependency added and no verification UI was opened. Agentmemory architecture decision: mem_mv1ws2a1_1df7797727b8. Live memory transport/capability/bootstrap/scheduler and native Windows remain open.

Memory transport checkpoint: 188 passed / 5 ignored / 0 failed, 47 suites, 105.23 seconds. Separately invoked existing-daemon fixture passed; actual company CLI polling remains open. Clippy/format/diff passed; Graphify 1,280 nodes / 3,021 edges, CocoIndex located remember_note. No dependency added, no daemon/UI launched. Architecture memory mem_mv1xilna_6df13588935e. CI 38025340939 terminal: macOS success, Windows build failure. Full goal remains active.

Cooperative worker bootstrap checkpoint: 191 passed / 5 ignored / 0 failed across 48 suites (66.29 seconds summed suite time, not wall time). Clippy all-targets with warnings denied, format and diff checks passed. Three new tests exercise real CLI subprocesses and authoritative local checkpoint commands; no company LLM or new daemon/UI was launched. Graphify: 1,304 nodes / 3,070 edges without LLM; CocoIndex located worker_address. No Cargo dependency added. Automatic memory collection/authentication/scheduler handoff and native Windows remain open.

Cooperative exit checkpoint: 193 passed / 5 ignored / 0 failed across 48 suites, 67.02 seconds summed suite time (not wall time). Clippy all-targets with warnings denied, formatting and whitespace passed. Graphify: 1,319 nodes / 3,099 edges without LLM; CocoIndex located exit_directive. Lesson lsn_d9c15a15246342f9 distinguishes directive completion, task completion and process exit. No dependency/daemon/UI added. Local evidence: `.agentconductor/verification/cooperative-exit/`. CI 38027108692 on the preceding bootstrap commit completed: macOS success, Windows build failure. Automatic reviewed-target handoff and full-product acceptance remain open.

Cooperative recovery checkpoint: targeted cooperative_worker/crash_recovery/stop_resume suites passed 11 tests, no failures. A real killed coordinator and live acknowledged worker validate the existing recovery implementation without a runtime change. Clippy with warnings denied, formatting and whitespace passed; the test harness explicitly reaps its coordinator on both branches. Graphify 1,325 nodes / 3,108 edges without LLM; CocoIndex located recover. No UI/daemon/dependency added. Evidence: `.agentconductor/verification/cooperative-recovery/`. The preceding full regression remains 193/5/0; no new full-regression total is claimed. Pending/accepted instruction continuation, automatic handoff and Windows/releases remain open.

### Product README and user manual (DD-42)

- [x] Plan: R-36 / BG-15 / SF-30 / UC-27 / FR-21 / NFR-14 / QAS-28 / DDR-11 mapped; first-contact questions (orchestration engine vs optional LLM coordinator, role assignment, usage-limit redistribution, prerequisites, agentmemory optional) added to R-36/QAS-28.
- [x] Task / Red/Green: `tests/public_docs.rs` (README under 120 lines, no shell blocks, screenshots, required sections; manual documents every `--help` command; manual links only shipped assets).
- [x] Verify: `public_docs` 3/3 passed; release package contracts 13/13 passed.
- [x] Complete: v0.1.0-preview.3 (Release run 38069707257: validate + native-package on macOS and Windows, publish) ships README, `docs/user-manual.md` and screenshots; both archives verified with `shasum -a 256 -c SHA256SUMS`. Full regression 221 passed / 9 ignored / 0 failed; Clippy denied warnings clean; no source file over 500 lines.

### Manual plan authoring (R-37 / DD-45)

- [x] Plan: R-37 → BG-16 → SF-31 → UC-28 → FR-22/NFR-15 → QAS-29 → DDR-12 → DD-45 (user decision 2026-10-11).
- [x] Task / Red: `tests/manual_plan.rs` failed with `unknown variant draft`.
- [x] Task / Green: startup `draft` request writes an empty three-task proposal without a job; **Write plan manually** button; README and manual updated.
- [x] Verify: draft launches no process; unfilled/overlapping drafts refused; filled draft approved by exact digest runs to `waiting_approval` with three tests; dirty tree and moved base refused. Full regression 227 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Workspace settings and switching (R-38/R-39, DD-46/DD-47)

- [x] Plan: R-38/R-39 → BG-17 → SF-32/SF-33 → UC-29/UC-30 → FR-23/FR-24/NFR-16 → QAS-30/QAS-31 → DDR-13 → DD-46/DD-47 (user request 2026-10-11).
- [x] Task / Red: `tests/workspace_settings.rs` failed (`unknown variant pick_folder`, missing settings layout).
- [x] Task / Green: goal first, **Workspace settings** group (coordinator, repository + **Browse…**, worktrees), OS folder dialog without new dependencies, title-bar workspace menu, recent registry, in-place switch with lease handover; README/manual updated (also corrected **+ Add**).
- [x] Verify: picker selection/cancel, save without repository, plan refused without repository, switch/recent/lease release, refusal while held elsewhere and for relative paths. Full regression 229 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check (macOS dialog, Windows CI) and release.

### Named workspace registry (R-40 / DD-48)

- [x] Plan: R-40 → BG-17 → SF-34 → UC-31 → FR-25/NFR-16 → QAS-32 → DDR-13 → DD-48 (user request 2026-10-11).
- [x] Task / Red: `tests/workspace_registry.rs` failed (`unknown field name`, no name field).
- [x] Task / Green: `name` in workspace config with validation, name field above the goal, `create_workspace` (slugged unique folder under `~/.agentconductor/workspaces/`), `forget_workspace` (registry only), menu by name with Remove / New workspace… / Open existing folder…; README and manual updated.
- [x] Verify: name save/limits, create twice gives distinct folders, listing by name, current cannot be removed, removal keeps files. Full regression 231 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Release change summaries (R-41 / DD-49)

- [x] Plan: R-41 → BG-10 → SF-35 → UC-32 → FR-26 → QAS-33 → DDR-08 → DD-49 (user request 2026-10-11).
- [x] Task / Red: `tests/test_release_notes.py` failed (no script, generic `--notes`).
- [x] Task / Green: `CHANGELOG.md` with sections for preview.1–6, `scripts/release_notes.py`, publish job checks out the tag and uses `--notes-file`.
- [x] Verify: exact section extraction, missing/empty refusal, every published tag covered, workflow contract; release contracts 15/15.
- [x] Complete: preview.6 published with its CHANGELOG notes (CI 38100249588 and verify jobs of Release run 38101164095 green; the publish job was not started by GitHub because of an account billing/spending limit, so the verified CI artifacts were checksum-checked and published locally with the same notes); earlier releases' notes replaced with their sections.

Release policy (user, 2026-10-11): releases are cut only when the user asks; each feature updates the next `CHANGELOG.md` section instead.

- [x] Amendment (user, 2026-10-11): **Open existing workspace…** lists saved workspaces in `~/.agentconductor` (`saved_workspaces` / `open_workspace`; folders without `startup.json` or outside the data directory refused); Red → Green in `tests/workspace_registry.rs`.

### Explained clean-baseline gate (R-42 / DD-50)

- [x] Plan: R-42 → BG-13 → SF-36 → UC-33 → FR-27 → QAS-34 → DDR-01 → DD-50 (user request 2026-10-11; clean baseline rule kept).
- [x] Task / Red: `tests/baseline_commit.rs` failed (`unknown variant baseline_status`, no dialog).
- [x] Task / Green: `baseline_status` and revision-checked `commit_baseline` requests; dialog with reason, changed files, **Commit and continue** / **Stop** before Generate plan and Write plan manually; manual and CHANGELOG updated.
- [x] Verify: modified and untracked files listed; no commit on stop or stale revision; one commit with all changes and goal-derived message; clean repository reports nothing to commit; planning then starts from the new HEAD. Full regression 233 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Ignore OS files and continue (R-44 / DD-52)

- [x] Plan: R-44 → BG-13 → SF-36 → UC-36 → FR-29 → QAS-37 → DDR-01 → DD-52 (user request 2026-10-11).
- [x] Task / Red: `os_metadata_files_are_ignored_locally_without_a_commit` failed (no `os_files`, no dialog guidance).
- [x] Task / Green: `os_files` in baseline status, revision-checked `ignore_os_files` writing missing patterns to the local exclude file, dialog guidance and **Ignore and continue** loop; manual and CHANGELOG updated.
- [x] Verify: nested `.DS_Store` and `._*` detected; patterns added once; no commit; non-OS file remains for decision; stale revision refused. Full regression 234 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Visible planning progress (R-45 / DD-53)

- [x] Plan: R-45 → BG-12 → SF-38 → UC-37 → FR-30 → QAS-38 → DDR-07 → DD-53 (user request 2026-10-11, after a plan looked missing while the coordinator was still working for ~45 s).
- [x] Task / Red: `tests/planning_progress.rs` failed (no `started_ms`/`log`, no progress element).
- [x] Task / Green: `started_ms` in the stored job, `started_ms` and `log` in live and recovered job snapshots, review-area progress with elapsed time, ready notice with scroll, failure reason with log folder; manual and CHANGELOG updated. Full regression 237 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check.

### Human-readable task instructions (R-46 / DD-54)

- [x] Plan: R-46 → BG-13 → SF-39 → UC-38 → FR-31 → QAS-39 → DDR-07 → DD-54 (user request 2026-10-11).
- [x] Task / Red: `tests/plan_readability.rs` failed (no section format in the planning prompt; editor 3 rows).
- [x] Task / Green: planning prompt requires labelled sections with bullets in the goal's language; instruction editor 6 rows minimum, auto-grows to 30; manual and CHANGELOG updated. Full regression 239 passed / 11 ignored / 0 failed.
- [ ] Complete: native check with a real coordinator plan.

### Agent role rules and numbered steps (R-47 / DD-55)

- [x] Plan: R-47 → BG-19 → SF-40 → UC-39 → FR-32 → QAS-40 → DDR-12 → DD-55 (user request 2026-10-11; numbered steps amendment).
- [x] Task / Red: `tests/role_rules.rs` failed (no `[roles.json]` argument, no `role` field, no table or steps).
- [x] Task / Green: `roles` module (validate, prompt section, plan check), optional `role` on tasks (omitted when absent), `plan … [roles.json]`, workspace `roles` saved and passed on Generate plan, role-aware manual drafts, step 1–3 headings, role table with Suggest defaults, role badges; README, manual and CHANGELOG updated. Full regression 242 passed / 11 ignored / 0 failed.
- [x] Amendment: role entries `{agent, model?}` (plain strings still accepted), model validated against the observed catalog, engine applies pinned models to tasks, model selectors in the role table; Red → Green `role_entries_can_pin_an_observed_model`.
- [x] Amendment: each step is its own card (Step 1 Workspace settings, Step 2 Agent roles with Save/Write/Generate, Step 3 Review the plan).
- [ ] Complete: native check with a real coordinator.

### Immediate agent registry persistence (R-48 / DD-56)

- [x] Plan: R-48 → BG-09 → SF-41 → UC-40 → FR-33 → QAS-41 → DDR-13 → DD-56 (user report 2026-10-11: added agents lost when the window was reopened).
- [x] Task / Red → Green: `tests/profile_persistence.rs`; `save_profiles` request (new `setup/profiles.rs`), coordinator/role repair, UI Use profile and Remove save at once.
- [x] Verify: agent list survives reopen, goal untouched, stale/invalid refused, coordinator and roles repaired.

### Model lists, plan retention and compact agents (R-49/R-50/R-51, DD-57/DD-58)

- [x] Plan: R-49 (amended: once on add, Models refreshes, quiet), R-50, R-51 → SF-41/SF-42 → UC-40/UC-41 → FR-34–FR-36 → QAS-42/QAS-43 → DDR-13 → DD-57/DD-58 (user reports 2026-10-11: no models in selectors; focus jumped; plan vanished after Models).
- [x] Task / Red → Green: `tests/model_catalogs.rs`, `tests/profile_persistence.rs` — catalog carry-over on save, catalog changes keep the proposal, consent compares execution views, save keeps the plan unless plan-relevant settings change, discovery only for newly added agents and silent, in-place model option refresh skipping the focused selector, compact cards.

### Shareable workspace settings (R-52 / DD-59)

- [x] Plan: R-52 → BG-20 → SF-43 → UC-42 → FR-37/NFR-18 → QAS-44 → DDR-15 → DD-59 (user request 2026-10-11).
- [x] Task / Red: `tests/workspace_share.rs` failed (`unknown variant export_settings`, no menu items).
- [x] Task / Green: `setup/share.rs` (export, preview, import), validation helpers moved to `setup/profiles.rs` (setup.rs under 500 lines), Workspace menu Export/Import with preview dialog and verification checkbox; manual, README, CHANGELOG.
- [x] Verify: no secrets/catalogs/goals/absolute repo path, `repository: "."`, LF and deterministic output; import reproduces settings, keeps local secrets/catalogs and goal, downgrades local claims unless confirmed; unknown format and stale revision refused.
- [ ] Complete: two-machine check with a real repository.

### Variable plan size and plan documents (R-53/R-54, DD-60)

- [x] Plan: R-53 (supersedes R-04's fixed three), R-54 → SF-44 → UC-43 → FR-38/FR-39 → QAS-45 → DD-60 (user decision 2026-10-11).
- [x] Task / Red: `tests/plan_size.rs` failed (three-task validation, fixed prompt, no `--from`, no `read_plan`, no add/remove).
- [x] Task / Green: 1–12 task validation, smallest-split prompt, `plan … [--from plan.md]` with `source-plan.md`, `setup/planning.rs` shared by Generate plan and Read plan (goal from heading), `pick_file` (macOS/Windows), Read plan button, Add/Remove task; old `requires_exactly_three_tasks` updated to the new rule; README, manual, CHANGELOG.
- [ ] Complete: native check with a real plan document and coordinator.

### Slugs, phases, progress, archives and plan rules (R-55–R-59, DD-61–DD-65)

- [x] Plan: R-55–R-59 → BG-21 → SF-45–SF-49 → UC-44–UC-48 → FR-40–FR-44/NFR-19 → QAS-46–QAS-50 → DDR-16 → DD-61–DD-65 (user request 2026-10-11).
- [x] Increment 1 — slug model, tree creation and local exclusion (DD-61): `src/slugs.rs`, `setup/slug.rs`, `create_slug`, slugs preserved by save; `tests/slugs.rs`.
- [x] Increment 2 — progress derivation and tree verifier (DD-63): `src/slug_progress.rs` (stages from file locations; mismatches for duplicates, undeclared files/phases, missing tasks, out-of-order phases); `tests/slug_progress.rs`.
- [ ] Increment 3 — plan rules file and workspace override (DD-65). Core done: `src/plan_rules.rs` (default rules, `~/.agentconductor/plan-rules.md` created once, override precedence), `PlanInputs`, `plan … --rules`; `tests/plan_rules.rs`. Pending: workspace `plan_rules` field, startup passing effective rules, editor (waits for Codex's uncommitted setup/UI work).
- [ ] Increment 4 — phased planning, one run per phase, task file movement (DD-62). Core done: `src/phases.rs` (write analysis/plan/phase/task files once, 1–6 phases × 1–12 tasks, safe ids, run plan only for the current phase with earlier-phase dependencies dropped, single-step atomic moves); `tests/phases.rs`. Coordinator phased planning done: `plan … --phases <slug-dir>` asks for analysis + 1–6 phases, validates each phase like a plan with role rules, writes the slug tree and returns phase one as the reviewable plan (`tests/phased_planning.rs`). Run hooks done: `run … --phase <slug-dir> <phase-id>` links the run (`slug-phase.json`); a tested candidate moves the phase to verification and `workflow::merge` (CLI and window) moves it to completed, reporting but not rolling back a failed tree update (`tests/phase_link.rs`). Pending: UI (startup-run passing the link).
- [x] Increment 5 — slug archives in export/import (DD-64): `src/archive.rs` (deterministic store-only ZIP, CRC-32, UTF-8 names, unsafe-entry refusal, staged extraction; validated with system `unzip -t`), `share.rs` export lists `{slug, goal, created_ms, archive, progress}` and import stages/verifies every archive before replacing (backup `.bak`); `tests/archive.rs`, `tests/slug_archives.rs`.
- [ ] Increment 6 — window: slug list, progress, plan rules editor.
  - [x] 6a backend: Generate/Read plan create or reuse the goal's slug and request phases (`--phases`) with the effective rules (`--rules`); a coordinator returning only tasks becomes phase `01-plan`; settle records the proposal's slug phase; Start passes `--phase` to `startup-run`; `next_phase` builds the next phase from the tree without a coordinator; `save_plan_rules` (workspace override / global file, written only when saved); snapshot lists slugs with progress and the current global rules; `Request` moved to `setup/request.rs`. `tests/slug_flow.rs` covers goal → slug → phase 1 → verification → merge → completed → next phase → rules; full regression 294 passed / 12 ignored.
  - [x] 6b page: Slug field (suggested from the goal), Goals & progress card (stages, per-phase counts, mismatch warnings, Plan next phase), Workspace ▾ Plan rules… dialog (global/workspace); `tests/slug_page.rs`; manual §1.4c. Full regression 295 passed / 12 ignored.
  - [ ] Native window check.

### Idle footprint (R-65 / QAS-58 / DD-71)

- [x] Plan: R-65 → NFR-05 → QAS-58 → DDR-06 → DD-71 (user directive 2026-10-11: the product exists to save resources).
- [x] Task / Red: `tests/idle_footprint.rs` failed (1 s `setInterval` in both pages, icons in every snapshot).
- [x] Task / Green: adaptive self-rescheduling refresh (1 s active, 5 s idle, paused while hidden) in workspace and Agent Room; snapshots carry icon data URLs only when `icons: true` (first load / agent kinds change).
- [x] Verify: release build `ui-new`, 30 s idle after 8 s warm-up — before: RSS 105–106 MiB, CPU avg ≈1.8 % with 3–4.5 % spikes; after: CPU avg ≈0.12 % (one 1.4 % sample), RSS 105 MiB right after opening settling to ≈88 MiB. CPU target met; RSS exceeds 100 MiB briefly at window start (WebKit helpers not attributed). Full regression 286 passed / 12 ignored / 0 failed.
- [x] Whole tree (release `ui-new`, 34 s idle): main ≈107 MiB + WebKit GPU/WebContent/Networking helpers ≈85 MiB = ≈192 MiB; average CPU ≈0.49 % across the tree. Quota/discovery children are short-lived and not resident.
- [ ] Follow-up: Agent Room during a run; bring the main process under 100 MiB at window start.

### Empty default agent list (R-66 / QAS-59 / DD-72)

- [x] Plan: R-66 → BG-09 → SF-41 → QAS-59 → DDR-13 → DD-72 (user request 2026-10-11; tests add their agents first).
- [x] Task / Red → Green: `tests/empty_default_agents.rs`; new workspaces have no agents and no coordinator, templates remain in + Add, empty registry saves, planning refused until an agent exists; quota and registry tests now register template agents at start. Full regression 287 passed / 12 ignored / 0 failed.


## Roadmap — messenger integration (R-43, planned)

Not started. Each phase follows Plan → Task → Verify → Complete with TDD.

- [ ] Plan: confirm DD-51 with the user — target company messenger and its API, relay vs. polling, who may approve, default redaction.
- [ ] Phase 1 — notifications: channel port over the audit outbox; Slack (Socket Mode) and generic webhook connectors; per-workspace channel settings and credential-store secrets; QAS-35.
- [ ] Phase 2 — decisions: Allow/Deny for pending tool requests with signature, linked approver, exact binding, expiry and replay checks; QAS-36.
- [ ] Phase 3 — company messenger decisions through the confirmed mechanism; briefing on request.
- [ ] Verify each phase with fixture servers in tests plus a live Slack workspace and the company messenger provided by the user.

### Claude quota collection — DD-66

- [x] Plan: trace R-60 → BG-22 → SF-50 → UC-49 → QAS-51 → DD-66.
- [x] Task / Red: stream conversion, unknown/malformed metadata, live persistence and metadata-only refresh regressions.
- [x] Task / Green: owned-stream collection and native board integration.
- [x] Verify: focused tests (21), source layout, complete local regression (268 passed / 11 ignored), formatting/lint. Windows validation is blocked by GitHub runner billing.
- [x] Complete: manual, changelog, source-linked wiki, agentmemory and commit/push.
- Optimized local macOS build passed. No new release published; actual Enterprise quota emission and Windows runner validation remain unverified.

### Inline agent quota — DD-67

- [x] Plan: R-61 → BG-22 → SF-51 → UC-50 → FR-46/NFR-21 → QAS-52 → DD-67.
- [x] Task: TDD badge behavior and retained-workspace snapshot.
- [x] Verify: 18 focused Rust checks, 1 Python/Node display check and JavaScript syntax; optimized build.
- [x] Complete: docs/wiki/memory, commit/push and replace the previous runtime.

### MCP Jira/Confluence goal imports — DD-68 (active)

- [x] Plan: R-62 → BG-23 → SF-52 → UC-51/52 → FR-47/48, NFR-22 → QAS-53–55 → DD-68.
- [x] Task 1 / TDD: OpenCode-compatible configuration, source bindings, redaction and settings UI.
- [x] Task 2 / TDD: explicit keywords/chips and confirmed shorthand resolution.
- [x] Task 3 / TDD: bounded read-only MCP retrieval and preview/apply with provenance.
- [x] Task 4 / TDD: imported snapshots → goal/slug → phased planning and local wiki provenance index; summaries remain reviewed knowledge-phase output.
- [x] Verify / local: local/remote connector fixtures, missing records, partial results, import safety, native preview/apply and macOS regression.
- [ ] Verify / company: authorized live company server and authentication.
- [ ] Complete: English operator docs, source-linked wiki, agentmemory and reviewed delivery.

### Workspace quota collection regression

- [x] Reproduce: active workspace has no last run; display fixtures did not test collection.
- [x] Red: fresh-workspace Codex quota request fails as unknown request.
- [x] Green: independent cache and CLI collection; actual Codex/Claude checks.
- [x] Verify: collection without runs, provider errors, cleanup, refresh scheduling and actual account values; native visual review remains separate.
- [x] Complete: docs/wiki/memory and replaced the prior owned runtime; fresh real account observations confirmed.

### Agent-specific refresh and action icons — DD-69

- [x] Plan: R-63 → BG-22 → SF-53 → UC-53 → FR-49/NFR-23 → QAS-56 → DD-69.
- [x] Task: per-agent two-minute scheduler, accessible refresh icons and Edit/Models/Remove icons.
- [x] Verify: periodic boundary checks, workspace/run cache binding, native CLI collection and cleanup, source layout and Clippy; optimized build checked before runtime replacement.
- [x] Complete: documentation/wiki/memory and runtime replacement; one native app process confirmed.


### Compact quota window presentation — DD-70

- [x] Plan: R-64 → BG-24 → SF-54 → UC-54 → FR-50/NFR-24 → QAS-57 → DD-70.
- [x] Task / Red: single-minimum display and absent plan_type fail new multi-window/Enterprise tests.
- [x] Task / Green: compact quota-controls group, provider duration labels and backward-compatible plan metadata.
- [x] Verify: 282 full Rust checks / 12 ignored before final UI refinement; final focused 14 / 1 ignored, JS display check, live account refresh, async delayed-provider and duplicate-request check, lint/build.
- [x] Complete: wiki/memory and replaced runtime with one-line numeric quota controls.

- [x] DD-70 interaction refinement: enabled button/icon pointer cursors and 5px quota-refresh spacing.


### Stable release numbering and operator screenshots

- [x] Plan: current version 0.1.0, subsequent version 0.1.1; GitHub publications are stable releases, not pre-releases.
- [x] Task / TDD: v0.1.0 notes regression fails before section migration; stable publish workflow removes --prerelease.
- [x] Verify: release-note tests, updated English operator manual and actual native screenshots.
- [x] Complete: current changes delivered with preview history preserved.

### Agent preferences and native AC identity — DD-73

- [x] Plan: R-67 → BG-25 → SF-55 → UC-55 → FR-51/NFR-25 → QAS-60 → DD-73.
- [x] Task/TDD Red: missing cadence field and outdated two-minute scheduler fail new checks.
- [x] Task/Green: settings, executable picker, consistent inputs, silent quota and native icon.
- [x] Verify: 293 Rust tests passed / 12 ignored / 0 failed, 22 Python checks, formatting, Clippy, JS syntax, optimized macOS build, actual native UI/screenshots and packaged shared/worktree smoke.
- [x] Complete: English manual/wiki, shared memory, reviewed commits and delivery.
- [x] Platform release follow-up: repository made public; v0.1.1 published with CI-verified macOS and Windows packages (Windows UI verification on a real machine remains open).

### Provider-driven profile and quota reset time — DD-74 / DD-75

- [x] Plan: R-68/R-69 → BG-26 → SF-56/57 → UC-56/57 → FR-52/53/NFR-26 → QAS-61/62 → DD-74/75.
- [x] Task/TDD Red: `tests/provider_switch.rs` (4) and `tests/test_quota_reset.py` failed first.
- [x] Task/Green: `renames` in `save_profiles`, provider onchange with `uniqueId`, dialog order, `quotaBadge` reset suffix in both pages.
- [x] Verify: 299 Rust passed / 0 failed, Python suites, clippy `-D warnings`, fmt, JS syntax.
- [x] Complete: manual, CHANGELOG (v0.1.1), wiki, agentmemory, commit/push.

### Interface language — DD-76

- [x] Plan: R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76.
- [x] Task/TDD Red: `tests/interface_language.rs` (4) and `tests/test_utf8_sources.py` (2) failed first.
- [x] Task/Green: `language.rs`, `i18n.js`, `i18n_ko.js` (318 entries), Settings dialog; textarea placeholders translated, text kept.
- [x] Verify: 303 Rust passed / 0 failed, 26 Python, clippy, fmt; native macOS window in Korean (`docs/assets/workspace-korean.jpg`), saved choice reloaded.
- [x] Complete: manual §1.2c, CHANGELOG, wiki, agentmemory, commit/push.

### Fallback model list and icon-only Settings — DD-77

- [x] Plan: R-71 → SF-59 → UC-59 → FR-55 → QAS-64 → DD-77; R-70 amendment.
- [x] Task/TDD Red: `tests/fallback_models.rs` (3) failed first.
- [x] Task/Green: `fallbackModels`, poll refresh, help text, ⚙ button, Korean entries.
- [x] Verify: full suites, clippy, fmt.
- [x] Complete: manual, CHANGELOG, commit/push.

### Messenger notifications phase 1 — DD-79

- [x] Plan: R-43 activation → BG-18 → SF-37 → UC-34/UC-61 → FR-28/NFR-17 → QAS-35/QAS-66 → DD-79.
- [x] Task/TDD Red: `tests/messenger.rs` (3 + 1 live) and `tests/messenger_settings.rs` failed first (module absent).
- [x] Task/Green: `src/messenger.rs`, `src/messenger/send.rs`, `workflow::save` and `Startup::settle` hooks, run link before `startup-run`, `messenger`/`save_messenger`/`test_messenger`, Messenger dialog, Korean strings, flush before CLI exit.
- [x] Verify: 310 Rust passed / 0 failed, Python suites, clippy, fmt; live delivery through the system curl to a local HTTPS receiver (Korean text, quotes, JSON content type; refused port reported).
- [x] Complete: manual §1.2d, CHANGELOG v0.1.2, wiki, agentmemory, commit/push.
#### DD-68 implementation acceptance — 2026-10-11

- [x] Plan: activate existing R-62 / BG-23 / SF-52 / UC-51–52 / FR-47–48 / NFR-22 / QAS-53–55 / DD-68.
- [x] Task / Red: missing module/UI, held-open SSE timeout and duplicate local session regressions reproduced.
- [x] Task / Green: workspace settings, on-demand discovery/import, shorthand confirmation, selected source evidence and private wiki index.
- [x] Verify / macOS: local/remote fixtures, child cleanup, stale/tampered evidence, partial failures, native preview/apply, 320 Rust tests and 32 Python tests.
- [x] Verify / Windows: CI 38134378978 passed native tests, all Python contracts, fmt/Clippy and exact packaged shared/worktree execution on source 5cefa6c.
- [x] Complete: English manual/wiki, agentmemory, reviewed commits and verified stable v0.1.2 packages; macOS runtime restarted with the CI binary.
- [ ] Company acceptance: authorized company MCP server and authentication configuration supplied by the operator.


DD-68 UI amendment (2026-10-11): configure MCP connections through the top-bar Settings menu; keep reference insertion and preview beside the goal. The Settings gear uses a 26 px glyph in a 32 × 32 px pointer target.


### Windows quota and icon correction — DD-78

- [x] Plan: R-72 → BG-28 → SF-60 → UC-60 → FR-56/NFR-28 → QAS-65 → DD-78.
- [x] Task / Red: missing fallback badge, launcher precedence and Windows ConPTY timeout reproduced (CI 38131529935); user confirmed error 193.
- [x] Task / Green: extension-aware launcher discovery/query recovery, bounded ConPTY with explicit standard handles, original provider badges.
- [x] Verify / macOS: 320 Rust tests, 32 Python tests, fmt/Clippy and release package smoke (shared/worktree, approval and merge).
- [x] Verify / Windows: CI 38134378978 passed 135 native tests (0 failed, 1 ignored), 32 Python tests, saved npm Codex profile, spaced native/batch Claude quota, trust refusal, icon fixtures and exact packaged execution.
- [ ] Subscription acceptance: user Windows account refresh after installing the corrected build.
- [x] Complete: manual/wiki, agentmemory, reviewed commits and native stable v0.1.2 delivery.


### Custom CLI models without discovery - DD-80

- [x] Plan: R-73 -> BG-29 -> SF-61 -> UC-62 -> FR-57/NFR-29 -> QAS-67 -> DD-80.
- [x] Task / Red: missing manual parser and capability guidance reproduced.
- [x] Task / Green: bounded ID editor, retained IDs, observed JSON compatibility, explicit operator provenance.
- [x] Verify: actual JavaScript parser/UI contracts, durable catalog without a discovery job, translations and native Korean dialog; 320 Rust tests and 32 Python tests passed locally.
- [x] Complete: English manual/wiki, agentmemory, committed changes and updated native runtime; stable release delivery is tracked with the platform checks above.

### Stable v0.1.2 delivery — 2026-10-11

- [x] Plan: preserve v0.1.1 and publish the next stable version without a prerelease flag.
- [x] Task: package exact source 5cefa6ce8a29ee7cdf4f19e302a440be2959d1e6 for macOS Apple silicon and Windows x86_64.
- [x] Verify: [CI 38134378978](https://github.com/neodamul/AgentConductor/actions/runs/38134378978), four jobs passed; macOS 320 Rust tests / Windows 135 Rust tests, 0 failures; 32 Python tests on each platform; fmt and Clippy passed.
- [x] Verify: each packaged binary executed three parallel tasks and three tests in shared and worktree modes; unapproved merge refused and approved merge verified.
- [x] Verify: downloaded archive and binary SHA-256, ZIP integrity, source manifest, dependency notices and SBOM; 42 macOS / 108 Windows dependencies.
- [x] Complete: [stable v0.1.2](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.2), both archives and SHA256SUMS uploaded; current macOS app replaced with the verified CI binary while preserving the workspace and reviewed plan.

External acceptance remains separate: real company MCP authentication and the operator Windows subscription account have not been supplied to CI. The existing macOS accessibility issue after sequential Settings/MCP dialogs remains open; native MCP preview/apply was visually verified, while the custom model dialog exposes its controls in the accessibility tree.

### Messenger decisions phase 2 — DD-81

- [x] Plan: R-43 phase 2 → BG-18 → SF-37 → UC-35/UC-63 → FR-58/NFR-30 → QAS-36/QAS-68 → DD-81.
- [x] Task/TDD Red: `tests/messenger_inbound.rs` (3 + 1 live) failed first (module and dialog fields absent); `messenger_settings` extended for the new secrets.
- [x] Task/Green: `src/messenger/inbound.rs` (codes, parse, RFC 2104 HMAC, Slack/relay readers, `poll_once`, `spawn`), `send::fetch`, named secrets and `status`, `src/ui/setup/messenger.rs`, startup-run hook, dialog fields, Korean strings.
- [x] Verify: 323 Rust passed / 0 failed, Python suites, clippy, fmt; live round trip through the system curl against a local HTTPS stand-in for Slack (announce → history with Bearer token → deny applied → confirmation).
- [x] Complete: manual, CHANGELOG v0.1.3, wiki, agentmemory, commit/push.

### Automatic slug naming — DD-82

- [x] Plan: R-74 → BG-12 → SF-46 → UC-64 → FR-59 → QAS-69 → DD-82.
- [x] Task/TDD Red: `tests/slugs.rs` (romanization, date fallback, truncation) and `tests/slug_page.rs` (no input) failed first.
- [x] Task/Green: Revised Romanization tables in `slugs::suggest_at`, UTC civil date fallback, on-disk collision check in `create_slug`, slug field and script removed.
- [x] Verify: 323 Rust passed / 0 failed, Python suites, clippy, fmt.
- [x] Complete: manual §1.4c, CHANGELOG v0.1.3, wiki, agentmemory, commit/push.
### Settings button spacing amendment — DD-68 / 2026-10-11

- [x] Plan: retain the visible settings gear and menu access while reducing excess title-bar button whitespace.
- [x] Task / Red: native screenshot showed the 44 × 44 px button disproportionately larger than adjacent title-bar actions.
- [x] Task / Green: reduce the button to 32 × 32 px; preserve its 26 px glyph, centered alignment, pointer cursor and focus indication.
- [x] Verify: 4 existing MCP UI contracts passed, optimized build succeeded and native Settings menu opened/closed correctly; screenshot confirms reduced button whitespace.
- [x] Complete: previous owned app stopped, updated runtime reopened with the existing workspace/plan preserved; correction recorded in shared memory.

### Enterprise allocation correction — DD-83

- [x] Plan: R-75 → BG-30 → SF-62 → UC-65 → FR-60/NFR-31 → QAS-70 → DD-83.
- [x] Task / Red: installed Codex protocol budget-only Enterprise response drops individualLimit/credits; regression fails.
- [x] Task / Green: explicit budgets, scope, remaining/allocated and conservative unknown totals.
- [x] Verify / macOS: 323 Rust tests passed, 0 failed, 13 ignored; 32 Python tests passed; fmt/Clippy and optimized build passed. Metadata-only subprocess fixture returns Enterprise allocation 640/1000; native ordinary-account refresh remains functional.
- [ ] Windows Enterprise acceptance: actual Codex/Claude errors and CLI metadata supplied by the operator.
- [ ] Complete: confirmed Codex budget correction is implemented and running locally; both actual Windows Enterprise accounts, Claude Enterprise output and platform CI remain pending.


### Native window behavior correction — DD-84 / v0.1.4

- [x] Plan: R-76 → BG-31 → SF-63 → UC-66 → FR-61/NFR-32 → QAS-71 → DD-84.
- [x] Task / Red: picker is owned by an external script; Windows GUI subsystem and hidden background flags absent.
- [x] Task / Green: main-thread NSOpenPanel, Windows GUI subsystem, console-free background commands and retained CLI parent console.
- [x] Verify / macOS: 328 Rust tests passed, 0 failed, 14 ignored; 32 Python tests; fmt/Clippy and release build passed. Native Browse opens the foreground panel, Cancel preserves fields, selecting the existing repository returns the correct path; saved workspace preserved.
- [x] Verify / Windows: CI 38189018437 passed 143 native tests (0 failed, 1 ignored), including PE GUI subsystem, actual console-free child/pipe fixture, picker owner script and native metadata checks; 32 Python tests, fmt/Clippy passed.
- [x] Verify / packaging: CI 38189018437, all four jobs passed; exact macOS/Windows archives executed shared/worktree parallel workflows, tests, unapproved merge refusal and approved merge; archive/binary SHA-256, source manifest, license notices and SBOM verified.
- [x] Complete: [stable v0.1.4](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.4) published from 0b006b2ac2f1ee4c47a16e37621ca30301b0773f; both native archives and SHA256SUMS uploaded with server digests verified. Previous owned runtime stopped; exact CI macOS binary reopened at /private/tmp/AgentConductor-v0.1.4.app; existing workspace and reviewed plan preserved.
- [ ] Operator acceptance: interactive Windows desktop Browse z-order on the operator machine (CI validates the native owner bootstrap and console behavior).


### New workspace save correction — DD-85

- [x] Plan: R-77 → BG-32 → SF-64 → UC-67 → FR-62/NFR-33 → QAS-72 → DD-85.
- [x] Task / Red: saving a plain directory reproduces git rev-parse not-a-repository; unborn planning fails HEAD.
- [x] Task / Green: validate directory at save; validate baseline before planning artifacts; retain independent quota collection.
- [x] Verify / macOS: 331 Rust tests passed, 0 failed, 14 ignored; 34 Python tests passed, including actual UI cooldown scheduling. Fresh non-Git save/reopen, invalid directory preservation, unborn baseline rejection without artifacts, and quota subprocess without Git/run pass.
- [ ] Complete: reviewed commit, native runtime and shared memory.
- [x] Product flow: initialization prototype superseded by ordinary folders under DD-88.
- [x] Product correction: ordinary-folder execution is active under DD-88; private history uses Git internally.
- [ ] Reported quota acceptance: operator OS and exact error pending; local current paid-account refresh works.

- [x] Task / Red (quota): paused provider fixture reproduces a shared planning slot and blocked Save.
- [x] Task / Green (quota): separate metadata broker and quota_job ledger; refresh scheduler waits for its slot without skipping the next provider for five minutes.
- [x] Verify (quota): Save succeeds while provider is paused; metadata has no planning job; duplicate refresh is refused; independent fresh-folder subprocess and actual UI scheduling/cooldown tests pass.


### Nonblocking team configuration — DD-86 / v0.1.5

- [x] Plan: R-78 → BG-33 → SF-65 → UC-68 → FR-63/NFR-34 → QAS-73 → DD-86.
- [x] Task / Red: paused model query occupies planning slot; metadata scheduling changes the revision and blocks registration.
- [x] Task / Green: bounded one-worker FIFO, isolated model snapshots, matching-profile catalog application, stable edit revision and immediate guarded registration dialog.
- [x] Verify / targeted: catalog pause/queue/registration/stale-result fixtures and 37 actual JavaScript/Python tests passed; native Models query permits opening/canceling registration and preserves the reviewed plan. Full platform/package verification remains below.
- [ ] Complete: native restart, English manual/README, commit/push, stable v0.1.5 binaries/checksums and shared memory.
- [x] Coordination: agentmemory signal to Claude requesting remote-work registration; Codex retains ownership of metadata/UI/release. Remote registration is separate from this release.


### Superseded Git initialization prototype — DD-87

- [x] Plan: R-79 → BG-34 → SF-66 → UC-69 → FR-64/NFR-35 → QAS-74 → DD-87.
- [x] Task / Red: initialization request absent; plain-project planning only fails a prerequisite.
- [x] Task / Green: dedicated consent, empty tree/commit, compare-and-set HEAD, untouched index and local audit; separate file-commit consent remains.
- [ ] Verify: false approval, replay, staged/untracked preservation, actual JavaScript continuation and macOS/Windows full regression.
- [x] Closed as superseded: user selected ordinary-folder operation under DD-88. Mandatory initialization and its unreleased UI/API were removed; this prototype is not shipped.


### Ordinary project folders — DD-88 / v0.1.5

- [x] Plan: R-80 -> BG-35 -> SF-67 -> UC-70 -> FR-65/NFR-36 -> QAS-75 -> DD-88.
- [x] Task / Red: folder workflow module absent; existing planning requires Git root/HEAD.
- [x] Task / Green: private bounded folder history, immutable origin fingerprints, plan/run binding, original-change refusal and approved file application.
- [x] Verify / local: real planning subprocess, native startup request, three-worker shared/worktree execution, unapproved refusal, original-edit preservation, exclusions/aliases and modification/deletion backups pass. Full macOS regression: 339 passed, 0 failed, 14 ignored; subsequent five folder tests include the new backup case. 37 Python tests, fmt and Clippy pass.
- [ ] Verify / platforms: exact final-source macOS/Windows CI and packaged four-case Git/folder × shared/worktree smoke.
- [ ] Complete: native interaction, English README/manual, shared memory, source commit/push and stable macOS/Windows v0.1.5 release.
- [x] Scope correction: Git initialization is optional. DD-87 is not the mandatory fresh-folder flow; DD-88 supersedes that prerequisite.

DD-86 live verification follow-up (passed): actual native Models refresh invalidated the quota cache because catalog observations were hashed as account identity. Add a failing retained-capacity test; exclude catalog metadata from the quota fingerprint while executable/auth settings remain binding.
