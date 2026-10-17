# Implementation plan and TDD

## Active issue #9 status (2026-10-17)

- [x] Plan: reconcile issue #9 with the current canonical branch and its Rust-only application/build-verification scope.
- [x] Task / Red: run the 300-line source-layout test and record the remaining oversized Rust modules and fixture files.
- [x] Task / Green / Refactor: remove legacy browser UI assets and Python/Node-owned tests; keep localization and interaction coverage in Rust native UI tests.
- [x] Task / Green: port subprocess test fixtures to Rust and gate the standalone test fixture binary behind `test-fixtures`.
- [x] Task / Refactor: split oversized Rust modules while preserving their public call paths; every scanned Rust source and test file is at most 300 lines.
- [x] Task / Red-Green: retain Agent Room summary, activity, and inspector as ordered native columns; render inspector selection before requesting activity details and keep task actions ahead of background detail requests.
- [x] Task / Green: make workspace stages vertically scrollable, keep the Settings menu readable, and retain goal text contrast in the Rust theme.
- [x] Verify: native macOS Agent Room fixture visibly renders summary/activity/inspector in three columns; AccessKit exposes all three regions and the status controls.
- [x] Verify: 50 focused native UI tests passed across 7 suites; `cargo fmt --all -- --check`, `git diff --check`, and native-feature all-target Clippy passed.
- [x] Verify: `cargo test --offline --all-targets --features test-fixtures` — 596 passed, 14 ignored, 0 failed across 203 suites.
- [x] Verify: `cargo check --offline --all-targets --features test-fixtures`, `cargo fmt --all -- --check`, and `git diff --check` passed.
- [x] Verify: `cargo clippy --offline --all-targets --features test-fixtures -- -D warnings` passed after the extracted helper borrow fixes.
- [x] Verify: `cargo build --offline --release --locked --bin agentconductor` passed; packaged binary `--help` smoke exited successfully without opening the desktop UI.
- [x] Verify: exact-source macOS package at `98d5357`; ZIP SHA-256 `9b5f9a76a177887865e6ffca585c64f0e341a87889811ff22fa7ed8169846190`; all four packaged smoke scenarios and archive integrity passed locally.
- [x] Verify: exact upstream license/source provenance and package metadata for all 214 active host dependencies; focused provenance/package/smoke tests and Clippy passed.
- [ ] Verify: exact-source macOS/Windows run `38995553011` for `98d5357`, including native package builds and packaged smoke; GitHub jobs are currently building. This Mac lacks the Windows target standard library and linker.
- [ ] Verify: replace public Agent Room screenshots from the v0.1.21 UI with reviewed captures of the current Rust native UI; complete live Korean IME and Windows UI/accessibility checks. Workspace screenshots and bilingual HTML manuals already use the native workspace capture.
- [x] Verify: controlled idle sample of the native macOS Agent Room on a completed local fixture over three five-second `top` samples: 0.0%, 1.2%, 1.6% CPU and 58 MB MEM; no child process. The idle view retains its five-second refresh cadence.
- [x] Complete / source-migration increment: pushed `3796b54`; updated this plan, the issue, the migration wiki, AgentMemory, and Claude handoff.
- [ ] Complete issue #9: finish exact-source Windows package and native UI acceptance, replace the public Agent Room screenshots, and complete the remaining license-attribution review; close only when every acceptance gate passes.

Issue #9 remains open. Exact-source macOS packaging and license provenance are verified locally; cross-platform CI and native product-acceptance checks remain pending.
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

## T-10 — Native Workspace profile controls and localization (GitHub issue #9)

- [x] Plan: keep the existing Rust dispatcher and profile API; localize the Team/profile workflow in English and Korean.
- [x] Task: localize role, profile, model/quota refresh, and removal-confirmation controls with accessible names.
- [x] Verify: 20 focused tests pass across profile localization, profile actions, role assignment, pending status, and i18n contracts; native-feature all-target Clippy passes with warnings denied. The broader run reported 533 passed and 14 ignored across 158 suites before the RTK runner was interrupted after its summary (exit 130).
- [x] Task: split touched Rust modules so every changed source and test file stays within 300 lines.
- [x] Verify: repository-wide source-layout gate; 34 unrelated existing files still exceed 300 lines.
- [x] Complete: pushed `863f2ae`, saved AgentMemory fact `mem_mvb0p8wv_fdd4b446642a`, signaled Claude, and updated GitHub issue #9 progress. The issue remains open.

## T-11 — Bilingual native workspace settings sharing (GitHub issue #9)

- [x] Plan: extend existing settings-sharing use case UC-42 and backend contract R-52/DD-59 under the Rust UI migration decision DD-160 without changing request/approval semantics.
- [x] Red: new AccessKit tests failed on untranslated preview, dirty-state, permission, and result labels.
- [x] Green: localize the preview, confirmation, permission choice, and async completion results; retain the compact English prompt and existing accessible action names.
- [x] Task: keep import/export result data language-neutral so changing the selected language also updates already-visible results.
- [x] Verify: 13 tests pass across sharing-language, workspace, and i18n suites; 13 native probe checks and native-feature all-target Clippy pass. Formatting and diff checks pass. Graphify: 6,301 nodes / 10,546 edges / 478 communities.
- [x] Verify: changed Rust files are below 300 lines; repository-wide layout still reports 34 pre-existing oversized files.
- [x] Complete: refreshed Graphify and wiki; saved AgentMemory, signaled Claude, posted issue #9 progress, and committed/pushed this slice.

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
- [x] Task/Green: `language.rs`, `messages.rs`, `i18n.js`, English/Korean JSON catalogs and explicit ID bindings; Settings dialog; textarea placeholders translated, text kept.
- [x] Verify: 303 Rust passed / 0 failed, 26 Python, clippy, fmt; native macOS window in Korean (`docs/assets/workspace-korean.jpg`), saved choice reloaded.
- [x] Complete: manual §1.2c, CHANGELOG, wiki, agentmemory, commit/push.

- [x] 2026-10-13 amendment: app-owned web/native messages use stable IDs and named parameters; locale data is exempt from the code line limit. Catalog coverage and live language-switch checks are in `tests/messages.rs`, `tests/i18n_runtime.cjs` and `tests/test_i18n_catalogs.py`.

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
- [x] Complete: stable v0.1.5 published; reviewed source 3ce28b0, exact CI macOS runtime reopened, existing workspace/plan preserved; shared memory recorded.
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
- [x] Complete: exact CI native restart, English manual/README, commits pushed, stable v0.1.5 macOS/Windows binaries and server-verified checksums; shared memory recorded.
- [x] Coordination: agentmemory signal to Claude requesting remote-work registration; Codex retains ownership of metadata/UI/release. Claude confirmed this request meant the connected claude rc session; the registration item is closed.


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
- [x] Verify / platforms: CI 38199934220 passed macOS 342 tests (14 ignored), Windows 157 (1 ignored), 0 failures; Python 37, fmt/Clippy and four-case Git/folder × shared/worktree smoke passed for each exact package.
- [x] Complete: native interactions, English README/manual and wiki updated, source pushed; stable macOS/Windows v0.1.5 released from 3ce28b08a832c5d1aead4523fb9c04314119ca24; exact CI macOS app running.
- [x] Scope correction: Git initialization is optional. DD-87 is not the mandatory fresh-folder flow; DD-88 supersedes that prerequisite.

DD-86 live verification follow-up (passed): actual native Models refresh invalidated the quota cache because catalog observations were hashed as account identity. Add a failing retained-capacity test; exclude catalog metadata from the quota fingerprint while executable/auth settings remain binding.

DD-88 Windows CI correction: CI 38196650126 passed macOS but Windows folder workflow failed when Git retention ref paths exceeded MAX_PATH. Enable core.longpaths per Git invocation, preserving global user configuration; rerun the same real workflow regression before packaging.

### Claude pre-release review — DD-86 / DD-88

- [x] Plan: QAS-76 resilience amendment to R-78/R-80, preserving NFR-05 resource constraints.
- [x] Task / Red: invalid catalog poisons Snapshot; unstarted replaced copies remain; completed working files remain.
- [x] Task / Green: durable failed metadata observations, continued Save/retry, superseded/orphan pruning, completed working-file compaction with retained Git objects, candidate-stage cleanup and rollback diagnostics.
- [x] Verify / local: restart/retry, queue/registration continuity, evidence/knowledge access after compaction and source layout pass; full Rust regression 342 passed, 0 failed, 14 ignored, plus 37 Python tests and fmt/Clippy.
- [x] Verify / platforms: reviewed-source CI 38199934220 all four jobs passed; 342 macOS and 157 Windows tests, 0 failures; four-case exact package smoke passed on both.
- [x] Complete: reviewed source 3ce28b0, manual/README shipped in archives, stable v0.1.5 published with verified server digests and native restart.

Prior source d9e4a38 passed all four jobs in CI 38197252244, including Windows long-path regression and four-case package smoke. Review hardening requires a new final-source CI before release.

### Stable v0.1.5 delivery evidence

Source: 3ce28b08a832c5d1aead4523fb9c04314119ca24. CI: 38199934220, all four jobs passed. Release: https://github.com/neodamul/AgentConductor/releases/tag/v0.1.5 (draft=false, prerelease=false). macOS archive SHA-256: 9abc807a519704a0a2834a848007c6b021c89004506891f0814b023e0d394d9e. Windows archive SHA-256: beab75cb6c894c52c51a7b3d94509ecbc79bfd075b7af39c46f3ac72f5933003. SBOM includes 42 macOS / 108 Windows dependency records with license texts/notices; no new dependency introduced. Packaged README/manual/license contents match source, allowing Windows CRLF checkout. Exact CI app is running at /private/tmp/AgentConductor-v0.1.5.app; existing health workspace and reviewed plan retained. Native quota shows actual authenticated Claude/Codex results; actual Windows Enterprise operator acceptance remains tracked separately.


### Agent tools guide — DD-89

- [x] Plan: R-81 -> BG-36 -> SF-68 -> UC-71 -> FR-66/NFR-37 -> QAS-77 -> DD-89.
- [x] Task / Red: tests/test_agent_tools_guide.py failed with no guide dialog or button.
- [x] Task / Green: Settings button, static dialog, Korean strings and manual section 1.2e.
- [x] Verify / local: 340 Rust passed, 0 failed, 14 ignored; 41 Python tests, fmt and Clippy pass; English and Korean dialog rendered in headless Chrome with one command per line.
- [ ] Verify / platforms: native app dialog check and macOS/Windows CI after merge.
- [ ] Complete: merge after the v0.1.5 release, changelog entry, shared memory.


### Windows picker path — DD-90

- [x] Plan: R-82 -> BG-37 -> SF-69 -> UC-72 -> FR-67/NFR-38 -> QAS-78 -> DD-90.
- [x] Task / Red: tests/picker_output.rs failed (no parsing function); Windows Korean round trip added to tests/windows_console.rs.
- [x] Task / Green: UTF-8 without BOM in the owner script; BOM-safe picked().
- [ ] Verify: Windows CI round trip and user confirmation on the reporting Windows machine.


### Planning start and activity — DD-91

- [x] Plan: R-83 -> BG-38 -> SF-70 -> UC-73 -> FR-68/NFR-39 -> QAS-79 -> DD-91.
- [x] Task / Red: tests/test_planning_feedback.py and the job.output assertion in tests/planning_progress.rs failed.
- [x] Task / Green: 300 s planning-entry limit, snapshot output activity, waiting/active/stale text, reduced-motion blink.
- [ ] Verify: full regression, macOS/Windows CI and user confirmation on Windows.


### English slugs — DD-92

- [x] Plan: R-84 -> BG-39 -> SF-71 -> UC-74 -> FR-69/NFR-40 -> QAS-80 -> DD-92.
- [x] Task / Red: Codex's tests (tests/slugs.rs, tests/startup_folder_plan.rs with AC_PLAN_SLUG mock) failed on romanization.
- [x] Task / Green: date placeholder for non-English goals, prompt slug, slug.txt, settle rename.
- [ ] Verify: full regression, CI, real coordinator naming on the user's goal.


### Enterprise percentage — DD-93

- [x] Plan: R-85 -> BG-40 -> SF-72 -> UC-75 -> FR-70/NFR-41 -> QAS-81 -> DD-93.
- [x] Task / Red: Codex's tests/test_quota_badge.py expectations for 64%/100% failed.
- [x] Task / Green: percentage amount in both quotaBadge copies.
- [ ] Verify: real Enterprise account; Claude Enterprise usage on Windows still needs the actual /usage output or error to diagnose.


### Windows Claude capacity — DD-94 / v0.1.7

- [x] Plan: R-86 -> BG-41 -> SF-73 -> UC-76 -> FR-71/NFR-42 -> QAS-82 -> DD-94.
- [x] Task / Red: claude_blocker missing; Windows verbatim-folder and first-run fixture cases added.
- [x] Task / Green: git_path working folder, shared claude_blocker on macOS and Windows.
- [ ] Verify: Windows CI, then the user's Windows refresh after completing Claude CLI first-run setup.

- [x] DD-91 amendment: snapshot poll waits up to 300 s while an action is in flight (Red: tests/test_planning_feedback.py::test_snapshot_waits_for_a_running_action).


### Windows workspace corrections / v0.1.8 — DD-95–DD-98
- [x] Plan: R-87–R-90 -> BG-42–BG-45 -> SF-74–SF-77 -> UC-77–UC-80 -> FR-72–FR-75 -> QAS-83–QAS-86 -> DD-95–DD-98.
- [x] Task / Red: C:\ root became C:, delete_workspace was unknown, asynchronous creation helper and corrected MCP labels were absent.
- [x] Task / Green: display-normalized paths, reliable creation, safe archived deletion, generic MCP registration.
- [x] Verify / local: 346 passed, 0 failed, 14 ignored (includes one source-layout check); final focused registry/project/picker/layout tests pass; Python 47, fmt and Clippy pass.
- [x] Verify / native macOS: actual Enter creation, original-workspace return, exact-name/path deletion confirmation, recovery evidence and MCP registration screen verified; existing health goal/reviewed plan preserved.
- [x] Verify / delivery: final source c94bea8; CI 38215024341 passes all four jobs. macOS 348/0/14 and Windows 169/0/1 passed/failed/ignored, plus source-layout checks; Python 48 per platform; four exact-package workflow cases per platform pass.
- [x] Complete: English README/manual/wiki shipped; stable v0.1.8 published with macOS/Windows archives and server-verified SHA256SUMS. Exact CI macOS binary reopened the original workspace; reviewed plan retained.

DD-96 native verification correction: actual Create was refused by historical terminal-success startup records whose PIDs were reused by an unrelated system process (29654, mediaanalysisd). Do not kill that process. Successful reaped startup records no longer gate switching; running/failed/unknown records retain quiescence checks. Add a regression with the current live PID as historical terminal evidence. Keep the creation dialog and title visible until the native command succeeds; report failures inside it and explicitly wire confirmation actions.

### Profile-save correction — DD-99 / v0.1.8 amendment
- [x] Plan: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
- [x] Task / Red: awaited save helper and visible in-dialog error contract absent.
- [x] Task / Green: await persistence, close on success, retain errors and guard duplicate/busy submissions.
- [x] Verify / local: Python 48 passed; persisted-profile, concurrent metadata registration, source layout and Clippy pass. Native Claude -> Codex fallback saves and closes; cyclic Codex -> Claude policy stays open with inline error. Original health workspace unchanged.
- [x] Verify / platforms: final-source CI 38215024341 and both native packages pass; all delivered assets pin c94bea8.
- [x] Complete: profile save correction ships in stable v0.1.8; superseded e31b010/e71e889 packages were not published.


### Merged Windows quota correction — PR #1 / v0.1.8
- [x] Plan: clarify R-86, UC-76/UC-16 and DD-94/DD-26 for existing quota behavior.
- [x] Task: merge 466169b; preserve project preflight, independent model windows and precise Codex authentication diagnostics.
- [x] Verify / focused: Claude usage and source layout pass; extend actual badge-function fixtures for named windows and prerequisite tooltips.
- [x] Verify / delivery: CI 38215024341 passes validation and packaging on both platforms; actual macOS Claude /usage test passes with zero generation and reaped ownership; native fresh Claude 100%/91% and Codex 48%/49% observed.
- [x] Complete: updated manual/notes included in stable v0.1.8; prior owned QA app stopped, exact CI native app restarted, original health workspace and plan digest 40f0fb859aa7fce1d998fbf24f357928b4789e711b2b77c27012c3d75490cfee preserved. Shared memory handoff recorded.

Delivery evidence: [v0.1.8](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.8), [CI 38215024341](https://github.com/neodamul/AgentConductor/actions/runs/38215024341), source c94bea8b4fb4b2aa37031c3e07089073bd47eaa9. macOS archive SHA256 bad200ddb1ddaf8e73c9ee146c8b34dc6a538644982daa20411d2290c80167c0; Windows archive SHA256 75fc33222f10961e47291a8eaf7118b95c1e509dbbe2701cd351e57508f928db. MIT root license, pinned dependency license texts/notices and SBOM included (42 macOS / 108 Windows components); no new runtime dependency. Live Windows GUI/provider authentication acceptance remains separate from CI and the operator-supplied Windows usage fixture.


### Session communication knowledge asset — DD-08 / DD-29 clarification
- [x] Plan: distinguish CLI interfaces, communication, authoritative state and reviewed knowledge; retain optional agentmemory.
- [x] Task: source observation, maintained wiki/index/log, design clarification and public README explanation.
- [x] Verify: public-docs contract tests 3 passed; UTF-8, local source hashes, README/wiki links and diff checks pass.
- [x] Complete: source-linked knowledge asset and public README documented; shared agentmemory record saved; documentation delivery is separate from pending runtime implementation. v0.1.9 remains owned by the other Claude machine.

The published v0.1.10 predates the selectable adapter; its post-release implementation and acceptance are tracked below.


### Selectable signal transport with Rust fallback — DD-100
- [x] Plan: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100; requirements and acceptance criteria recorded.
- [x] Task / Red: tests/signal_transport.rs (default/persisted preference, Local never contacts a healthy server, healthy publish once and idempotent repeat, refused server falls back with cause and the same pollable directive, silent server settles locally within 10 s, briefing route) and tests/test_signal_settings_ui.py failed: no route module or Settings controls. Stale recipient, expiry, restart and duplicate/late acknowledgment were already covered by tests/cooperative.rs and tests/cooperative/memory.rs (exact-ID, idempotent acknowledge); they apply unchanged because the directive identity never changes on fallback.
- [x] Task / Green: cooperative::route (preference file per workspace, record-once signal_route per directive, one external operation per workspace via kernel lease and durable pre-dispatch attempt claim, existing 8 s memory client), checkpoint-issue routing, briefing signal_route, Settings transport/port/last route with Korean strings.
- [x] Verify / local regression: 368 Rust tests passed, 14 external/native-account tests ignored; 51 Python contracts passed; formatting and warning-free Clippy passed. Restart uncertainty and competing-owner regressions pass; original cooperative identity/approval tests remain green.
- [x] Verify / native Settings: isolated macOS workspace selected agentmemory, saved, closed and reopened the dialog; exact preference persisted. QA app stopped and the published v0.1.10 app restored.
- [x] Verify / platforms: [CI 38245260274](https://github.com/neodamul/AgentConductor/actions/runs/38245260274), source 3b0c1f690ca939afe56576de887aae42e16987b5, passed macOS/Windows tests, formatting, Clippy and exact-binary package smoke. Windows logs confirm all eight signal transport tests passed, including restart uncertainty and competing-owner cases.
- [ ] Verify / operator: actual company cooperative CLI recipient, Windows desktop Settings and remote-route absence acceptance.
- [ ] Complete: operator manual/UI documentation, runtime/wiki evidence and reviewed release.

The adapter is implemented on main after v0.1.10. It is not included in that published binary; remaining real-recipient/native acceptance stays unchecked. Review added restart-uncertainty and competing-owner tests; a transport receipt is not an execution acknowledgment.


### Compact app version — DD-101
- [x] Plan: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101.
- [x] Task / Red: generated document had no build-version label (zero matching labels).
- [x] Task / Green: both titlebars use a compile-time version placeholder and compact shared styling.
- [x] Verify / local: titlebar version, Room UI and source layout pass.
- [x] Complete: native runtime verification, shared memory and stable v0.1.10 delivery.

### Disposable cache cleanup — DD-102
- [x] Plan: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102.
- [x] Task / Red/Green: bounded lifecycle pruning and confirmed Settings cache preview/cleanup.
- [x] Verify: repeated refreshes, active/referenced/unknown protection, aliases, changed previews and platform contracts; native macOS cache preview observed, permanent removal verified with disposable fixtures.
- [x] Complete: manual/wiki, memory and stable v0.1.10 delivery.

### Abandon blocked planning workspace — DD-103
- [x] Plan: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103.
- [x] Task / Red/Green: stop-marker planning cancellation and same-window current-workspace deletion.
- [x] Verify: blocked/failed planning, owned quiescence, metadata settlement, refusal retention and project protection; native macOS stop-and-delete completed.
- [x] Complete: manual/wiki, native macOS UI, memory and stable v0.1.10 delivery.

Verification checkpoint (2026-10-12, integrated v0.1.10 source): 360 Rust tests passed, 14 external/native-account tests ignored; 48 Python tests passed; formatting and warning-free Clippy passed. Native macOS observed the version label and read-only cache preview. An independent waiting CLI was stopped through the current-workspace Delete action, its snapshot removed and app data archived, and a fresh workspace opened in the same window; project sources were preserved. At that checkpoint platform CI/release was pending; completed delivery evidence follows. Windows desktop interaction remains operator acceptance.

Delivery: [stable v0.1.10](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.10), source e445e2e221449e88f34a57346ba07025ecf0cabe; [Release CI 38233701284](https://github.com/neodamul/AgentConductor/actions/runs/38233701284) passed both platform validation/package smoke and publication. Downloaded packages passed CRC, binary/archive SHA256, exact source/manual/README, bundled icons, MIT root license, dependency notices/texts and SBOM checks (42 macOS / 108 Windows components). macOS ZIP SHA256 92c35f88b56797cc1138e6c3ff7b0602f8bcded68e99701f880b9c6aaa3b4c02; Windows ZIP SHA256 2ed98ee89a23a9d66bad4209e3d720a8b255c29dfbca74fdf07b85d9b8c1952f. User-authorized removal of 13 Releases strictly below v0.1.8 completed after this audit; all Git tags/history retained. Native Windows desktop interaction remains operator acceptance.

### Signal transport review correction — post-v0.1.10
- [x] Plan: review Claude 196e55c against restart/idempotency and bounded-operation requirements.
- [x] Task / Red: simulated dispatch-before-receipt crash republished the directive.
- [x] Task / Green: durable pre-dispatch claim plus kernel-owned workspace lease; no age-based lock stealing.
- [x] Verify: local full regression, isolated native macOS Settings persistence and macOS/Windows CI/package smoke completed.
- [x] Complete: implementation pushed as 3b0c1f6; source-linked wiki and progress evidence updated; shared memory/Claude handoff. Published v0.1.10 tag remains e445e2e; this adapter and correction are source for the subsequent release. Real company recipient acceptance remains separately pending.

### Delivery closure — 2026-10-13 (user decision)

- [x] Complete: v0.1.3 is the delivered binary (macOS aarch64 and Windows x86_64, CI-verified, SHA256SUMS).
- [ ] Follow-up (operator environment): open the Windows package on a real Windows PC.
- [ ] Follow-up (operator account): send a test message through a real Slack or company webhook and answer one tool request from the messenger.

### Agents work inside the workspace sandbox; the coordinator decides beyond it — DD-104
- [x] Plan: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Usage query accepts the Claude folder-trust prompt — DD-105
- [x] Plan: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Planning start and workspace naming feedback — DD-106
- [x] Plan: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Run and task status at a glance — DD-107
- [x] Plan: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Goal folders directly in the project folder — DD-108
- [x] Plan: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Agent CLI list is a master list — DD-109
- [x] Plan: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Reorder agent cards — DD-110
- [x] Plan: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Readable file paths everywhere — DD-111
- [x] Plan: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Activity tab reads like the CLI — DD-112
- [x] Plan: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Coordinator final step in the task list — DD-113
- [x] Plan: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Decide a live tool request from its window — DD-114
- [x] Plan: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Coordinator decides routine requests and records each decision — DD-115
- [x] Plan: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Task list without account usage — DD-116
- [x] Plan: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### The coordinator is selectable and shows its work; one name — DD-117
- [x] Plan: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Failure cause and the next action on a failed run — DD-118
- [x] Plan: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### The next step is always shown — DD-119
- [x] Plan: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Plan review and run completion are organised by phase — DD-121
- [x] Plan: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### A blocking verification finding fails integration verification — DD-120
- [x] Plan: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### The run's phase beside the workspace name — DD-122
- [x] Plan: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Readable Agent Room text and agent icons sized to their names — DD-123
- [x] Plan: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Task cards in Review the plan fold to one line — DD-124
- [x] Plan: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Review the whole phased plan up front — DD-125
- [x] Plan: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Operator refinements — 2026-10-13 (R-112, R-116)
- [x] Goals & progress: each phase carries a tag (Completed, Next/In verification, Planned) and the next phase is bold.
- [x] Review the plan heading wraps instead of breaking Korean text into single characters; phase numbers read n/m.
- [x] Fold arrows for tasks and phases are 22 px round controls instead of a small triangle.
- [x] Opening a workspace routes to the work (DD-126, operator request 2026-10-13): an unfinished run opens the Agent Room; a plan ready for review, or planning in progress, scrolls to Review the plan. The startup snapshot carries the last run state.

### A run survives closing or losing the desktop app — DD-127
- [x] Plan: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Interrupted runs recover and resume automatically — DD-128
- [x] Plan: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Interruptions and silent workers are visible — DD-129
- [x] Plan: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.
- [ ] Activity: consecutive thinking_tokens events collapse into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows.
- [ ] Running task with no output for 5+ minutes shows a No output for n minutes warning; a task whose process is gone shows Not responding — Recover.

### Reassign tasks that have not started while the run is working — DD-130
- [x] Plan: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Agents run with the highest permission by default — DD-131
- [x] Plan: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131.
- [x] Task / Red/Green: full_access templates, derive_permission, validate_permissions and validate_profiles, Codex full-access resume, read-only coordinator mapping.
- [ ] Verify: Windows local tests and a native run whose verifier opens the UI with Playwright.
- [ ] Complete: release.

### The coordinator resolves verification failures — DD-132
- [x] Plan: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132.
- [x] Task / Red/Green: workflow::repair wired after run/resume commands; review_as with the recorded coordinator; tests/repair.rs (Unix fixture).
- [ ] Verify: Windows local tests and the stuck ladder run resumed through Resume.
- [ ] Complete: release.

### Selected-folder execution without project Git metadata — DD-133
- [x] Plan: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133.
- [x] Task / Red/Green: external Git binding, recorded baseline, direct cwd, Accept/restore Reject and owned-file warning.
- [x] Verify: focused Windows regressions for external history, baseline restore and edit warnings; existing Git/private mode; installed Codex initial/resume CLI parsing; formatting, Clippy and 62 Python contracts.
- [ ] Native follow-up: Claude rebuild/relaunch and smoke-check direct outputs before removing six legacy snapshots. Ladder phases 1–4 are already completed under the previous flow (Claude signal, 2026-10-13).
- [ ] Complete: release.

### Deleting workspaces needs no replacement name — DD-133
- [x] Plan: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133.
- [x] Task / Red/Green: delete_workspace opens the most recently used other workspace; dialog name field removed; tests/workspace_registry.rs deletes until none is left.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

## Bilingual release documentation — DD-134 / v0.1.16

- [x] Plan: pull the latest release source, inspect changed behavior and preserve concurrent Claude work.
- [x] Task / Red: reproduce language-dependent error assertions and missing current screenshot references.
- [x] Task: update both manuals and public README for setup tabs, direct folder execution, recovery, repair and permissions.
- [x] Task / Green: capture genuine English/Korean workspace, profile and completed-run screens; validate linked assets.
- [x] Verify: Rust 393 passed / 0 failed / 14 ignored; Python 63 passed; formatting, Clippy, native checks and package documentation coverage passed.
- [x] Complete: record verified native evidence, prepare the bilingual documentation archive and hand off for the user's v0.1.16 packaging. Commit/push identity is recorded in agentmemory.

### The app updates itself from GitHub releases — DD-135
- [x] Plan: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135.
- [x] Task / Red/Green: src/update.rs, src/ui/update.rs, title-bar icon and dialog, menu item, CLI update; tests/update.rs.
- [x] Verify: Windows self-replacement of a running 0.1.15 build with the published v0.1.16 archive; relaunch removes the leftover.
- [ ] Complete: release; macOS bundle replacement verified on a Mac.


### DD-136 Quota resume coordinator (workflow / TDD)

Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

- [x] Plan: trace requirements, durable coordinator design and cancellation boundaries.
- [x] Red: reset parsing; deadline/fallback boundaries; persistence, reassignment, stop and duplicate watcher tests.
- [x] Task: implement only after recording failing tests.
- [x] Verify: focused Rust39, Python65, formatting, all-targets Clippy; independent native coordinator due execution, lease restart, same-account reassignment and Stop cancellation.
- [x] Complete: own-file commit, main integration/push and AgentMemory handoff.

## v0.1.17 manual and README reconciliation — DD-134 / DD-135 / DD-136

- [x] Plan: compare latest code from b6890f5 with both manuals and the public README; use the requested 0.1.17 documentation version.
- [x] Task: document update controls/checksums/idle gates/private-release limits, persistent availability reservations, five-hour fallback rules, full-diff review and Save-to-Team navigation.
- [x] Task: label existing native screenshots as 0.1.16 reference captures; do not fabricate a 0.1.17 screenshot or claim it is published.
- [x] Verify: all 3 public documentation checks and 19 release/packaging checks passed; README remains 119 lines.
- [x] Complete: verified documentation prepared for commit/push; handoff and resulting commit are recorded in agentmemory.

## Latest-source native screenshot refresh — DD-134

- [x] Plan: preserve the operator's reviewed plan and inspect the current source/package version before replacement.
- [x] Task: build the optimized source with two jobs, terminate the previous owned GUI and recapture four actual native screens.
- [x] Task: execute a fresh exact-binary three-worker fixture, verify overlapping tasks, passing tests and refused unapproved merge before recording the completed run.
- [x] Verify: 3 documentation and 19 package checks passed; screenshot fingerprints recorded; reviewed operator plan unchanged; one optimized GUI restored on its original workspace.
- [x] Complete: refreshed assets and verified runtime prepared for commit/push; resulting commit and handoff recorded in agentmemory.

### Selectable themes and session status tags — DD-137

- [x] Plan: R-128 → BG-83 → SF-115 → UC-118 → FR-113/NFR-49 → QAS-124 → DD-137.
- [x] Task/TDD Red: `tests/interface_theme.rs` (4) failed first; `room_task_rows` and `interface_language` updated to the amended decision.
- [x] Task/Green: 144 color literals replaced by 24 semantic tokens (light E / dark F), named-color and alpha-hex leftovers removed, `ok-bg` split from `accent-soft`, theme preference + `set_theme` + Settings select, session status on its own line, `docs/design-guide.md`.
- [x] Verify: full Rust and Python suites, clippy, fmt; native captures of workspace and Agent Room in light, dark and Korean; token pairs ≥ WCAG AA 4.5:1.
- [x] Complete: CHANGELOG (Unreleased), wiki, agentmemory, commit/push; manual text handed to Codex.

### Plans know whether the project uses Git, and verification commands that cannot work are replaced — DD-138
- [x] Plan: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138.
- [x] Task / Red/Green: planner Git line, external Git environment for integration tests, Revision.test_command; tests in tests/folder_workflow.rs.
- [x] Verify: Windows folder_workflow (git diff --check in a plain folder passes; coordinator replaces a failing command).
- [ ] Complete: release.

## Latest GitHub design and build reconciliation — DD-137 / DD-138

- [x] Plan: pull 0211f38, inspect the new design guide and distinguish published v0.1.17 from later source features.
- [x] Task / Red: native dark-theme QA and computed CSS contrast expose an unreadable notification (light contrast 1.21:1).
- [x] Task / Red/Green: disabled primary contrast failed at 1.91:1; existing soft/fill at full opacity now passes in both themes.
- [x] Task / Green: use existing bar/bar-ink and err-bg/err pairs; contrast tests pass for both themes without new tokens or dependencies.
- [x] Task: update README and English/Korean manuals for themes, task-first rows and Git-aware test-command repair.
- [x] Verify: inspect both screens in both themes and Korean; refresh native captures; run design/docs/package checks and confirm the original plan/preferences remain.
- [x] Complete: two verified change units committed/pushed; one optimized app restored on the operator workspace; memory handoff recorded.

## Compact account cards and step explanations — DD-139

- [x] Plan: R-130 → BG-85 → SF-117 → UC-120 → FR-115 → QAS-126 → DD-139.
- [x] Task / Red: exhausted quota test expected percentages only and failed on the appended reset time.
- [x] Task / Green: tooltip-only reset details, compact badge/refresh, reduced card spacing, consistent step introductions and available-width helper text.
- [x] Verify: quota/tabs/theme/docs regression; native light/dark visual checks and updated captures.
- [x] Complete: one updated app, commit/push and agentmemory/Claude handoff.

## Focused Settings and tool guide menu — DD-140

- [x] Plan: R-131 → BG-86 → SF-118 → UC-121 → FR-116 → QAS-127 → DD-140.
- [x] Task / Red: conditional port-visibility runtime test failed before the handler existed.
- [x] Task / Green: readable label/control spacing, conditional port field with retained value, guide moved to workspace menu.
- [x] Verify: Settings/guide regression, both-theme native interaction and document checks.
- [x] Complete: restart one updated app, commit/push and handoff.

### Custom CLIs that approve their own tools can run — DD-141
- [x] Plan: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141.
- [x] Task / Red/Green: operator_auto_approve mode, agent form checkbox, tests/self_approving_cli.rs.
- [x] Verify: Windows tests.
- [ ] Complete: release.

## Stable release v0.1.18

- [x] Plan: include DD-137–DD-141 and validated UI changes; next stable version after v0.1.17.
- [x] Task: repair stale planning-progress class selector using the stable review-panel ID; update package version, changelog and bilingual manuals.
- [ ] Verify: full native/platform regressions, package smoke and macOS/Windows CI.
- [ ] Complete: publish both verified binaries and SHA256SUMS; record release/handoff.

## Scrollable release notes — DD-142 (next release)

- [x] Plan: R-133 → BG-88 → SF-120 → UC-123 → FR-118 → QAS-129 → DD-142; preserve v0.1.18 tag.
- [x] Task / Red: actual-body rendering test failed before the panel existed.
- [x] Task / Green: reuse fetched body, literal text binding and focusable bounded-height scrolling.
- [x] Verify: updater status and UI runtime tests, source layout, fmt/clippy.
- [x] CI follow-up: next-source native review test allows 15 seconds for child approval/merge under runner load, preserving all approval assertions and printing the snapshot on timeout.
- [x] Complete: commit/push for the next version; published/tagged v0.1.18 is unchanged.

### DD-143 MCP workflow / TDD

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.

- [x] Plan: latest main/design guide, requirement trace and read/summary boundaries.
- [x] Red: native oversized response and absent auto-routing API; runtime missing coordinator/server scope cases.
- [x] Implement: server-scoped UI, schema/ID-valid coordinator reads, metadata helpers and full Markdown summaries.
- [x] Verify: native/runtime regressions, Windows portable theme4/language2 and layout checks; actual screenshot capture was unavailable (Computer Use native pipe missing), so no rendered-screen claim.
- [ ] Complete: own-file main integration/push, release verification and Claude signal.


### DD-144 Planning feedback workflow

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.

- [x] Plan / Red: preserve logs and prove the previous failure remains before retry.
- [x] Implement: synchronous reset, job-scoped progress, actual CLI icon, paid-fallback card control removal.
- [x] Verify: runtime job transitions, quota scheduling, translations and semantic-theme checks.
- [ ] Complete: own-file commit/main/push/release and Claude handoff.


### DD-145 Approval and API limit workflow

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.

- [x] Plan / Red: untyped API JSON becomes Unknown; unchanged plan remains dirty.
- [x] Implement: explicit restriction cache and normalized review snapshot comparison.
- [x] Verify: real offline CLI-to-durable availability/five-hour fallback, normalized approval guards, native regressions, formatting, Clippy and Python67.
- [ ] Complete: main/push/release and AgentMemory handoff.


### DD-146 / DD-147 Workflow completion

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

- [x] Plan: scoped transient notices and offline shared menu.
- [x] Red: recoverable events rejected, coordinator rejected first reconnect; manual menu absent; quiet review claimed approval lock.
- [x] Implement: narrowly matched warnings, localized safe manual and HTTP guide; read-only review skips availability lease.
- [ ] Verify / Complete: integrated platform tests, main push, stable package and Claude handoff.

### Workspace and Agent Room feedback refinements — DD-148
- [x] Plan: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148.
- [x] Task / Red/Green: tests/goal_sources_ui.rs, tests/started_phase_label.rs, tests/room_quota_selection.rs, tests/test_quota_badge.py.
- [x] Verify: Windows Rust/Python/Node UI tests; native screenshot of step 1; tag alignment checked in a browser with the real stylesheets.
- [ ] Complete: release.

### Runs keep going and stay visible — DD-149
- [x] Plan: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149.
- [x] Task / Red/Green: tests/transient_capacity.rs, folder_workflow stray test, room_task_rows, busy_button_runtime.cjs + test_busy_buttons.py, room_routing.rs, startup.rs.
- [x] Verify: Windows Rust/Python/Node tests.
- [ ] Complete: release.


### DD-150 / DD-151 Workflow

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog

- [x] Plan: signal settings govern instructions; replace the per-click Windows helper.
- [x] Red: actual planner/worker prompts lack selected backend, picker starts PowerShell.
- [x] Implement: host-owned routing contract, optional-tool wording and direct COM picker.
- [x] Verify: planner/worker preference regressions, local/remote-failure/idempotence transport checks, Unicode/native console tests, language/theme/docs/layout checks, Python70, fmt and all-target Clippy.
- [x] Complete: own commit/main/push, verified v0.1.21 release and AgentMemory/Claude handoff.


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

## v0.1.22 documentation and offline HTML — DD-153

- [x] Plan: R-144 → BG-99 → SF-131 → UC-134 → FR-129 → QAS-140 → DD-153; review tagged0.1.21 and new UI workflow.
- [x] Task / Red: HTML rendering regression fails before converter exists.
- [x] Task / Green: semantic generated English/Korean HTML, safe local images/links, lazy native loading and updated operator text.
- [x] Verify: native English/Korean tables, in-dialog locale links and Light/Dark checks; latest screenshots; local Rust448/Python74, fmt/Clippy; both platform CI validations and four packaged workflow modes passed.
- [x] Complete: source19d08999df4b committed/pushed; stable v0.1.22 published; wiki asset and agentmemory/Claude handoff recorded.

Release note: v0.1.21 was published while this documentation update was being verified. Preserve its tag and publish this patch as v0.1.22. Windows release validation previously failed at NO_HEADING; constrain automation to the launched process and retain rendered-control assertions.

Release evidence: [successful Action38549103869](https://github.com/neodamul/AgentConductor/actions/runs/38549103869) and [stable v0.1.22](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.22). Downloaded both published archives; verified archive SHA256SUMS and every internal file hash, source commit and English/Korean semantic HTML contents (platform newlines normalized for text comparison). Formerly failing Windows WebView2 UIAutomation saved the Unicode goal successfully. Existing operator workspace was reopened without running its prepared plan; original Korean/Light preference restored. Physical Windows manual interaction and automatic updater testing remain outside this documentation/release verification; updater testing was previously assigned to Claude.


### v0.1.21 verification and next-version request

Release: https://github.com/neodamul/AgentConductor/releases/tag/v0.1.21; frozen source a0bdb7b3424aff908608c77b579bb4931b0e4340. Native Windows and macOS Rust/Python71/formatting/Clippy validation and exact release-binary packaging passed: https://github.com/neodamul/AgentConductor/actions/runs/38545630595. The tag workflow first timed out finding the Windows UI Automation heading. Its rerun passed that UI test but separate PowerShell containment fixtures timed out. The identical-source main validation and both exact native package smoke jobs fully passed; their unchanged, source/binary/archive-hash-verified artifacts were published to the existing tag. No tests were skipped and no tag or source was rewritten. Both published ZIPs were downloaded and checked against SHA256SUMS and GitHub asset digests.

DD-152: adjacent workspace export/import, localized labels and the unchanged native request routing passed runtime Python and native room-routing/layout/language checks; main integration and v0.1.21 publication completed.

Next version (requested by the user): preserve direct planning-rule editing and allow a selected folder to supply rules by recursively reading its Markdown (.md) files. Keep this separate from the frozen v0.1.21 release. Implement with requirement-to-design trace, design-guide styling, localized message IDs and TDD; show the applied source files and deterministic ordering.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

- [x] Plan / Red: saved-folder cooldown does not reset; planning invocation has no schema contract.
- [x] Implement: path-aware Claude scheduling and native CLI structured response flags.
- [x] Verify: 16 targeted Rust regressions including the schema-bound phased planner subprocess, Python75 runtime/contracts, source bounds, formatting and all-target Clippy. No authenticated onsite Enterprise account was available; its response is not claimed as verified.
- [x] Complete: own-file commit/main integration/push; local app restart and AgentMemory/Claude handoff recorded in the task completion memory.

CLI references: https://code.claude.com/docs/en/cli-reference ; https://developers.openai.com/codex/cli/reference

## Latest-source manuals and reusable release skill — v0.1.23

- [x] Plan: source b329745; document R-145/DD-154 saved-project Claude quota scheduling and R-146/DD-155 schema-bound planning. Preserve pending planning-folder requirements as unimplemented.
- [x] Task: refresh public README and both Markdown/HTML manuals; add discoverable agentconductor-release skill and portable repository copy using existing packaging scripts.
- [x] Verify: Rust452/Python75, docs/renderer/layout, fmt/Clippy and skill validation passed; native EN/KO new text checked; both platform Release tests and four packaged workflow modes passed.
- [x] Complete: stable v0.1.23 source9f2da2a published; downloaded archive and all internal hashes plus four manual files checked against the frozen tag; skill registered and portable copy committed.

Release evidence: [Action38553599059](https://github.com/neodamul/AgentConductor/actions/runs/38553599059), [stable v0.1.23](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.23). Automatic updater and physical Windows Enterprise metadata were not tested in this documentation release. Native layout was unchanged, so genuine existing screenshots were retained.

## GitHub #4 — DD-156

- [x] Plan: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156; completed room vs active execution.
- [x] Red: completed Room creation failed with return-to-workspace message; running report remained attached.
- [x] Green: check actual run/task activity and Controller::busy; detach only after successful transition.
- [x] Verify: completed-room/active-run regression, workspace registry/navigation, room routing, language/source bounds, docs and all-target Clippy passed locally; platform CI pending.
- [x] Complete: issue4 commit0523939 pushed; macOS/Windows validation passed in Action38582625952; GitHub #4 closed with evidence.

## GitHub #5 — DD-157

- [x] Plan: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157; reproduce prompt readiness independently of authenticated provider availability.
- [x] Red: native PTY timed out on ❯; actual spent/allocated-only format failed parsing.
- [x] Green: shared prompt/trust readiness, exact allocation subtraction, stage-specific timeouts and no invented date-only timestamp.
- [x] Verify: native PTY trust-choice > prompt → Enterprise allocation fixture, three prompt variants, ownership cleanup, existing usage/budget/time tests, Rust26/Python75 and Clippy passed locally; Windows CI pending.
- [ ] Complete: issue-specific commit/push; onsite Enterprise evidence before claiming the original report resolved.

- [ ] Complete: issue-specific commit/push and GitHub issue result.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

- [x] Plan / Red: actual blocked planner cannot create another workspace; stale workspace/closed-room replies replace the current view.
- [x] Implement: parked active sessions, target-first ownership, serialized navigation, deletion protection and stale-response guards.
- [x] Verify: actual planner progress/completion through workspace creation and repeated switching; same job restoration, running-room reconnection, target lease refusal, active deletion protection and idle lease cleanup; 19 unique targeted Windows Rust tests, Python77 including late-response regressions, source bounds, formatting and all-target Clippy. No macOS/native screenshot acceptance is claimed.
- [x] Complete: own-file main commit/push; local app restart and AgentMemory/Claude handoff recorded in task completion memory.

## v0.1.24 verified native release

- [x] Plan: freeze latest main f9ad6d9, including R-147/DD-156, R-148/DD-157 and R-149/DD-158; preserve existing published tags and pending planning-rule folder design.
- [x] Task: synchronize Cargo version, README and English/Korean Markdown plus generated offline HTML with implemented active workspace navigation and Enterprise quota collection. Existing screenshots show the unchanged layout; no new UI layout is introduced by this release.
- [x] Verify: full locked Rust/Python/formatting/Clippy checks and both native exact-package smoke jobs; download published archives and verify raw archive/internal hashes, tagged source and bilingual manuals.
- [x] Complete: stable v0.1.24 Release Action/publication, separate completion record, main synchronization and AgentMemory/Claude handoff.

Authenticated onsite Enterprise behavior, physical macOS interaction and automatic updater installation are not claimed by fixture or CI validation.

Release evidence: [all five successful Release jobs](https://github.com/neodamul/AgentConductor/actions/runs/38592812353), [stable v0.1.24](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.24), and [downloaded artifact verification](evidence/v0.1.24-release.json). Frozen source aa939ba6fc111bac28331b389fb51199df0e7aab includes the incoming71145c5 deterministic MCP SSE fixture. Both published archives match archive/internal raw SHA256SUMS and GitHub asset digests; README, Cargo.lock, bilingual Markdown/HTML and seven screenshots per platform match the frozen tag. Text comparison alone normalizes platform line endings. Native package receipts verify shared/worktree × Git/folder, three tasks and tests per case, refusal before approval and verified approved integration. No test skip, gate bypass or tag rewrite.

Local Python77, formatting and all-target Clippy passed. Extra local all-target native validation initially exhausted disk space; Codex incremental build cache alone was removed after realpath containment checks. Its retry hit an existing mutual_fallback60s deadline under concurrent subprocess load; failure logs are retained, and the exact-source CI Windows/macOS full suites passed. The full retry finished with284 passed,1 timed-out and1 existing ignored test; the unchanged timed-out assertion then passed alone in33.20s, and the final-source MCP SSE test passed in0.42s. Do not retroactively describe the first full retry as all green. Claude received the verified release handoff; recursive planning-rule folder support remains queued for a later version.
## GitHub #6 — DD-159

- [x] Plan: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159; one folder owner, multiple independent goal contexts and preserved execution history.
- [x] Red: real phased fixture run/approval/merge completed; new_goal request was missing.
- [x] Green: new/select goal, optional goal-run links, imported-link clearing, nonterminal/busy protection, project ownership lease and visible native controls.
- [ ] Verify: real completed-run/goal regression, UI runtime, sharing/import compatibility, language/theme/layout and native screenshots.
- [ ] Complete: issue-specific commit/push, platform CI and GitHub resolution; patch release through the registered skill.

## Rust-only application and build migration — DD-160

- [x] Plan: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160; keep installed UI/runtime, release packaging, manual generation and owned verification in Rust; preserve external CLI integration.
- [x] Task 1: inventory the runtime and release path; select the Rust binary as the offline manual-render command without adding a server or dependency.
- [x] Red / Green: Rust tests first failed without a renderer; implemented safe Markdown tables, links, code blocks and offline PNG assets in `src/manual.rs`.
- [x] Verify Task 1: generated English/Korean HTML matches committed files exactly; sanitizer tests, source-layout check and all-target Clippy pass.
- [x] Task 1 cleanup: removed the superseded Python manual renderer and its three Python-only tests; Rust tests retain generation, escaping, local-image and escaped-table-pipe coverage.
- [x] Verify Task 1 cleanup: focused Rust renderer tests, source layout, formatting and all-target Clippy pass; the three legacy renderer contracts passed before removal.
- [x] Task 3 / release notes: Rust unit tests cover exact extraction, missing/empty sections, UTF-8 and the release workflow; removed `scripts/release_notes.py` and its Python test.
- [x] Verify release notes: CLI output matches the former script byte-for-byte for v0.1.24; focused tests, formatting and Clippy pass.
- [x] Task 3: moved macOS Finder bundle assembly to Rust, preserving the executable/icon layout, plist metadata, executable mode and overwrite protection; removed `scripts/macos_bundle.py`.
- [x] Task 3: moved ZIP archive creation to Rust; retain relative package paths, DEFLATE compression, executable metadata, DOS date bounds, symlink rejection and partial-file cleanup; removed `scripts/release_archive.py`.
- [x] Verify Task 3 locally: macOS bundle and archive tests, full Rust suite, remaining Python contracts, formatting, Clippy and license-text review for both native target dependency sets pass.
- [x] Task 3: moved packaged-binary manifest, SHA-256 and checksum-file validation to Rust before execution; removed the Python validator and its duplicate tests.
- [x] Verify package preflight locally: tests cover valid packages, corrupt hashes, unsafe names, malformed commits, checksum mismatches, symlinks and the hidden CLI response.
- [x] Task 3: moved packaged-workflow acceptance into the Rust CLI; verify shared/worktree execution on Git repositories and plain folders, parallel scoped tasks, tests, diffs and the approval-gated merge; removed the Python smoke runner.
- [x] Verify Rust smoke migration locally: red-before-command and green integration test; all Rust targets (478 passed, 14 ignored), release-mode smoke suite (6 passed), Python contracts (66 passed), formatting and Clippy pass.
- [x] Fix Windows ZIP timestamp fixture: CI on 90df88e showed epoch setup was Unix-only while the 1980 clamp assertion ran on every OS; set and verify epoch before archive creation on all platforms.
- [x] Verify ZIP archive fixture locally: all 3 archive integration tests pass on macOS.
- [x] Verify ZIP timestamp fixture on exact-source Windows CI: `Execute applicable native tests` passed on commit a4a5fe0 (run 38730449653); the full Windows job later failed Clippy on the Unix-only permissions binding, with package jobs skipped.
- [ ] Verify this acceptance path in exact-source macOS and Windows package CI.
- [ ] Verify Task 3: validate generated plist and Rust archive command in exact-source macOS and Windows package CI.
### Task 3.1 — Native package assembly in Rust

- [x] Plan: preserve exact target binary and source commit, dependency closure, SPDX allowlist, reviewed upstream license provenance/digests, bundled SQLite notice, referenced README assets, macOS bundle, SBOM, license notices and checksums; refuse overwrite and validate before output.
- [x] Red: Rust package contract tests first failed with the unimplemented assembler; coverage now includes both targets, upstream revision/version/digest, SQLite notice, unsupported licenses, missing files and staging cleanup.
- [x] Green: implemented the Rust package/license modules and `package-release` CLI; the native-package CI step now invokes the Rust binary without adding dependencies.
- [x] Verify core contracts: 6 Rust package tests; full Rust suite 484 passed / 14 ignored across 163 suites; all-target Clippy and formatting pass locally; authored Rust source/test files remain below 500 lines.
- [x] Complete source migration: remove the three Python release packagers and duplicate Python package test; update the registered release skill, platform workflow, and LLM wiki.
- [x] Fix Windows Clippy regression: cfg(unix) permission metadata now uses a shadow binding; macOS Clippy passes locally.
- [x] Verify the actual native package-release command on macOS commit 46c0e4e: exact commit and binary hashes, 58 dependency notices, ZIP checksum/CRC, and all four smoke modes (Git/folder × shared/worktree), including tests, diff review, approval refusal and approved merge.
- [x] Verify exact-source macOS/Windows validation and package artifacts in GitHub CI: run 38733217698 passed all four jobs (native validation and exact-package smoke on macOS and Windows) for commit 46c0e4e; downloaded ZIPs verify with `ARCHIVE-SHA256SUMS` (macOS `f562ef5e6ae7be8aee62284300f65c7ef1c26bfe39dcd4db73cc8db63cda0d20`, Windows `e3a6b964c3de6d9b10a15526ca069de7c08f384cbec737e8d77c26beef68de3b`).
- [x] Complete: both native runners built the clean-commit binary, rendered bilingual manuals, ran package-release and approval-gated workflow smoke, and retained package/evidence artifacts.

- [ ] Task 2: measure idle resources; port the JavaScript-driven desktop UI to Rust while preserving keyboard/accessibility, bilingual behavior and cross-platform screens.
- [x] Task 2.1 / TDD: move Jira issue-key and labeled Confluence page-reference suggestion parsing and explicit-reference span extraction from JavaScript to a bounded Rust parser exposed through the existing native request channel; add integration and UI runtime regression coverage.
- [x] Verify Task 2.1: final UTF-16 span-based UI passes the complete offline Rust all-target suite, 59 Python contract tests, MCP UI runtime, source-layout limit, formatting and all-target Clippy; 14 environment-dependent Rust tests remain ignored.
- [ ] Task 2 remaining: migrate the rest of the interactive UI behavior and remove project-owned JavaScript runtime logic while preserving one-window, low-resource native desktop behavior.
- [x] Task 2.2 / TDD: replaced duplicate JavaScript quota badge formatters with Rust-localized snapshot data for startup agent cards; retained compact Enterprise percentages, window sorting, reset details, and stale/unavailable output.
- [x] Verify Task 2.2: all-target Rust regression (499 passed, 14 ignored), all 44 Python contracts, focused Clippy, formatting and diff checks pass; the slow repository-wide source-layout test was filtered after blocking, and changed Rust files were checked directly for the 500-line/test-separation rules.
- [x] Task 2.3 / Plan: move per-agent quota refresh interval decisions into Rust snapshots, retaining the current 1–60 minute setting (5-minute fallback), async metadata queue, and immediate retry when Claude's project binding changes.
- [x] Task 2.3 / Red: Rust boundary tests first failed because the native refresh policy was absent; startup snapshot tests and source-contract tests cover both consumers.
- [x] Task 2.3 / Green: added one Rust interval policy and per-profile snapshot map for startup and Agent Room; removed the duplicate JavaScript due predicates.
- [x] Task 2.3 / Verify: Rust all-target regression (502 passed, 14 ignored, 1 slow source-layout test filtered), all 44 Python contracts, quota scheduling/project-change contracts, formatting, all-target Clippy and diff checks pass; cooldown advances only after an accepted startup refresh, and rejected refreshes remain retryable.
- [x] Task 2.4 / Plan: move quota window labels from Agent Room JavaScript into the existing Rust quota display layer and pass localized labels through the native snapshot; preserve provider order and bilingual text.
- [x] Task 2.4 / Red/Green: Rust tests first failed because the room label map was absent; added it to snapshots and removed the duplicate/unused JavaScript formatters.
- [x] Task 2.4 / Verify locally: Rust all-target regression 505 passed, 14 ignored, 1 slow source-layout scan filtered; all 43 Python contracts, focused native-room/quota tests, formatting, Clippy, diff and changed-file line-limit checks pass.
- [x] Task 2.4 / Platform CI diagnosis: exact-source Windows CI passed on `b5a5cd0`; macOS CI exposed a parallel test-fixture collision in `tests/macos_bundle.rs` (timestamp-only temporary directory names).
- [x] Task 2.4 / TDD correction: replace timestamp-only fixture naming with collision-safe process/counter IDs and add a distinct-directory regression; focused macOS bundle tests pass locally (3 passed).
- [x] Task 2.4 / Verify platform CI: run 38756510671 passed macOS and Windows validation; Windows native package smoke passed. Its queued macOS package job was superseded by the next source push.
- [ ] Task 2.4 / Native interaction/accessibility: gather platform evidence; non-quota restriction/availability date formatting remains JavaScript until its Rust UI port.
- [x] Task 2.5 / Plan: move Enterprise budget-row labels and percentage/amount selection into Rust quota snapshots; preserve stale, unlimited, bilingual, and reset data behavior.
- [x] Task 2.5 / Red/Green: added Rust tests before the snapshot API existed; implemented shared remaining/allocated percentage logic, per-agent row data, and removed the duplicate JavaScript calculations.
- [x] Task 2.5 / Verify locally: quota display tests (7), native room snapshot test (1), formatting, all-target Clippy, diff check, and changed Rust file limits pass.
- [x] Task 2.5 / Verify platform: the corrected Rust snapshot contract and budget UI passed exact-source macOS/Windows validation and both native-package smoke jobs in run 38763436037. Native display/accessibility evidence remains open under Task 2.4.
- [x] Task 2.6 / Plan: move Agent Room quota window and Enterprise budget reset-time formatting to Rust snapshots; preserve local timezone and English/Korean reset labels, and keep missing resets hidden.
- [x] Task 2.6 / Red/Green: add reset-label/localization/null and UI snapshot contract coverage first; the new test failed before `window_reset_labels` existed, then passed after Rust snapshot formatting replaced JavaScript `Date` formatting for quota rows.
- [x] Task 2.6 / Verify locally: quota display and native-room suites pass (9 tests); all-target Clippy, formatting and diff checks pass; changed code/test files remain below 500 lines.
- [x] Task 2.6 / Repair stale cross-platform assertion: replace the obsolete JavaScript-share test contract with the Rust snapshot contract; the quota display, room quota selection, and native-room suites pass (11 tests).
- [x] Task 2.6 / Verify platform: exact-source macOS/Windows validation and both package/execute smoke jobs passed in run 38763436037 for `0faefc1`; Rust-generated English/Korean manuals and exact release binaries were exercised.
- [ ] Task 2.6 / Native display/accessibility: inspect both platform UIs, reset labels, screen-reader names, keyboard navigation and Korean IME behavior.
- [x] Task 2.7 / Plan: add DD-161 native Rust UI prototype gate; evaluate minimal eframe/egui (MIT OR Apache-2.0, AccessKit) against existing Wry resource budget, keyboard, bilingual, accessibility, and platform requirements before migrating production screens. No dependency added before prototype evidence.
- [x] Task 2.7 / Prototype TDD: build an isolated eframe 0.37.0 native window with English/Korean text, a goal input, and agent-state controls; an AccessKit-tree test first exposed the missing goal-field name, then passed after adding an explicit accessible name.
- [x] Task 2.7 / macOS build: offline release build and accessibility test pass; removing bundled default fonts avoids their OFL/Ubuntu font license expression, and the system-font build is 7.7 MiB with 290 normal dependency-tree entries.
- [x] Task 2.7 / macOS idle resources: after 65 seconds, `footprint` reported 47 MB physical footprint and `ps` reported 0.0% CPU; launch peak was 156 MB. Follow NFR-05 and the existing WebKit baseline by comparing physical footprint, not RSS (shared framework pages inflate RSS).
- [x] Task 2.7 / macOS interaction: packaged native window exposed the named goal text field and checkbox in AccessKit; Tab moved focus to the checkbox, Space toggled it, and Korean text rendered in the goal field.
- [x] Task 2.7 / License TDD: package-provenance test first failed on unreviewed BSL-1.0; pin exact upstream BSL-1.0 and ISC texts by source commit and SHA-256, allow these permissive SPDX licenses, and verify both notices are assembled into the Windows package. License tests pass; this does not add eframe to production.
- [x] Task 2.7 / Repository probe TDD: promote the isolated AccessKit and `Room::dispatch` tests into an opt-in `native-ui-probe` feature and separate binary. The first build failed on the absent probe module, then exposed the eframe 0.37 `App::ui` API and test texture cleanup contract; after fixing those, both accessibility and direct-dispatch tests pass. Default app/package targets do not enable eframe.
- [x] Task 2.7 / macOS candidate CI: exact-source run 38777444017 for 64446d4 passed native tests, optional probe tests/build/Clippy, package and embedded-UI checks, formatting, and lint.
- [x] Task 2.7 / Windows candidate CI: exact-source run 38777444017 passed the native build, source layout, full native tests, optional probe tests/build/Clippy, package contracts, formatting and lint. No release dependency is enabled.
- [x] Task 2.7 / TDD fix for optional binary selection: exact package step failed on macOS because Cargo could not choose between the production CLI and the feature-gated probe (`cargo run --release --locked -- manual render`). Setting `default-run = "agentconductor"` made the same offline release command render both manuals successfully.
- [x] Task 2.7 / Verify fixed package workflow: exact-source run 38780554955 on 7a1b12f passed macOS and Windows validation, optional native UI tests/build/Clippy, Rust manual rendering, optimized release build, and execution of each platform's exact packaged binary. This verifies the default package excludes the optional eframe probe.
- [x] Task 2.7 / Current-main confirmation: run 38784456587 on 1639d1c also passed macOS and Windows validation and packaged-binary smoke checks; the candidate's source and release packaging are still limited to the Rust CLI with WebView-based UI.
- [x] Task 2.8 / Goal-save exact-source verification: run 38788589621 on `ed273bf` passed macOS and Windows native tests, optional eframe probe tests/build/Clippy, source-layout and packaging contracts, formatting, all-target Clippy, Rust manual generation, and both release-package execution smoke checks.
- [ ] Task 2.7 / Remaining gates: physical Korean IME composition is untested and the skeletal prototype does not yet meet the product's visual/responsive layout; candidate-package license/resource comparison remains unverified. The workflow's current default-app package jobs do not contain eframe.
- [ ] Task 2.7 / Decision: accept eframe only if every DD-161 gate passes; otherwise evaluate another native Rust UI implementation without keeping JavaScript/WebView as the final UI.
- [x] Task 2.8 / Integration architecture: inspected the production entry points and verified `Room::dispatch` is the Rust request boundary for workspace and Agent Room operations. A native shell can reuse `Room`, `Startup`, `Controller`, approval checks and leases directly, without adding a local server or retaining WebView IPC.
- [x] Task 2.8 / Dispatch TDD proof (isolated prototype only): the direct-dispatch snapshot test failed before `RequestWorker` existed, then passed with a bounded request channel and background `Room::dispatch` worker. This proves the narrow request/result path; it is not production integration or screen parity.
- [x] Task 2.8 / Snapshot-render TDD: added a regression test that saves a goal through `Room::dispatch`, requests the Rust snapshot, and verifies the native renderer paints that saved goal; removed the static Claude/Codex preview cards and renders registered profiles from snapshot data. Opt-in probe suite: 3 passed; Clippy with warnings denied and formatting pass. This remains a prototype renderer, not production-screen parity.
- [x] Task 2.8 / Goal-save TDD: the AccessKit contract failed before the accessible Save goal action existed; added a revision-bound request that preserves the rest of the config, sends save through the bounded background worker, then fetches a fresh snapshot only after `{ "saved": true }`. The probe keeps controls disabled while saving and shows errors. The async round-trip test verifies the persisted goal; probe suite 4 passed, Clippy and formatting pass. Production integration remains pending.
- [x] Task 2.8 / Workspace identity TDD: extended the opt-in native probe's form to edit workspace name and goal together; accessible field/action contracts and the `Room::dispatch` save-refresh round trip verify both values persist. Focused probe suite 4 passed. This is still an isolated prototype slice and does not replace the shipped WebView UI.
- [x] Task 2.8 / Repository path TDD: added the repository path field to the opt-in native workspace form and verified persistence through the async Rust dispatch path. The test asserts the canonicalized macOS path; focused probe suite 4 passed and focused Clippy is clean. This remains an opt-in prototype slice.
- [x] Task 2.8 / Native folder-picker TDD: added an accessible Browse control that opens the existing platform picker on the background worker; selection fills the repository field and cancellation preserves its prior value. Worker response and cancellation-state tests pass; focused suite 6 passed, targeted Clippy is clean, and both code/test files remain under 500 lines. The screen is still an opt-in probe, not the shipped UI.
- [x] Task 2.8 / Workspace isolation TDD: the new save contract first failed to compile because the selected isolation mode was not part of the request; added Shared folder and Git worktree radio controls, snapshot restoration and mode persistence through the existing Rust save handler. The accessible native probe suite passes all 7 tests and focused Clippy is clean; this remains an opt-in probe, not shipped UI parity.
- [x] Task 2.8 / Workspace isolation exact-source verification: run 38804235226 on `8683862` passed native validation and exact release-package execution on macOS and Windows, including the updated native probe tests. The implementation is still probe-only; production UI migration remains open.
- [x] Task 2.8 / Exact-source verification: run 38799761260 on `ec1339c` passed macOS and Windows native validation, optional probe tests/build/Clippy, repository contracts, formatting, all-target Clippy, Rust manual rendering, and both exact release-package smoke checks. The native workspace controls remain in an opt-in probe; production UI parity and migration are open.
- [ ] Task 2.8 / TDD: build the bounded native request/result handoff and state rendering; keep child-process/filesystem waits off the UI frame thread, and preserve Workspace plus Agent Room as separate windows/viewports. Add interaction and workflow parity tests before replacing either existing surface.
- [x] Task 2.9 / Plan: expose registered profile templates and coordinator selection in the native Workspace team step; preserve Rust-owned profile validation, private environment carry-over and the 16-profile limit. Verify unique IDs and profile/coordinator persistence through the existing save dispatcher before treating the native screen as editable.
- [x] Task 2.9 / Red/Green: added template ID-collision/capacity and coordinator persistence regressions first; compile failed while the profile helper and request fields were missing. Added template-based agent registration, a unique suffixed ID, the 16-profile cap, coordinator radio controls, and save/restore via the existing Rust dispatcher.
- [x] Task 2.9 / Verify: native probe suite 9 passed, including a real Room::dispatch save/snapshot round trip for Claude/Codex profiles and Codex coordinator; focused feature Clippy, rustfmt and diff checks pass. src/native_ui_probe.rs (439 lines) and tests/native_ui_probe.rs (302 lines) remain under 500 lines. Production UI migration is still open.
- [x] Task 2.9 / Cross-platform verify: exact-source run 38808098099 passed after a failed-job-only retry. macOS and Windows validation, native UI probe, formatting/Clippy, bilingual manual rendering, and both exact release-package smoke jobs passed; the first Windows UI Automation `NO_HEADING` did not reproduce.
- [x] Task 2.10 / Resource: remove the full workspace snapshot clone from every native UI frame by borrowing disjoint team state fields; preserve existing Team behavior and pending-state controls.
- [x] Task 2.10 / Verify: the existing native probe regressions (9 passed), feature Clippy and workspace `cargo fmt --check` pass; `src/native_ui_probe.rs` is 439 lines and its test remains 302 lines. Exact-source run 38812385807 passed macOS and Windows validation plus bilingual manual and exact release-package smoke jobs.
- [x] Task 2.11 / Plan → Red → Green: add a Rust-native Agent Room snapshot renderer for the run goal, run state, and each task's assigned agent/status; the regression first failed because only the workspace goal was rendered. Active tasks pulse green, approval/blocked states are amber, failures red, completed green, and pending/unknown gray. The view requests a 500 ms repaint only while work is running.
- [x] Task 2.11 / Verify locally: 11 native UI regressions pass, focused feature Clippy and rustfmt pass; `src/native_ui_probe.rs`, its Rust renderer module and the integration test are 434, 69 and 54 lines. The 500 ms blink test exposed that a continuous sine pulse could repeat the same color at repaint boundaries; use alternating half-second phases instead. This rendering slice remains in the opt-in prototype; actual production-window integration, cross-platform UI acceptance and exact-source CI remain open.
- [x] User workflow directive: keep normal push/PR CI triggers; verify each implementation unit locally and continue without waiting for hosted CI, which may run independently. Release still runs the reusable macOS/Windows validation.
- [x] Task 2.12 / Plan: render the actual `Controller::snapshot` envelope in the native Agent Room probe while retaining the startup/proposal snapshot and existing prototype fixtures; preserve the existing Rust snapshot schema.
- [x] Task 2.12 / Red: a regression using the real `report.state`, `report.plan` and `report.tasks` shape failed because the renderer only read top-level fields.
- [x] Task 2.12 / Green: read the nested report when present, with fallbacks for startup `run_state`, workspace proposal plan and existing top-level probe snapshots.
- [x] Task 2.12 / Verify locally: 12 native UI tests, all-target compile, source-layout (500-line/test separation), feature Clippy, rustfmt and diff checks pass. Graphify extraction of the touched renderer/test produced 10 nodes and 13 edges. CocoIndex is unavailable as a CLI or enabled tool here; no dependency was added.
- [x] Task 2.12 / Complete: the bounded renderer slice is complete, but production Agent Room connection, screen/workflow parity and native platform acceptance remain open.
- [x] Task 2.13 / Plan: add a probe-only launch path that opens a real run through `Room::open`, shows the real controller snapshot in an Agent Room surface, and keeps the existing workspace probe path intact. Do not launch WebView or construct a second Room/controller; retain standard Room recovery behavior.
- [x] Task 2.13 / Red: the real-run/accessibility regression did not compile because the probe had no Agent Room connection constructor.
- [x] Task 2.13 / Green: add `--room <run-dir>` to the existing opt-in probe binary; construct through `Room::open`, render only Room controls, and refresh snapshots asynchronously on the existing bounded worker. Active jobs refresh every second and idle runs every five seconds, matching the shipped Room cadence.
- [x] Task 2.13 / Verify locally: 15 native UI tests, native probe binary check, default all-target compile, source-layout (500-line/test separation), feature Clippy, rustfmt and diff checks pass. Touched source/test files remain under 500 lines.
- [x] Task 2.13 / Complete: the actual run controller is now reachable in the opt-in native probe without starting WebView or constructing another Room/controller. `Room::open` retains its standard recovery/auto-resume policy. Production UI replacement, full workflow/accessibility parity, native macOS/Windows UI acceptance and resource measurement remain open.
- [x] Task 2.14 / Plan: expose a confirmed, reasoned stop action in the native Agent Room probe using the existing `Controller::action`/`workflow::stop` boundary; preserve busy-state disabling and audit semantics.
- [x] Task 2.14 / Red: accessible stop/confirmation and revision-bound stop-request regressions failed because the native Room had no stop controls or request builder.
- [x] Task 2.14 / Green: show Stop only for stoppable states; require confirmation and a nonempty reason; send the revision-bound request through the existing worker; preserve the accepted message and refresh the controller snapshot.
- [x] Task 2.14 / Verify locally: the real SQLite-backed fixture transitions from `waiting_availability` to `stopped`; stop controls and reason field are accessible, terminal runs hide/clear a stale confirmation, and empty or >16 KiB reasons are rejected. All 17 native UI tests, default all-target compile, source-layout, focused feature Clippy, rustfmt and diff checks pass. All modified Rust source/test files are ≤500 lines (`native_ui_probe.rs` is exactly 500).
- [x] Task 2.14 / Complete: the stop interaction is implemented in the probe. Existing coordinator stop processing remains authoritative; an accepted request is not described as an instantaneous process kill. Production UI replacement, other Room actions, cross-platform acceptance and resource measurement remain open.
- [x] Task 2.15 / Plan: add a read-only candidate evidence view using `Controller::evidence`; show the exact validated diff, test outcomes and evidence revision, load asynchronously, and invalidate evidence when the run revision changes.
- [x] Task 2.15 / Red → Green: add an accessible, on-demand evidence action to the Room probe; collect existing `Controller::evidence` asynchronously, show the candidate, test outcomes/output and validated diff, and discard results from a stale run revision. A renderer/contract regression was introduced before the evidence renderer and reply path existed.
- [x] Task 2.15 / Verify locally: full default all-target suite passed (333 passed, 14 ignored); 21 focused Room/native UI/diff evidence tests passed; feature Clippy with warnings denied, formatting and source-layout pass. Touched Rust source/test files are below 500 lines. Remote push/PR CI is not awaited. Production UI migration remains incomplete and the probe is still opt-in.
- [x] Task 2.15 / Complete: the read-only evidence view is available in the native Room probe; production UI parity, app-window integration and macOS/Windows acceptance remain open.
- [x] Task 2.16 / Plan: advance the opt-in Rust Workspace probe toward the specified 1. Workspace settings → 2. Agent roles → 3. Review the plan flow (R-45/R-47 and R-151; SF-39/SF-40 and UC-17; DD-160/DD-161). Keep the shipped WebView path unchanged until workflow parity is verified.
- [x] Task 2.16 / Red: the revision-bound Prepare regression initially failed because the request builder was absent; step navigation also exposed an outdated AccessKit button contract.
- [x] Task 2.16 / Green: connect step navigation, the existing Rust plan-generation/progress/proposal-review path through `Room::dispatch`, team/coordinator saving, role-agent assignment and observed-model selection. Model choices come only from each registered profile's observed catalog; saved role rules pass through existing Rust validation.
- [x] Task 2.16 / Verify → Complete: 26 focused native UI tests, source-layout (500-line bound), feature Clippy with warnings denied, rustfmt, and `git diff --check` pass locally. Largest touched Rust file is `tests/native_ui_probe.rs` at 492 lines. Wiki and AgentMemory record the verified slice and its opt-in boundary. Production WebView replacement, cross-platform UI acceptance and resource measurement remain open.
- [x] Task 2.17 / Plan → Red → Green → Verify → Complete: expose registered agent choices and observed model catalogs in the Team step; save them in the existing RoleRules shape through the Rust workspace dispatcher. A save-contract test first failed while the role-aware path was missing; AccessKit verifies the role/model selector names. The generated planner already validates and applies these same rules. Included in Task 2.16's 26-test, Clippy, source-layout and formatting verification.
- [x] Task 2.18 / Plan (R-78, QAS-56/QAS-73, DD-86): bring per-profile quota badges and asynchronous model/quota refresh controls into the native Team step. Reuse the existing Rust metadata queue and authenticated CLI subprocesses; poll only while metadata work is active and keep refresh/actions independent per profile.
- [x] Task 2.18 / Red: the profile UI/request regressions first failed to compile because the native Team screen had no profile metadata controls.
- [x] Task 2.18 / Green: expose per-profile observed model counts, quota badges/tooltips, and accessible model/quota refresh actions; bind requests to the current snapshot revision and dispatch them through the bounded Rust worker. Refresh snapshots while metadata is active at the existing 1s cadence and use 5s while idle. Merge only matching observed catalogs so unsaved/new profile drafts survive.
- [x] Task 2.18 / Verify → Complete: 30 focused native UI tests, source-layout (500-line bound), feature Clippy with warnings denied, rustfmt, and diff checks pass locally. Largest touched Rust file is `tests/native_ui_probe.rs` at 492 lines. Wiki and AgentMemory updated. This is still opt-in probe behavior; custom-agent catalog editing, full profile editor, shipped UI replacement, and native platform/resource acceptance remain open.
- [x] Task 2.19 / Plan (R-67 → BG-25 → SF-55 → UC-55 → FR-51/NFR-25 → QAS-60 → DD-73): add native per-agent runtime preference editing for executable, JSON argv, input mode, session cap, and 1–60 minute quota cadence; reuse the existing native OS picker and save only the profile list so unrelated workspace drafts stay unsaved.
- [x] Task 2.19 / Red → Green → Verify → Complete: accessible editor/save/picker and persistence regressions were added first; cadence-only saves retain a ready proposal, executable changes clear stale observed catalogs and require review, and path picking does not launch a CLI. Focused native UI suites (33), quota preferences (2), source-layout, feature Clippy, formatting, diff checks and the 500-line bound pass locally. Wiki and AgentMemory updated. Provider switching and ID renames remain a separate R-68 slice.
- [x] Added the root `goal.md` with the requested delivery, architecture, TDD, documentation, tooling, memory, GitHub issue and release workflow.
- [x] Task 2.20 / Plan (R-68 → BG-26 → SF-56 → UC-56 → FR-52/NFR-26 → QAS-61 → DD-74): port complete provider-template replacement to the opt-in native Team editor; generate a collision-free provider ID, clear stale catalog data, and submit `renames` so coordinator, role, and fallback references follow the profile. Keep executable discovery filesystem-only and asynchronous/off the UI thread.
- [x] Task 2.20 / Red → Green → Verify → Complete: tests first failed for missing provider replacement and rename request support. The native selector now replaces the whole profile template, resolves executable paths on the bounded background worker without launching a CLI, and sends the backend reference rename map. Local coordinator/role/fallback drafts follow the new ID and persisted profiles are reloaded after save. Verification passed 36 native UI tests, 5 provider-switch tests, source-layout, feature Clippy, formatting, diff checks, and the ≤500-line bound (largest touched file: `profile_tools.rs`, 388 lines). Wiki and AgentMemory updated. Custom CLI protocol authoring stays in issue #12.
- [x] Task 2.21 / Plan (R-152 → BG-107 → SF-139 → UC-142 → FR-137/NFR-49 → QAS-148 → DD-162): issue #12 is classified as `enhancement`; define a machine-local, versioned JSON registry with declarative CLI invocation, JSONL/model/quota rules, explicit bounds, and no script/API-key execution. Preserve verified Rust process-control adapters for capabilities that cannot be safely declared as data.
- [ ] Task 2.21 / Task → TDD: **deferred until the production UI migration is active**. Implement the strict, symlink-safe registry loader and file diagnostics first, then bind validated definitions into the agent template registry. Add malformed/duplicate/boundary tests before parsing or UI integration.
- [ ] Task 2.21 / Verify → Complete: **deferred until the production UI migration is active**. Map custom JSONL event and quota declarations to normalized runtime data; preserve all built-in agent fixtures, auth boundaries and resource limits. Verify local tests, source-layout, Clippy, Graphify and cross-platform JSON/file handling; update the wiki/AgentMemory and commit/push each finished slice. Unsupported protocol declarations stay explicit failures rather than provider fallbacks.
- [ ] Task 2.22 / Plan — production native UI conversion (R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160/DD-161): move the eframe/egui implementation from an opt-in probe into the actual Workspace and Agent Room launch paths in reviewable slices. Before changing the default launcher, inventory each shipped screen/action against the probe, define parity tests for approvals/settings/navigation/accessibility, and preserve the current WebView as fallback until the native path passes those contracts.
- [ ] Task 2.22 / Task → TDD: integrate one production screen at a time through `Room::dispatch` and the bounded background worker; keep process/filesystem waits off the UI frame thread. Test the real `ui-new`/`ui` launch paths, screen actions, keyboard/accessibility labels, bilingual strings, and no duplicate Room/controller creation.
- [ ] Task 2.22 / Verify → Complete: measure idle CPU and physical footprint against the shipped baseline; verify Korean IME, macOS and Windows behavior, app resources/licenses, and workflow parity. Remove WebView/owned UI JavaScript only after both windows and settings/workspace management are covered. Update AgentMemory/wiki, validate locally, and commit/push each complete slice; do not release a partial cutover.
- [x] Task 2.22 / Initial launch integration: the actual `ui-new` and `ui` commands now select Workspace and Agent Room surfaces explicitly. With the opt-in native feature they open eframe through one shared launcher and reuse the command's existing Rust `Room`; default builds still use the shipped native WebView shell.
- [x] Task 2.22 / Initial verification: 37 native UI tests across five suites, native-feature all-target Clippy (`-D warnings`), source-layout (500-line bound), default and feature all-target compilation, formatting, and diff checks pass locally. This validates launch-path wiring only; production default remains WebView and full UI parity is still open.
- [x] Task 2.22 / Native palette TDD: the new egui palette regression first failed against stock dark visuals; `appearance::apply` now maps the guide's E/F surface, text, fill, selection, warning/error, and active-control values while retaining the saved theme choice.
- [x] Task 2.22 / Native palette verify: 11 tests across appearance, interface theme/language, and source-layout suites; native-feature all-target Clippy (`-D warnings`), rustfmt, diff checks, the 500-line bound, and offline probe build pass.
- [ ] Task 2.22 / Native palette visual acceptance: refresh the current preview from this build and inspect Workspace and Agent Room in Korean/light and Korean/dark against `docs/design-guide.md` and the existing manual screenshots. The default shipped launcher remains WebView.
- [x] Task 2.23 / Plan → TDD: port Agent Room's human candidate decisions into the native surface. Approve/merge require on-demand evidence for the exact current revision, each action is explicitly confirmed, rejection requires a bounded reason, and requests carry the current candidate/revision into the existing Rust controller.
- [x] Task 2.23 / Red → Green: exact candidate/revision action payload tests and the stale-evidence confirmation regression were added first; the launch suite first failed to compile without the decision request, then the UI test caught an approval/merge confirmation that survived a revision change. The native controller now clears stale confirmations; backend revision, diff and candidate checks remain authoritative.
- [x] Task 2.23 / Verify → Complete: 41 tests across five native UI suites, native-feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 448 lines, `native_room_render.rs` 392, `native_ui_probe.rs` tests 497. Scoped Graphify: 197 nodes / 383 edges. Wiki and AgentMemory updated. UI still runs only in the opt-in feature; shipped screen parity remains open.
- [x] Task 2.24 / Plan → TDD: port the Agent Room's Resume and Recover controls. Match current run-state gating; Recover requires explicit confirmation and leaves live/unknown process refusal to the existing Rust recovery guard. Bind both actions to the current revision and keep work on the existing background worker.
- [x] Task 2.24 / Red → Green: resume/recover request tests first failed to compile because the shared request builder was missing; the UI contract then exposed private confirmation state. The final controls preserve shipped state gating, revision-bound requests, explicit recovery confirmation, and clear the dialog when the run leaves `running`.
- [x] Task 2.24 / Verify → Complete: 43 tests across five native UI suites, native-feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 455 lines, `room_actions.rs` 128, `native_room_render.rs` 464. Scoped Graphify: 214 nodes / 408 edges. Update wiki/AgentMemory and commit/push this slice; feature remains opt-in while broader parity is incomplete.
- [x] Task 2.25 / Plan → TDD: port the Agent Room's human-gated tool request panel. Display the exact tool/input/task/status from the Rust snapshot, allow one-time approve, reject, or refresh only for a pending request, confirm each decision, and bind the request to the live revision and exact request ID.
- [x] Task 2.25 / Red → Green: request tests first failed to compile without the exact-receipt request builder; the AccessKit panel test confirmed visible decision controls and caught a test assertion that confused text nodes with accessible button names. The corrected renderer displays the full bounded request input and clears confirmation on revision/status changes. Live decisions remain available only for active runs; idle approvals auto-resume after successful action completion.
- [x] Task 2.25 / Verify → Complete: 45 tests across five native UI suites, native-feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 461 lines, `room_actions.rs` 334, `native_ui_launch.rs` 235, `native_ui_probe.rs` tests 497. Scoped Graphify: 220 nodes / 432 edges. Wiki and AgentMemory updated. The native path remains opt-in pending remaining workflow parity and platform acceptance.
- [x] Task 2.26 / Plan → TDD: port the Agent Room's read-only team briefing to native UI. Fetch through the existing Rust Room dispatcher on the background worker; show observed session/process state, work, last activity, availability restrictions, attention, and approval summary. No agent control or side effect is added.
- [x] Task 2.26 / Red → Green: the native rendering regression first failed because `State` had no briefing payload. The dispatcher integration test first used a startup controller, which correctly rejects Agent Room requests; changing it to open a completed run exercised the real `briefing` endpoint on the worker. The panel renders captured sessions and approval summaries with local timestamps and remains read-only.
- [x] Task 2.26 / Verify → Complete: 47 tests across five native suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 474 lines, `room_actions.rs` 334, `room_briefing.rs` 145, `native_ui_launch.rs` 311. Scoped Graphify: 232 nodes / 446 edges. CocoIndex is unavailable. Wiki/AgentMemory updated. Native UI remains opt-in pending remaining parity and platform acceptance.
- [x] Task 2.27 / Plan (UC-02, UC-04, UC-141): add explicit native review consent and start for the exact ready proposal. Render task assignments, owned paths, model, tests, repository base and digest before confirmation; bind the start request to the current startup revision and proposal digest. Preserve all existing backend validation and human approval recording.
- [x] Task 2.27 / Red → Green: request tests first failed because the revision-bound start request was missing. Added explicit consent showing goal, repository/base, digest, workspace mode, task agent/model/executable/arguments/permission/fallback, writable paths, and test command. Confirmation clears on revision/digest/phase change, and start routes through the bounded Rust worker. A native-worker integration fixture generated a real reviewed plan and verified the exact approved plan starts through `Room::dispatch`.
- [x] Task 2.27 / Verify → Complete: 50 tests across five native suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `native_ui_probe.rs` 456 lines, `workspace.rs` 404, `execution.rs` 123, `native_ui_launch.rs` 454. Scoped Graphify: 247 nodes / 471 edges. Wiki/AgentMemory updated; native prototype parity at that checkpoint was 12/25 (48%), 1 partial, 12 not started. Release remains WebView and 0/25 converted.
- [x] Task 2.28 / Plan (UC-28, UC-38, UC-141): add native task-plan editing for assignment, session model, instructions, owned paths, dependencies and test command. Support bounded add/remove, validate the edited typed plan, save against the current revision/digest, and require the updated plan to be reviewed and approved again.
- [x] Task 2.28 / Red → Green: added request validation first (missing export failed to compile), then rendered editable task fields. Invalid JSON argv remains editable until corrected. The dispatcher integration test creates a real ready proposal, refuses a stale digest, saves the edited plan, and verifies a new digest; the AccessKit test verifies accessible field names.
- [x] Task 2.28 / Verify → Complete: 54 tests across six native UI suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed file is `src/ui/setup/dispatch.rs` at 496 lines. Updated the fixed denominator: prototype parity 13/25 (52%), one partial and 11 not started; release conversion remains 0/25. Wiki/AgentMemory updated; native UI remains opt-in pending production parity and platform acceptance.
- [x] Task 2.29 / Plan (GitHub #16, UC-02, UC-141): make the native Review surface display an explicit start-in-progress status immediately after approval, then route to Agent Room only after the report for the exact approved run exists. Keep the current WebView launch unchanged until UI parity is complete; the issue remains open until the Rust view is in the shipped app.
- [x] Task 2.29 / Red → Green: pending-label and exact-run readiness tests first failed to compile without the shared Rust helpers. The transition retains the returned run path, polls asynchronously, refuses a prior run's report, opens the existing Room dispatcher, and loads its snapshot before replacing Review with Agent Room. Failed or unopenable runs stop the transition and show the error.
- [x] Task 2.29 / Verify → Complete: 57 tests across seven native UI suites, 11 compatibility tests across three suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest touched Rust file is `workspace.rs` at 479 lines. Prototype parity stays 13/25 (52%), one partial and 11 not started; release conversion remains 0/25. Issue #16 remains open because the shipped UI still uses WebView. Wiki/AgentMemory updated and commit/push completed.
- [x] Task 2.30 / Plan (GitHub #15, UC-28): when an edited task plan differs from the reviewed digest, make the save action visually prominent and state that saving requires reviewing the new plan again. Preserve the existing disabled state when no edits exist or a save is pending.
- [x] Task 2.30 / Red → Green: the accessibility-tree regression first failed because the dirty state had no explicit next-step label. The native editor now exposes “Plan changes need review” and an accent-filled “Save changed plan and review again” action only for dirty drafts.
- [x] Task 2.30 / Verify → Complete: 58 tests across seven native UI suites, feature all-target Clippy (`-D warnings`), source-layout, rustfmt, diff and 500-line checks pass locally; largest touched files are 237 and 220 lines. Graphify updated the isolated UI graph to 562 nodes / 1,075 edges. CocoIndex incrementally indexed 519 files into 5,493 chunks with zero errors. Workflow parity remains 13/25 (52%) prototype and 0/25 release conversion. GitHub #15 remains open until the shipped UI includes the fix. Wiki/AgentMemory updated; commit/push completed.
- [x] Task 2.31 / Plan (GitHub #11, UC-06, UC-16): make the active coordinator visible in native plan review with a pulsing status light, reusing the Room running-state color and repaint cadence. Keep terminal and non-running states static.
- [x] Task 2.31 / Red → Green: a regression first failed because the plan screen had no activity-color source. The shared planning module now returns a pulse color only for a running plan, and Review renders it as an accessible coordinator indicator while its existing 500 ms repaint keeps the pulse moving.
- [x] Task 2.31 / Verify → Complete: 59 tests across eight native UI suites, feature all-target Clippy (`-D warnings`), source-layout, rustfmt, diff and 500-line checks pass locally; touched files are 472, 26, 497 and 18 lines. Graphify updated the isolated UI graph to 596 nodes / 1,152 edges. CocoIndex indexed 521 files into 5,497 chunks with zero errors. Workflow parity stays 13/25 (52%) prototype and 0/25 release conversion. GitHub #11 remains open pending shipped UI verification. Wiki/AgentMemory updated; commit/push completed.
- [x] Task 2.32 / Plan (GitHub #14, UC-23): surface each newly pending Claude Code tool request immediately in the native Agent Room, with exact tool/task/input evidence and an explicit decision. Preserve the existing revision-bound confirmation, allow/deny validation, and inline request list; defer only the request the operator dismisses.
- [x] Task 2.32 / Red → Green: the new native UI regression first failed because pending receipts appeared only in the inline list. A per-request modal now displays exact request evidence and accessible allow/deny/defer controls. Allow and deny still require the existing second confirmation; deferring one request keeps it in the list and advances the next pending request.
- [x] Task 2.32 / Verify → Complete: 61 tests across nine native UI suites and five live Claude tool-decision integration tests pass; feature all-target Clippy (`-D warnings`), source-layout, rustfmt, diff and 500-line checks pass locally. Largest touched files are 481 lines (`room.rs`), 438 (`room_actions.rs`), 475 (`native_ui_launch.rs`) and 123 (`native_ui_tool_dialog.rs`). Graphify updated the isolated UI graph to 653 nodes / 1,272 edges. CocoIndex indexed 522 files / 5,512 chunks with zero errors. Workflow parity remains 13/25 prototype and 0/25 release conversion. GitHub #14 remains open until shipped UI verification; wiki/AgentMemory updated and commit/push completed.
- [x] Task 2.33 / Plan (CI regression): ensure every integration test that imports the optional `native-ui-probe` module is Cargo-gated by that feature, so the production-default `cargo build --all-targets --locked` remains valid.
- [x] Task 2.33 / Red → Green: the new Rust manifest contract first failed for the four missing Cargo test targets; all nine native UI integration targets are now explicitly gated by `native-ui-probe`. Platform validation runs all nine feature suites together using a literal YAML block, and the release-workflow contract checks the block style and target coverage.
- [x] Task 2.33 / Local Verify: default `cargo build --all-targets --locked`, default all-target Clippy, feature-enabled Clippy, source-layout, targeted rustfmt, diff checks and the nine UI suites plus release/manifest contracts (69 passed) are green. An initial full default all-target test run exposed one stale workflow assertion; it was corrected and the affected contract suites pass.
- [ ] Task 2.33 / Cross-platform Verify: Actions run 38838080638 passed macOS, including the default all-target build and test suite, but Windows failed the Rust release-workflow contract because the checkout's CRLF line endings did not match its LF-only assertion. Both workflow contract paths now normalize CRLF to LF, with a synthetic CRLF regression; verify the exact pushed source on Windows before closing this CI task.
- [x] Task 2.34 / Plan (UC-141, FR-136, QAS-147, DD-161; UI workflow #14): expose bounded cache preview and explicit digest-bound clearing from the native Workspace screen. Reuse existing Rust cache policy and `Room::dispatch` on the background request worker; keep protected work/evidence outside the clearable inventory. Do not block the UI thread, add a service, or enable destructive action before preview.
- [x] Task 2.34 / Red: add a feature-gated native cache contract covering the preview/clear request payloads, accessible summary/path rendering, and empty-cache state; it first failed because the native cache surface did not exist.
- [x] Task 2.34 / Red regression: the existing exact Workspace accessibility test then failed because it did not include the new cache control; update that shared button contract so the addition remains visible to assistive technology.
- [x] Task 2.34 / Green: connect the Workspace cache control to async preview, display reviewed disposable paths/size and protected or uncertain count, then clear only the exact preview digest through the existing Rust handler.
- [x] Task 2.34 / Verify → Complete: the background Room worker previews and clears a real disposable cache fixture while preserving `startup.json`; 6 cache-specific tests and all 12 native feature/workflow suites (76 passed) pass. The default all-target regression passed 518 tests with 14 ignored; the subsequent CRLF-adjusted `release_notes` suite passes 8 tests. Default all-target build, both default/feature Clippy (`-D warnings`), source-layout, formatting, diff and 500-line checks pass. CocoIndex: 525 files / 5,536 chunks; scoped Graphify: 698 nodes / 1,328 edges. Prototype parity is now 14/25 (56%), with 1 partial (4%) and 10 not started (40%); shipped/default UI remains 0/25 pending full parity and platform acceptance.
- [x] Task 2.35 / Plan (UC-141, FR-136, QAS-147, DD-161; UI workflow #3): finish the native registered-agent lifecycle by adding a trash-icon removal action with explicit confirmation. Save through the existing Rust `save_profiles` boundary, remove coordinator/role/fallback references using its established semantics, cancel editing of a removed profile, and disable removal during active metadata/profile-edit operations.
- [x] Task 2.35 / Red: extend native profile accessibility and lifecycle regressions for the remove control, explicit confirmation, reference cleanup and profile-registry request. The UI accessibility contract was observed failing before the remove action was implemented.
- [x] Task 2.35 / Green: confirmed removal updates the local profile draft, clears stale coordinator/role/fallback references, closes editing for the removed profile, and emits `SaveProfiles` through the existing asynchronous Rust dispatcher path.
- [x] Task 2.35 / Verify → Complete: all 12 native profile tests and 5 provider-switch persistence/reference tests pass; default `cargo check --all-targets`, feature Clippy (`-D warnings`), changed-file rustfmt, diff checks, and the explicit 500-line count pass. The broad feature test run was stopped after 120 passed / 2 ignored to avoid an unnecessary full-suite wait; focused profile and persistence tests are the completion evidence. CocoIndex indexed 525 files / 5,549 chunks; scoped Graphify is 714 nodes / 1,458 edges. Prototype parity is now 15/25 (60%), with 0 partial and 10 not started (40%); shipped/default UI remains 0/25.
- [x] Task 2.36 / Plan (R-39/R-40/R-88/R-89 → BG-17/BG-43/BG-44 → SF-33/SF-34/SF-75/SF-76 → UC-30/31/78/79 → FR-24/25/73/74, NFR-16 → QAS-31/32/84/85 → DDR-13 → DD-47/48/96/97; UI workflow #9): add one native workspace menu backed by the existing asynchronous `Room::dispatch` operations. Cover list/reopen/switch, named creation, remove-from-list, and confirmed archival deletion using exact name/path; surface backend refusals and leave the active workspace intact on failed actions.
- [x] Task 2.36 / Red: feature-gated tests first failed because the native workspace navigation module was absent. Contracts cover list/create/open/switch/forget/delete, discard confirmation, exact archive confirmation, accessible menu actions, and workspace reload.
- [x] Task 2.36 / Green: implemented native workspace navigation in `workspace/navigation.rs` (439 lines), reusing the existing request worker and Rust backend validation. Create/open/switch refresh the workspace snapshot asynchronously; forgetting only removes the registry entry; archival requires the exact workspace name and states project data is preserved.
- [x] Task 2.36 / Verify → Complete: 5 navigation, 4 workspace registry, and 8 release-workflow tests pass; default all-target check, native-feature all-target Clippy (`-D warnings`), source-layout (including the 500-line bound), formatting, and diff checks pass. Prototype parity is 16/25 (64%), 0 partial, 9 not started; shipped/default UI remains 0/25 pending full parity and platform acceptance. Issue #9 remains open.
- [x] Task 2.36 / CI follow-up: added the missing `Open workspace menu` accessible button label to the macOS probe expectation; all 14 `native_ui_probe` tests and feature-gated Clippy pass on `codex/issue-9-accessible-workspace-menu`. Exact-source macOS/Windows validation and native-package jobs passed for `7bfad6e` in [run 38844468171](https://github.com/neodamul/AgentConductor/actions/runs/38844468171).
- [x] Task 2.36 / Package-gate regression: exact-source run 38991687927 passed macOS/Windows validation but both package jobs exposed missing `egui-wgpu 0.37.0` license text. Added Cargo-source VCS pin `7c0b404`, official release-license revision `841a328`, SHA-256 provenance, and a native-package regression; focused provenance and packaging tests pass locally.
- [x] Task 2.36 / Local package follow-up: the canonical run exposed missing provenance for `objc2-metal` 0.2.2/0.3.2, `objc2-quartz-core` 0.3.2, and `spirv 0.4.0+sdk-1.4.341.0`; exact upstream documents/VCS/digests now replace the duplicate generic record. The 214-dependency audit also found Cargo's `Apache-2.0/MIT` spelling; packaging now accepts it and normalizes SBOM output to SPDX `OR`. 3 provenance tests, 2 package tests, and the regression pass.
- [x] Task 2.36 / Smoke-log regression: package assembly succeeded after the 214-dependency audit, then repeated local smoke found a `command-0` collision because evidence reused a fixed directory. A TDD regression failed on the second same-path run; each smoke now gets a unique evidence subdirectory and both runs pass.
- [x] Task 2.36 / Verify local package: exact-source `cargo run --release --locked -- package-release` passed on `6eb27d5781b5`; macOS archive SHA-256 is `e3b081a417da6938be903aac26ccecde57a988af455f4af9bec0c550368fb8c9`. Manifest records the exact source commit and 214 dependencies; all four packaged smoke cases passed.
- [ ] Task 2.36 / Platform verify: rebuild the final documented source, then rerun exact-source macOS/Windows validation and packaging; continue issue #9 acceptance gates.

### Native workspace settings import/export (R-52 / DD-59; UI workflow 10)
- [x] Plan: R-52 → BG-20 → SF-43 → UC-42 → FR-37/NFR-18 → QAS-44 → DDR-15 → DD-59; complete the existing share workflow in the opt-in native Workspace UI without changing the secure Rust backend.
- [x] Red: added native request and accessibility contracts; the first run failed because the native sharing module did not exist.
- [x] Green: reuse the asynchronous `RequestWorker` and existing `export_settings`, `preview_settings`, and `import_settings` requests. Disable import/export while the workspace has unsaved edits; show the imported settings summary and backend warnings; default permission claims to unverified and require explicit import confirmation; refresh after import.
- [x] Verify: 7 native workspace navigation/sharing tests, 2 existing real workspace-share tests, focused native-feature Clippy, and rustfmt pass. The 500-line bound holds (largest changed Rust file: 490 lines).
- [x] Verify: exact-source CI run 38852463148 passed all four jobs for `a69132b`: macOS validation (6m42s), Windows validation (14m49s), macOS package build/render/execute (6m2s), and Windows package build/render/execute (10m5s).
- Prototype parity: 17/25 (68%); 8 not started; 0 partial. Shipped/default UI conversion remains 0/25, so issue #9 stays open.
- [x] CI regression triage: run 38851108577 passed macOS build, source-layout, and applicable tests, then failed the native UI contract because `native_probe_controls_have_accessible_names` did not expect the new Import/Export buttons. The same assertion failed locally; updated it to include both accessible labels.
- [x] Follow-up local verification: all 76 native UI tests across 11 suites, both native binary checks, and the exact feature-gated Clippy command passed before the contract-only correction. The corrected `native_ui_probe` suite (14 tests) and focused Clippy pass; every changed Rust file is at most 499 lines.
- [x] Re-run exact-source macOS/Windows CI for commit containing the accessible-label test correction; all validation and exact native-package jobs passed in run 38852463148.
- [x] Plan: R-128 → BG-83 → SF-115 → UC-118 → FR-113/NFR-49 → QAS-124 → DD-137; this slice ports only theme selection into the opt-in native UI.
- [x] Red: add `tests/native_ui_appearance.rs` first; it failed because no native appearance module or selector existed.
- [x] Green: add the shared navigation menu, persist the existing light/dark preference, apply the saved theme from cached state, and use semantic panel fill/stroke for workspace cards.
- [x] Verify: 2 appearance and 7 workspace tests, feature all-target Clippy (`-D warnings`), rustfmt, diff checks, and a rebuilt macOS preview. Accessibility exposes Appearance/Light/Dark; the dark preference survives app restart and workspace cards render in dark mode.
- [ ] Remaining workflow 11: native language switching and status-tag parity; keep prototype parity at 17/25 complete, 1 partial, 7 not started. Shipped/default UI remains 0/25 because this feature is opt-in.
- [x] Plan: R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76; this slice adds the shared native selector and localizes only shared navigation labels.
- [x] Red: add a restart-persistence/accessibility contract; it failed because native settings exposed no language selector or localized step tabs.
- [x] Green: persist English/Korean in the existing machine-wide preferences; localize the shared Settings menu, theme choices, workspace-menu accessible label, and three workflow tabs from the existing catalogs.
- [x] Verify: 1 language persistence/accessibility, 2 appearance, 7 workspace, 13 native probe, 5 i18n-catalog, and 2 existing interface-language tests pass; native-feature all-target Clippy, changed-file rustfmt, changed-file diff checks, and changed-source line counts pass. Manual macOS preview confirms Korean menu/tabs and saved selection.
- [ ] Remaining workflow 11: translate the remaining Agent Room controls/content. Native status tags now have their own verification slice; prototype parity remains 17/25 complete, 1 partial, 7 not started; shipped/default UI remains 0/25.
- [x] Native Workspace form localization (R-70 / DD-76 / UC-58): translated workspace name, project folder, goal, isolation choices, save action, and settings-sharing labels/actions in visible text and accessible names; user-entered values remain untouched and bilingual catalogs are reused.
- [x] Red: the expanded native language contract failed because execution-isolation and sharing heading labels were absent from the accessible tree.
- [x] Green: localized the Workspace form and settings-sharing controls using existing catalogs and explicit accessible names; updated the native probe contract to follow the active language.
- [x] Verify: 1 language, 7 workspace, 2 appearance, 13 native-probe, 5 catalog, and 2 interface-language tests pass; native-feature all-target Clippy (`-D warnings`) passes. Rebuilt and inspected the single macOS preview window with persisted Korean selection and user-entered values intact.
- [ ] Native task/run status tags (R-70 / DD-137 / UC-58): localize states, expose task identity and status to assistive technology, use theme-specific WCAG AA color pairs, and pulse the running indicator without polling or disk reads.
- [x] Red: the run-render accessibility test failed because task state was only colored text with no task/status accessible name.
- [x] Green: render compact semantic status tags for run and task states, reuse the bilingual catalog, preserve the active pulse with scheduled repaints, and classify blocked as an error.
- [x] Verify: 2 native status, 12 room render, 12 launch, 2 tool-dialog, 13 native-probe, and 5 i18n tests pass; feature all-target Clippy, changed-file rustfmt/diff and 500-line checks pass. Rebuilt preview confirms Korean task labels and subdued amber status tags in light and dark themes.
- [x] Plan (R-70 / DD-76 / UC-58): localize the primary Agent Room controls and stop-confirmation flow in both visible text and accessibility names; keep stop gating and review dispatch unchanged.
- [x] Red: add a Korean Agent Room accessibility contract first; it failed because the refresh, briefing, stop, and review controls exposed English labels.
- [x] Green: reuse the existing bilingual catalog for those controls and the stop dialog, retaining the existing confirmation validation and bounded stop-reason input.
- [x] Verify: 35 focused Agent Room, status, launch, tool-dialog, and i18n tests pass; feature all-target Clippy (`-D warnings`), rustfmt, and 500-line checks pass (`room.rs` 486 lines; touched probe test 494 lines). A full feature all-target run exposed the old probe contract's English-only refresh label under saved Korean settings; the contract now reads the selected-language catalog and its regression passes. Rebuilt preview inspection is in progress.
- [x] Plan (R-70 / DD-76 / UC-58): localize candidate approve/reject/merge actions and confirmation prompts, including the rejection-reason field, while preserving exact-candidate evidence checks, explicit confirmation, and bounded reason validation.
- [x] Red: the new Korean candidate-decision accessibility contract first failed because candidate buttons still exposed English names.
- [x] Green: reuse existing approve/reject/merge catalog entries, add bilingual confirmation and rejection-reason entries, and expose each confirmation prompt through the accessibility tree.
- [x] Verify: 33 focused candidate, Agent Room language/render, launch, tool-dialog, and i18n tests pass; focused native-library plus candidate-test Clippy (`-D warnings`) and direct formatting checks pass. `room.rs` is 492 lines and the new test is 86 lines. The full feature all-target run was cut short after its earlier locale-only failure was corrected; source-layout was not run to completion.
- [x] Plan (R-70 / DD-76 / UC-58): localize pending tool-request review, decision prompts, actions, task/status context, and accessible names in the native Agent Room; preserve revision-bound request validation and approval gating.
- [x] Red: extend the tool-dialog AccessKit regression in Korean; it failed because the request actions still exposed English accessible names.
- [x] Green: reuse catalog actions where available and add bilingual labels for tool confirmations, context, fallback text, and accessible action names. The underlying approve/reject/refresh eligibility and exact request ID remain unchanged.
- [x] Verify: 9 focused tool-dialog, Agent Room language, candidate, and i18n tests pass; one unrelated broad source-reference test was excluded after it stalled reading the full source tree. Rustfmt and diff checks pass; changed Rust files are within 500 lines (`room.rs` 493, `room_actions.rs` 482, tool-dialog test 186). The rebuilt opt-in preview reopened as one macOS window on the preserved Korean Workspace data. Scoped Graphify extraction: 258 nodes / 325 edges. CocoIndex indexed 534 files / 5,662 chunks (7 reprocessed, 0 errors).
- [ ] Verify remaining for this slice: native-feature Clippy and full source-layout test; Clippy was interrupted after prolonged full-crate source reads. This opt-in build still does not change the shipped WebView UI.
- [x] Plan (R-70 / DD-76 / UC-58): localize native run-resume and recovery controls plus the recovery safety explanation, preserving the Rust run-state gates and refusal to recover live/unknown processes.
- [x] Red: extend the Korean Agent Room AccessKit contract; it failed because Resume and Recover controls retained English names.
- [x] Green: reuse the existing interrupted-run and recovery-safety catalog entries, add bilingual Resume/Cancel/Confirm labels, and pass the selected language through the existing Room control renderer.
- [x] Verify tests: the Korean Agent Room Accessibility test passes for resume/recover labels, cancellation/confirmation, and the visible recovery safety text; the bilingual catalog key/placeholder contract also passes. Rustfmt is applied.
- [x] Verify remaining: the Korean Room language test and bilingual catalog key/placeholder test pass; Rustfmt and changed-file diff checks pass; Rust files remain below 500 lines (`room.rs` 493, `room_actions.rs` 489, Room language test 82). The native probe builds with debug info disabled and was reopened as one macOS window; existing Korean Workspace values remain intact. Scoped Graphify: 256 nodes / 323 edges. CocoIndex: 534 files / 5,667 chunks (7 reprocessed, 0 errors).
- [x] Follow-up verification: native-feature all-target Clippy (`-D warnings`) and the full `source_layout` integration test pass on the current checkout. These checks do not change the fact that the opt-in native build has not replaced the shipped WebView UI.
- [x] Plan (R-70 / DD-76 / UC-58): pass the selected language into the read-only Agent Room team briefing and localize its visible labels, known statuses, fallback values, timestamps, and approval summary. Preserve the existing read-only data and dispatcher behavior.
- [x] Red: add a Korean rendering regression for briefing title, completed and availability-wait states, session metadata, process state, and the empty approval summary; it failed while the briefing remained English-only.
- [x] Green: reuse the bilingual message catalog, status-label helper, and template interpolator; no new dependency, polling, or mutable action was added.
- [x] Verify: both Agent Room language tests and the bilingual catalog ID/placeholder contract pass; native probe builds, Rustfmt and diff checks pass, and changed Rust files remain under 500 lines (`room.rs` 493, `room_briefing.rs` 179, language test 135). The refreshed macOS preview has one window and preserves the existing Korean Workspace values. Scoped Graphify: 54 nodes / 111 edges; CocoIndex: 534 files / 5,674 chunks, 7 reprocessed, 0 errors. Workflow 11 remains partial at 17/25 prototype workflows; shipped/default conversion remains 0/25.
- [x] After commit: update AgentMemory and GitHub issue #9 with the team-briefing localization and its verification.
- [x] Plan (R-70 / DD-76 / UC-58): localize the Agent Room heading and read-only review-evidence summary while preserving raw test output, diff bytes, current evidence checks, and the existing English `render_evidence` API.
- [x] Red: Korean Room rendering first failed on the English heading; a new evidence contract also failed to compile because the shared renderer had no language-aware entry point.
- [x] Green: route the Room through `evidence_in_language`, keep the existing public English wrapper, and reuse bilingual catalog interpolation for revision, candidate, test exit/timeout, output stream, and truncation labels.
- [x] Verify: 3 native Room-language tests, 12 Room-render tests, and the catalog ID/placeholder contract pass; native-feature all-target Clippy (`-D warnings`), `source_layout`, rustfmt, diff checks, and the native probe build pass. All changed Rust files are within 500 lines (`native_ui_probe.rs` 497, `room.rs` 493, `render.rs` 270; affected tests 180 and 452). The updated preview has one window and retains the current Korean Workspace values. Scoped Graphify: 129 nodes / 283 edges; CocoIndex: 534 files / 5,686 chunks, 9 reprocessed, 0 errors.

### Native MCP server registration — DD-98 partial workflow 12
- [x] Plan: use existing R-90 → BG-45 → SF-77 → UC-80 → FR-75 → QAS-86 → DD-98; registration remains distinct from Jira/Confluence source import.
- [x] Red: add tests for loading saved OpenCode-shaped definitions, empty settings, revision-bound JSONC save requests and the 64 KiB dispatcher limit; they first fail because the native MCP editor does not exist.
- [x] Green: add the registration entry to the shared Settings menu and a native JSON/JSONC editor. Save uses the existing background Rust worker and revision-bound `mcp_save`; saving only persists definitions and never launches the server. Refresh the snapshot without replacing other workspace drafts.
- [x] Verify: focused MCP/settings/language/native-probe/backend/source-layout tests pass (24 total); native-feature all-target Clippy passes. An initial source-layout check caught `workspace.rs` at 501 lines; move `execution_ready` into its execution module and keep both `native_ui_probe.rs` and `workspace.rs` at 500 lines.
- [x] Verify: 24 tests across six MCP/settings/language/native-probe/backend/source-layout suites; native-feature all-target Clippy (`-D warnings`), `cargo fmt --check`, `git diff --check`, and the ≤500-line source-layout guard pass. Built and reopened the existing single macOS preview process; Settings → MCP servers renders the OpenCode JSON/JSONC editor with the saved empty maps. Graphify scoped 14 files: 220 nodes / 467 edges. CocoIndex command is installed but cannot start because `mcp.server.fastmcp` is missing.
- [x] Complete: pushed implementation commit `927575c`; AgentMemory record `mem_mvaojp4g_504e4c27874c` and [issue #9 progress update](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6210944269) are saved.
- [ ] Workflow 12 remainder: discover MCP tools, preview Jira/Confluence references and separately approve applying source evidence.
- Prototype parity remains 17/25 complete, 2 partial, 6 not started (24%); shipped/default UI conversion remains 0/25.

### Native messenger settings and delivery — workflow 13
- [x] Plan: reuse R-43 → BG-18 → SF-37 → UC-61 → FR-28/NFR-17 → QAS-35/QAS-66 → DD-79; expose the existing Rust save/test boundary from native Settings without exporting secret values.
- [x] Red: the bilingual native form/accessibility and revision-bound request tests first failed because the native messenger editor and Settings route were absent.
- [x] Green: add localized Settings access, asynchronous status/save/test handling, masked Slack/company webhook fields, explicit test delivery, saved-secret preservation, and pending-state interaction lockout through the existing Rust dispatcher.
- [x] Verify: 28 tests across seven targeted suites; native-feature all-target Clippy (`-D warnings`), `source_layout`, bilingual message catalog, formatting, diff checks, and the ≤500-line limit pass locally.
- [x] Complete: native prototype parity is 18/25 (72%), with 2 partial and 5 not started; shipped/default conversion remains 0/25. The existing single preview process was not replaced or duplicated.
- [ ] Acceptance: compare the rebuilt native screen against `docs/design-guide.md` and the current manual screenshots, verify keyboard/Korean IME and accessibility on macOS and Windows, and perform an authorized live webhook test.

### Native Workspace roster and goal header — workflows 11/16 partial
- [x] Plan: keep the #9 migration opt-in and preserve the shipped Workspace behavior and the 25-workflow denominator; follow the existing light/dark design tokens and screen-reader labels.
- [x] Red: add bilingual AccessKit coverage for the goal headline and all three stage actions; it first fails because the native header module is absent.
- [x] Green: add a persistent agent roster with status, session/model counts, quota details and refresh, coordinator marking, and edit/register actions. Restore the goal-first heading, step eyebrow, clearer card hierarchy, and legible secondary text; keep polling limited to active planning.
- [x] Verify: 43 tests across nine native/UI/catalog/source-layout suites; feature all-target Clippy with warnings denied; native probe build; formatting, diff and 500-line source checks pass locally.
- [x] Preview: replace the existing single macOS preview process with the rebuilt binary; visually confirm the bilingual goal heading, selected stage, agent roster, and retained workspace draft.
- [ ] Acceptance: review both themes against the documented design and current manual screenshots, then verify Korean IME, keyboard/screen-reader operation, and macOS/Windows behavior. Prototype parity remains 18/25 (72%), with 2 partial and 5 not started; shipped/default conversion remains 0/25.

### Native task feedback and reassignment — workflow 18
- [x] Plan: reuse task feedback R-110 → BG-65 → SF-97 → UC-100 → FR-95 → QAS-106 → DD-118 and pending-task assignment R-121 → BG-76 → SF-108 → UC-111 → FR-106 → QAS-117 → DD-130; keep the native UI on the existing Rust Room boundary.
- [x] Red: add revision-bound task feedback/reassignment, bilingual accessibility, and real Room-dispatch persistence tests before adding controls; initial compile failed because the task controls module was absent.
- [x] Green: add localized task/profile selectors, per-task feedback, bounded validation, audited Room requests, and resume after accepted feedback; keep assignment disabled while work is active.
- [x] Verify: 50 tests across nine suites, native-feature all-target Clippy (`-D warnings`), `cargo fmt --check`, offline native binary build, `source_layout`, diff check, and all changed Rust files under 500 lines.
- [x] Complete prototype slice: native workflow parity is 19/25 (76%), 2 partial, 4 not started; shipped/default conversion remains 0/25. Live-window interaction, Korean IME/keyboard and Windows acceptance remain open.

### Native task session and model selection — workflow 19
- [x] Plan: reuse R-20 → BG-09 → SF-11 → UC-11 → FR-02 → QAS-11 → DD-13; selecting a task selects its owned session, while the provider session ID remains read-only and the chosen model must come from that agent's observed catalog.
- [x] Red: test bilingual session/model controls, revision-bound model actions, catalog rejection, and persistence through Room dispatch; first run failed because `Action::SetModel` and the controls did not exist.
- [x] Green: show the selected task's provider session ID with an accessible name and allow an idle task's assigned agent model to be changed through the existing Rust action.
- [x] Verify: 51 tests across nine native suites, 8 native-session/recovery tests, native-feature all-target Clippy (`-D warnings`), rustfmt, offline native build, source layout, diff checks, and all changed Rust files under 500 lines. Scoped Graphify AST extraction: 360 nodes / 726 edges across 24 Rust files. CocoIndex CLI is unavailable in this environment.
- [x] Complete prototype slice: native parity is 20/25 (80%), 2 partial, 3 not started; shipped/default conversion remains 0/25. Live GUI, Korean IME/keyboard, and Windows acceptance remain open.

### Native schedule after availability restrictions — workflow 20
- [x] Plan: reuse R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136; accept explicit RFC 3339 timestamps with timezone, keep scheduling limited to a waiting task, and send through Room's existing Rust scheduler.
- [x] Red: add bilingual schedule-control, revision-bound request, timestamp/status validation, and persisted manual-schedule tests; the first run exposed the missing action and the fixture was changed to produce a real quota-waiting run.
- [x] Green: expose the restriction reason, timezone-explicit timestamp field and schedule action only for waiting tasks; route persistence through the existing Rust scheduler without starting work before the due time.
- [x] Verify: 61 tests across 11 native UI/session/recovery suites, feature all-target Clippy (`-D warnings`), rustfmt, offline native build, source layout, diff check, and all changed Rust files under 500 lines. Scoped Graphify extraction: 20 nodes / 31 edges across 2 Room-control files; CocoIndex CLI is unavailable in this environment.
- [x] Complete prototype slice: native parity is 21/25 (84%), 2 partial, 2 not started; shipped/default conversion remains 0/25. Live GUI, Korean IME/keyboard, and Windows acceptance remain open.

### Native task activity, logs, and lessons — workflow 21
- [x] Plan: reuse R-104 → BG-59 → SF-91 → UC-94 → FR-89 → QAS-100 → DD-112 and the existing bounded `Room::Task` detail response; keep provider event summaries separate from JSONL raw logs and show only reviewed lesson assets.
- [x] Red: add localized Activity/Raw logs/Lessons rendering, Claude/Codex/OpenCode event normalization, unknown-event and no-raw-JSON checks, plus real Room-dispatch coverage; first run failed because the native module did not exist.
- [x] Green: load selected-task details through the existing async request worker; refresh with the Room snapshot cadence and render bounded output.
- [x] Verify: 66 tests across 12 native UI/session/recovery suites, feature all-target Clippy (`-D warnings`), rustfmt, offline native build, source layout, diff check, and all changed Rust files under 500 lines. Scoped Graphify extraction: 57 nodes / 125 edges across 4 Room-control files; CocoIndex CLI is unavailable in this environment.
- [x] Complete prototype slice: native parity is 22/25 (88%), 2 partial, 1 not started at this checkpoint; shipped/default conversion remains 0/25. Live GUI, Korean IME/keyboard, and Windows acceptance remain open.

### Native agent tools guide — workflow 15 slice
- [x] Plan: reuse `R-81 → BG-36 → SF-68 → UC-71 → FR-66/NFR-37 → QAS-77 → DD-89`; keep the guide in the Workspace menu, read-only, bilingual, and independent of Settings.
- [x] Red: extend native UI tests to expose the guide entry and every command to assistive technology; the new test first failed because the native guide did not exist.
- [x] Green: render an inline, scrollable native guide with selectable commands and explicit copy buttons; copy only writes to the clipboard and no tool process/network request is made.
- [x] Verify: 11 guide/workspace/language tests, native-feature all-target Clippy (`-D warnings`), `source_layout`, offline native build, rustfmt, diff checks, and the ≤500-line guard pass. Scoped Graphify extraction: 30 nodes / 61 edges. CocoIndex remains unavailable because its CLI environment lacks `mcp.server.fastmcp`. No additional app window was opened.
- [x] Complete this slice: workflow 15 moves from not started to partial. Native parity remains 22/25 complete (88%), 3 partial, 0 not started; shipped/default conversion remains 0/25. Manual reading and release updates remain to be ported before workflow 15 is complete.

### Native offline user manual — workflow 15 slice
- [x] Plan: reuse `R-144 → BG-99 → SF-131 → UC-134 → FR-129 → QAS-140 → DD-153`; parse the existing Markdown once per open/language change and render headings, lists, tables, and code with native egui controls.
- [x] Red: add Rust tests for block parsing and menu exposure; the initial run failed because the native block parser and manual view were missing.
- [x] Green: share the block parser with HTML generation, connect User manual from the Workspace menu, and render the current manual with native egui controls and code-copy actions.
- [x] Verify: 17 tests across five suites, committed HTML generation parity for both languages, native-feature all-target Clippy (`-D warnings`), `source_layout`, offline native build, rustfmt, diff checks, and the ≤500-line guard pass. Scoped Graphify extraction: 47 nodes / 101 edges. CocoIndex remains unavailable because its CLI environment lacks `mcp.server.fastmcp`.
- [x] Screenshot TDD / Red: extend the native AccessKit test over every screenshot referenced by the English and Korean manuals; the new image-role assertion failed while the native viewer exposed only screenshot text.
- [x] Screenshot Green: render the six whitelisted bundled assets as bounded egui images with accessible alt text and captions; decode on demand through an optional native-only Rust image dependency, cache textures in egui context data, and release the cache when the manual closes.
- [x] Screenshot Verify: all referenced English/Korean screenshots render with accessible image labels; native manual tests (3) and HTML renderer tests (4) pass offline. The changed Rust/test files remain below 300 lines; full repository layout remains red on 52 pre-existing oversized files.
- [x] Full local verification: all-target native regression reached the known source-layout failure (52 files remain oversized); focused native manual/HTML tests, all-target Clippy, default/probe binary builds, rustfmt and diff checks pass.
- [x] Tooling refresh: Graphify rebuilt the structural graph without LLM calls (6,086 nodes / 11,051 edges); CocoIndex refreshed 7 changed files with zero errors (566 files / 5,962 chunks).
- [ ] Complete: workflow 15 stays partial until screenshots and release updates are also available in the native interface.

### Native release update panel — workflow 15 slice
- [x] Plan: reuse the existing `update_status`, `check_update`, and `apply_update` Rust dispatcher and its checksum-verified installer; keep network work on the existing request worker and preserve the all-workspaces idle gate before install.
- [x] Red: add native status, release-note, check/apply action, and accessibility contracts before rendering the update panel; the focused test first failed because the panel/module was missing.
- [x] Green: open the localized updates panel from Settings, show scrollable release notes, and poll only while the panel's check/install is active.
- [x] Verify: 19 focused tests across six suites, native-feature all-target Clippy (`-D warnings`), `source_layout` (442 files; 197.21 seconds), changed-file rustfmt, offline native probe binary build, diff check, and ≤500-line checks pass. Cargo required a temporary `CARGO_HOME` because the default global cache stalled before spawning rustc. Scoped Graphify: 72 nodes / 134 edges. CocoIndex remains unavailable because `mcp.server.fastmcp` is missing.
- [ ] Complete: workflow 15 remains partial until native screenshots and automatic update notification are preserved.

### Automatic native release notification — workflow 15 / DD-135
- [x] Plan: reuse `R-126 → BG-81 → SF-113 → UC-116 → FR-111 → QAS-122 → DD-135`; check immediately and every 600 seconds, keep checks asynchronous, poll an active update while its panel is closed, and show a localized accessible update button beside Settings.
- [x] Red: add deterministic ten-minute scheduling, pending-request deferral, closed-panel status polling, and AccessKit notification contracts; they fail before the native scheduler/button exists.
- [x] Green: use one egui repaint deadline, the existing Rust update dispatcher, and the existing SettingsAction route; avoid duplicate checks and idle polling. Defer scheduling until the initial snapshot arrives so response ordering stays valid.
- [x] Verify: 16 focused update/appearance/workspace tests, all-target native Clippy, default and native-probe builds, rustfmt, changed-file 300-line limits, and diff checks pass. The complete suite remains blocked by the separately tracked repository-wide 300-line migration.
- [x] Complete: native prototype workflow 15 now retains screenshot and automatic update notification behavior; shipped/default UI migration and cross-platform acceptance remain separate open work.

### Safe OpenCode MCP registration examples — GitHub #17 / DD-163
- [x] Plan: trace R-153 → BG-108 → SF-140 → UC-143 → FR-138/NFR-50 → QAS-149 → DD-102/DD-163; show examples only when no MCP server is registered, preserve source bindings, and keep sample servers inert JSONC comments.
- [x] Red: add a test that the empty native editor exposes local/remote OpenCode fields while parsing to empty settings; observe it fail before seeding the form.
- [x] Green: show examples only when the MCP server map is empty; preserve any existing source bindings, suppress the sample note for registered servers, keep examples commented, use `{env:VARIABLE}` and `.invalid`, and retain save-without-start behavior.
- [x] Verify: 16 tests across MCP, i18n, message-catalog, and source-layout suites; native-feature all-target Clippy (`-D warnings`); `cargo fmt --check`; `git diff --check`; offline native probe build; and 163/151-line Rust files. The existing single macOS preview window stays open and visibly shows examples for its empty server map. Graphify scoped extraction: 149 nodes / 305 edges. CocoIndex remains unavailable because `mcp.server.fastmcp` is missing.
- [x] Complete: commit `b8929bf` is pushed on the issue-linked branch `codex/issue-17-opencode-mcp-examples`; issue #17 is closed after acceptance, issue #9 progress is updated, and AgentMemory record `mem_mvap9360_680561d463cb` captures the handoff. The native UI remains opt-in and shipped UI parity is unchanged.
- [x] Task 3.2 / TDD first slice: port all five agent-tools-guide checks to `tests/agent_tools_guide.rs`; the new Rust check first failed while the Python unittest file existed, then passed after its removal.
- [x] Verify first slice: 5 Rust guide tests, all 55 remaining Python contracts, source-layout, formatting, diff check and all-target Clippy pass.
- [x] Task 3.2 second slice: port the MIT license/manifest/README consistency contract into `tests/project_license.rs`; its guard first failed while the Python module existed, then passed after removal.
- [x] Task 3.2 third slice: port stable-release triggers, validation dependency, permissions, archive/checksum publication and pinned-action checks into `tests/release_notes.rs`; remove the redundant Python release-workflow contract.
- [x] Task 3.2 fourth slice: port PNG/ICO/ICNS/RGBA container and dimensions, embedded-titlebar artwork identity, and Cocoa/Windows shell icon contracts to `tests/app_brand.rs`; remove the Python unittest file.
- [x] Task 3.2 fifth slice: port UTF-8 text-I/O and embedded UI charset contracts to `tests/utf8_sources.rs`; remove the Python unittest module.
- [x] Task 3.2 sixth slice / TDD: replace the Python/Node update-note display contract with a Rust integration test; prove the text-binding/scroll contract and absence of the prior Python module.
- [x] Verify sixth slice: Rust all-target regression 503 passed, 14 ignored, 1 slow source-layout test filtered; all 43 remaining Python contracts pass; formatting, all-target Clippy, diff and changed-file line-limit checks pass.
- [x] Task 3.2 seventh slice / TDD: move i18n catalog integrity, placeholder parity, explicit source references, static-markup checks and legacy-helper checks from Python into Rust; retain the live language-switch/user-content runtime regression in the existing Rust test target until the production WebView UI is replaced. The Rust ownership guard failed first while the Python module existed.
- [x] Verify seventh slice: 5 Rust localization-contract tests and 3 existing message/runtime tests pass; all-target Rust regression passes 515 tests with 14 ignored and the one slow source-layout test filtered; all 38 remaining Python tests pass; the source-layout test passes separately; formatting, all-target Clippy, diff check and the 307-line test-file limit pass.
- [x] Cross-platform TDD / Red: exact-source run 38817585058 passed macOS but Windows `mcp_remote::streamable_http_reads_sse_without_waiting_for_server_disconnect` timed out at the fixture's 3-second per-request deadline; the focused macOS test passes.
- [ ] Cross-platform TDD / Green: relax only the slow-runner fixture deadline while retaining its open-SSE/no-EOF assertion, then rerun exact-source Windows and macOS CI.
- [ ] Task 3.2 remaining: migrate the other 19 Python contract modules and remove their runner/helper only after Rust equivalents pass.
- [ ] Task 3: finish exact-source cross-platform verification of Rust package assembly, license manifest, archive smoke checks and remaining release/manual tooling; then assess remaining Python verification paths.
- [ ] Verify: all workflows and cross-platform packages pass with no project-owned JS/Python runtime or build dependency; inspect archive contents and idle resource use.
- [ ] Complete: remove superseded authored JS/Python paths and CI steps, commit/push independently verified units, update documentation and release through the registered skill.

### 300-line source and test bound — R-154 / GitHub #9
- [x] Plan: R-154 → BG-109 → SF-141 → UC-144 → FR-139/NFR-51 → QAS-150 → DDR-17 → DD-164. Add the limit to `goal.md`, default planner rules, requirements, and architecture documents.
- [x] Red: change `tests/plan_rules.rs` to require 300 lines; confirmed it fails against the old default.
- [x] Red: lower `tests/source_layout.rs` to 300 lines; confirmed it fails and names the 57 oversized files.
- [x] Task / Green slices: split review/recovery CLI dispatch and native task-activity views/transcript formatting by responsibility; retain public dispatch/render paths.
- [x] Verify slices: 14 review/stop/recovery CLI tests and 5 native task-activity UI tests pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; all new files are below 300 lines.
- [x] Task / Green subunit: split src/native_ui_probe.rs into app, probe, and worker modules while keeping open_window and RequestWorker re-exports at their original paths.
- [x] Verify subunit: 88 tests across 16 native UI suites; native-feature all-target Clippy, rustfmt, and diff checks pass; the four files are 63/88/263/78 lines. The global layout guard now reports 53 remaining files.
- [x] Task / Green subunit: route settings/profile/MCP/messenger replies through poll/settings.rs; separate workspace refresh and error handling into poll/workspace_ops.rs and poll/errors.rs while preserving pending-state transitions.
- [x] Verify poller subunit: the same 88 tests across 16 native UI suites, native-feature all-target Clippy, rustfmt, and diff checks pass; poll.rs and its three child modules are 279/144/45/19 lines. The global layout guard now reports 52 remaining files.
- [x] Task / Green subunit: move the independent legacy i18n assertion into `tests/i18n_legacy_compatibility.rs`, reducing `tests/i18n_contracts.rs` from 307 to 294 lines.
- [x] Verify i18n test split: all 5 tests across both suites pass; rustfmt and diff checks pass; both files are below 300 lines. The global layout guard no longer lists this file and reports 51 remaining violations.
- [x] Task / Green subunit: split metadata-refresh and catalog-preservation checks into `tests/native_ui_profile_refresh.rs`, reducing `tests/native_ui_profiles.rs` from 326 to 293 lines.
- [x] Verify profile test split: all 12 tests across both suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; the files are 293 and 32 lines. The global layout guard reports 50 remaining violations.
- [x] Task / Green subunit: move missing-workspace-file denial coverage into `tests/guard_policy_missing_file.rs`, reducing `tests/guard_policy.rs` from 315 to 287 lines.
- [x] Verify guard-policy split: all 7 tests across both suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; the files are 287 and 65 lines. The global layout guard reports 49 remaining violations.
- [x] Task / Green subunit: split native task scheduling and waiting-state controls from `tests/native_ui_task_controls.rs` into `tests/native_ui_task_scheduling.rs`; preserve the task controls and timezone-aware scheduling assertions.
- [x] Verify task-control split: 6 tests across both suites, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 217 and 183 lines. The global layout guard reports 48 remaining violations.
- [x] Task / Green subunit: split Unix process approval and protocol tests into `tests/process_control.rs` and share the process fixture in `tests/support/process.rs`; lifecycle tests remain in `tests/process.rs`.
- [x] Verify process-test split: all 12 tests across both suites pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 159, 146, and 40 lines. The global layout guard reports 47 remaining violations.
- [x] Task / Green subunit: separate reviewed-license, license-notice, and upstream-byte checks into `tests/release_package_licenses.rs`; share only the package fixture with assembly/failure tests.
- [x] Verify release-package test split: all 7 tests across both suites pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 104, 211, and 58 lines. The global layout guard reports 46 remaining violations.
- [x] Task / Green subunit: move cooperative-worker process harness functions into `tests/support/cooperative_worker.rs`; keep all behavioral cases and assertions in `tests/cooperative_worker.rs`.
- [x] Verify cooperative-worker split: all 7 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 74 and 297 lines. The global layout guard reports 45 remaining violations.
- [x] Task / Green subunit: move the company CLI packet integration case into `tests/cooperative_cli.rs` and share setup/issue fixtures across both suites.
- [x] Verify cooperative CLI split: 12 tests pass and 1 remains ignored across both suites; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. The global layout guard reports 44 remaining violations.
- [x] Task / Green subunit: move the explicitly ignored local-agentmemory daemon round-trip case to `tests/cooperative/daemon_round_trip.rs` while retaining the existing shared fixtures and target.
- [x] Verify cooperative memory split: 11 tests pass and 1 remains ignored; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. The global layout guard reports 43 remaining violations.
- [x] Task / Green subunit: move native room stop/confirmation and candidate-decision contracts into a nested module under the existing integration target.
- [x] Verify native room split: all 12 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 293 and 163 lines. The global layout guard reports 42 remaining violations.
- [x] Task / Green subunit: move native launch/tool-decision request contracts into nested `tests/native_ui_launch/actions.rs` under the existing integration target.
- [x] Verify native launch split: all 12 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 295 and 185 lines. The global layout guard reports 41 remaining violations.
- [x] Task / Green subunit: isolate native Workspace persistence, profile/team, and Room probe cases in a nested module under `tests/native_ui_probe.rs`.
- [x] Verify native probe split: all 13 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 239 and 258 lines. The global layout guard reports 40 remaining violations.
- [x] Task / Green subunit: move Git-reference, report/SQLite, notification, and run-knowledge persistence into `src/workflow/persistence.rs`; preserve the private workflow-level `save` path for child modules.
- [x] Verify workflow persistence split: 14 tests across workflow, recovery, checkpoint, and CLI suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 297 and 65 lines. The global layout guard reports 39 remaining violations.
- [x] Task / Green subunit: move frozen-evidence review orchestration to `src/workflow/review/orchestration.rs`; preserve `workflow::review`, coordinator-only review, and `review_as` visibility paths.
- [x] Verify review split: all 9 review tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 182 and 222 lines. The global layout guard reports 38 remaining violations.
- [x] Task / Green subunit: move subprocess/Git/workspace identity verification into `src/workflow/permissions/context.rs`; preserve `workflow::verify_tool_context` for CLI, scheduler, and native-guard callers.
- [x] Verify permission-context split: 8 tests across tool-context, live-decision, and guarded-control suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 252 and 88 lines. The global layout guard reports 36 remaining violations.
- [x] Complete subunits: commits `3c2640b` (review command dispatch), `599307b` (native task-activity views), and `1f4b155` (native UI launcher modules) are pushed to `codex/issue-9-accessible-workspace-menu`.
- [x] Knowledge tools: Graphify refreshed after the tool-controls split: 6,349 nodes / 10,545 edges / 484 communities. CocoIndex was invoked but its installed cocoindex-code environment fails to import mcp.server.fastmcp; no fresh CocoIndex index is claimed.
- [x] Red: the 300-line source-layout gate reported 34 violations before this slice, including `src/native_ui_probe/workspace/messenger.rs` at 339 lines.
- [x] Task / Green subunit: move `State::show` and its rendering helpers into `workspace/messenger/view.rs`, preserving `messenger::State::show`, request builders, event behavior, and dispatcher entry points.
- [x] Verify subunit: both `native_ui_messenger` tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 135 and 213 lines; the global source-layout gate now reports 33 remaining violations.
- [x] Complete subunit: pushed the messenger-view split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line gate reported 33 violations before this slice, including `src/native_ui_probe/room/task_controls.rs` at 417 lines.
- [x] Task / Green subunit: split task-control rendering and request validation into `view.rs` and `request.rs`, preserving the public `task_controls::show` and `task_controls::request` paths.
- [x] Verify subunit: 6 task-control/scheduling tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 41, 281, and 104 lines; the source-layout gate now reports 31 remaining violations.
- [x] Complete subunit: pushed the task-controls split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line guard reported 31 violations before this slice, including `src/ui/setup/share.rs` at 326 lines.
- [x] Task / Green subunit: split export, preview, and import/archive staging into focused child modules; preserve the existing `Startup` dispatch methods and `crate::ui::setup` visibility.
- [x] Verify subunit: 6 workspace-share/archive/messenger/MCP tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 41, 70, 37, and 192 lines; the source-layout gate now reports 30 remaining violations.
- [x] Complete subunit: pushed the settings-sharing split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line guard reported 30 violations before this slice, including `src/native_ui_probe/room.rs` at 497 lines.
- [x] Task / Green subunit: move the Agent Room view and decision controls into `room/view.rs`; preserve `room::show`, state, event, and renderer call paths.
- [x] Verify subunit: 19 render/language/tool-dialog/candidate tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 237 and 268 lines; the source-layout gate now reports 29 remaining violations.
- [x] Complete subunit: pushed the Agent Room view split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line gate reported 29 violations before this slice, including `src/native_ui_probe/room_actions.rs` at 491 lines.
- [x] Task / Green subunit: move tool-request rendering and confirmation controls to `room_actions/tool_view.rs`; preserve `room_actions::tool_controls`, request validation, and action dispatch paths.
- [x] Verify subunit: 27 native launch/tool-dialog/Room-render tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 211 and 286 lines; the source-layout gate now reports 28 remaining violations.
- [x] Complete subunit: pushed the tool-controls split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [ ] Current Red: the global 300-line guard still identifies 27 files above the limit; the source-build launcher now selects native Rust UI, but release-package acceptance remains open.
- [ ] Remaining source-bound ledger: `src/cooperative.rs` (436), `src/coordinator.rs` (405), `src/git.rs` (394), `src/guard.rs` (458), `src/icons.rs` (484), `src/mcp/imports.rs` (445), `src/memory/client.rs` (393), `src/messenger/inbound.rs` (381), `src/messenger.rs` (385), `src/native_guard.rs` (415), `src/process.rs` (446), `src/quota/display.rs` (404), `src/quota/terminal/windows.rs` (365), `src/quota/terminal.rs` (424), `src/quota.rs` (445), `src/state.rs` (417), `src/ui/controller.rs` (436), `src/ui/native/cocoa.rs` (461), `src/ui/room.js` (367), `src/ui/setup/dispatch.rs` (496), `src/ui/setup.rs` (486), `src/update.rs` (426), `src/workflow/execution.rs` (481), `src/workflow/recovery.rs` (371), `src/workflow/scheduler.rs` (461), `tests/support/mock_agent.rs` (500), and `tests/workspace_registry.rs` (380).
- [ ] Task / Green: split each remaining listed file by responsibility, preserving exports and behavior; keep `src/` and `tests/` separate.
- [ ] Verify: source-layout guard, focused suites, all-target regression, formatting, Clippy, native feature build, and diff checks.
- [ ] Complete: commit/push remaining source-bound slices and update the issue/wiki. Continue issue #9 until all acceptance criteria pass; do not start other issue triage first.

### Release package writer split — issue #9 / 300-line source bound
- [x] Plan: preserve `release_package::assemble` and package outputs while separating archive writing from package validation and license review.
- [x] Red: `source_layout` failed with `src/release_package.rs` at 350 lines; the seven release-package and license tests passed before extraction.
- [x] Green: move the private `PackagePlan` and `write_package` implementation into `src/release_package/archive.rs`; keep the public API and helper error paths unchanged.
- [x] Verify: seven release-package/license tests, native all-target Clippy (`-D warnings`), the locked application build, rustfmt, and diff checks pass. The strict source-layout test now reports 28 other files; the extracted files are 219 and 140 lines.
- [x] Complete subunit: pushed `6d763fa`, recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219267552), saved AgentMemory `mem_mvb5aiww_40844f57405f`, and signaled Claude (`sig_mvb5b1pu_c9527e792667`). The source-layout guard reports 28 remaining violations; issue #9 remains open.

### Release smoke package validation split — issue #9 / 300-line source bound
- [x] Plan: preserve `release_smoke::validate_package` and `release_smoke::run_smoke` while separating package validation, subprocess orchestration, and workflow scenarios.
- [x] Red: the 300-line source guard identified `src/release_smoke.rs` at 453 lines; the six exact-package smoke tests passed before the move.
- [x] Green: split the existing functions into `validate.rs`, `run.rs`, and `workflow.rs`, retaining the public re-exports and process timeout/error behavior.
- [x] Verify: all 6 release-smoke tests pass before and after; native all-target Clippy (`-D warnings`), locked application build, rustfmt, and diff checks pass. The strict source-layout test now reports 27 other files; new Rust files are 6, 74, 146, and 242 lines.
- [x] Complete subunit: pushed `ecf2414`, recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219375990), saved AgentMemory `mem_mvb5ie1k_0362c7154e2b`, and signaled Claude (`sig_mvb5ig35_7102ed248986`). The source-layout guard reports 27 remaining violations; issue #9 remains open.

### Native MCP tool discovery — workflow 12 / DD-98
- [x] Plan: extend the existing OpenCode-compatible MCP settings workflow; keep discovery explicit, bound it to the saved workspace revision and server, and reuse the background Rust dispatcher.
- [x] Red: add request serialization and AccessKit contracts first; the focused test failed because native tool discovery had no revision-bound request.
- [x] Green: expose saved-server selection and an accessible **Discover tools** action. Start discovery only on click, show the returned tool catalog, and poll the existing dispatcher once per second only while that MCP job is active.
- [x] Verify: 28 tests across the native MCP, native probe, and language suites; 4 backend MCP workspace/source-binding tests; native-feature all-target Clippy (`-D warnings`); formatting and diff checks pass. The full native-feature all-target regression passes every test except the strict `source_layout` guard, which reports 36 files above 300 lines. Every changed Rust source/test file is below 300 lines.
- [x] Complete: native Settings registration/discovery and Jira/Confluence preview, source review, and explicit apply are implemented in the opt-in prototype. Prototype parity is 24/25; shipped/default conversion remains 0/25.

### Native Jira/Confluence source review — workflow 12 / DD-68, DD-143
- [x] Plan: keep source references explicit as `jira:KEY-123` and `confluence:PAGE_ID`; reuse the existing MCP preview/apply commands and asynchronous Rust dispatcher; preserve the exact-preview, read-only human review gate.
- [x] Red: revision-bound preview/apply tests were added first and failed because the native request builders were missing; they now define the exact saved-revision and displayed-selection contracts.
- [x] Green: add source insertion, preview status, bounded record review, optional page selection, and explicit apply to the native Workspace; keep each source/test file at or below 300 lines.
- [x] Verify: 32 tests across six native/MCP suites, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. Full native-feature all-target regression passes all targets except the strict `source_layout` guard, which reports 36 files above 300 lines. Every changed Rust source/test file is at or below 300 lines.
- [x] Complete: pushed `3100a96` on `codex/issue-9-accessible-workspace-menu` and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6215970001). The LLM Wiki records workflow 12 as complete in the opt-in prototype; shipped/default UI conversion remains 0/25.

### Native task editor localization — workflow 11 / DD-160 (active)
- [x] Plan: pass the saved interface language into the native task editor and resolve visible labels and accessible names through the existing English/Korean message catalogs; leave task IDs, goals, prompts, paths, and agent output as user data.
- [x] Red: add a Korean AccessKit regression for the task editor's field labels, assignment/model controls, and review/save actions; it failed because `task_controls_in_language` did not exist.
- [x] Green: localize task editor headings, labels, controls, and review/save actions while preserving user data, edit validation, request digest, and the existing English `task_controls` entry point.
- [x] Verify: 11 task/editor and native language/catalog tests pass; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. Changed Rust source/test files are 279, 295, 99, and 273 lines, all within the 300-line limit.
- [x] Complete: pushed `88db903` on `codex/issue-9-accessible-workspace-menu` and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6216084493); the LLM Wiki records the slice. Workflow 11 and the 25-workflow denominator remain unchanged until all native controls are bilingual.

### Native role editor localization — workflow 11 / DD-160 (active)
- [x] Plan: pass the selected language into role assignment controls and translate role names, menu actions, model defaults, and accessible names while keeping agent/model IDs unchanged.
- [x] Red: add a Korean AccessKit regression for role headings, add/remove actions, model defaults, and accessible names; it failed because `show_role_rules_in_language` did not exist.
- [x] Green: localize role names, add/remove actions, default-model text, and accessible names while preserving the existing English `show_role_rules` entry point and registered-agent/model validation.
- [x] Verify: 5 role and message-catalog tests pass; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. Changed Rust files are 296, 86, 132, and 84 lines, within the 300-line limit.
- [x] Complete: pushed `d82e17d` on `codex/issue-9-accessible-workspace-menu` and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6216166951); the LLM Wiki records the slice. Workflow 11 remains partial until all native Workspace and Agent Room controls are bilingual.

### Native plan-review localization — workflow 11 / DD-160 (active)
- [x] Plan: route Review headings, progress, elapsed/output details, failures, logs, actions, and save guidance through the existing English/Korean catalogs; keep coordinator IDs, log paths, and agent-provided errors as data.
- [x] Red: add English/Korean catalog and interpolation regressions first; the focused test failed because the new Review messages were absent.
- [x] Green: localize the plan-review surface while preserving plan generation, task editing, execution consent, and existing error content.
- [x] Verify: 22 focused native/message tests pass; the native-feature all-target regression passes 202 tests across 58 suites with 4 ignored when excluding the source-layout guard; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. The strict `source_layout` test separately fails on 36 pre-existing files above 300 lines. Changed Rust/test files are 118 and 56 lines.
- [x] Complete: refresh the LLM Wiki and Graphify, commit/push only this slice, update GitHub #9 and AgentMemory, and signal Claude. Workflow 11 remains partial; prototype parity stays 24/25 and shipped/default conversion stays 0/25.

### Native workspace navigation localization and split — workflow 11 / DD-160 (active)
- [x] Plan: localize Workspace menu and confirmation copy in English/Korean, give the new-workspace field an accessible name, and split the oversized navigation module without changing request/event contracts.
- [x] Red: add Korean AccessKit tests for menu actions, dynamic workspace names, archive safety copy, dirty-workspace loss warnings, and create/open warning interpolation; the test first failed because `show_in_language` was absent, then exposed untranslated controls.
- [x] Green: keep the existing `show` entry point, add a context-based localized renderer, split view/confirmation/helpers into focused modules, and keep each changed source/test file at or below 300 lines.
- [x] Verify: 14 navigation/i18n tests pass; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. The strict source-layout guard now reports 35 other files above 300 lines. Changed Rust/test files are 297, 209, 124, 13, 9, 111, and 300 lines.
- [x] Complete: refreshed the LLM Wiki and Graphify (6,264 nodes / 10,512 edges / 472 communities), committed/pushed this slice, updated GitHub #9 and AgentMemory, and signaled Claude. Workflow 11 remains partial; prototype parity stays 24/25 and shipped/default conversion stays 0/25.

### Native execution-consent localization — workflow 11 / DD-160 (active)
- [x] Plan: reuse the existing bilingual workflow requirement; localize only the exact-plan consent panel and accessible action names while preserving the selected revision/digest, command values, and public English `controls` entry point.
- [x] Red: the new bilingual AccessKit/render regression first failed because Workspace exposed no language-aware execution controls.
- [x] Green: pass the saved language from Plan Review, render all static consent labels through the existing English/Korean catalogs, and retain the original English wrapper and revision-bound start event.
- [x] Verify: 15 tests across execution consent, existing launch compatibility, and catalog contracts; native-feature all-target Clippy (`-D warnings`), formatting, diff checks, and the default all-target compile pass. Changed Rust source/test files are below 300 lines.
- [x] Tooling: Graphify refreshed the canonical tree (6,363 nodes / 10,574 edges / 492 communities). CocoIndex was retried; its launcher still fails to import `mcp.server.fastmcp`, so no fresh CocoIndex index is claimed.
- [x] Complete: committed and pushed as `a242d2a` on the issue #9 branch; posted [progress update](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218320478), refreshed the LLM Wiki/Graphify, and saved AgentMemory records `mem_mvb35m0t_ac1b8524bc43` and `mem_mvb3hgge_406665e57871`. The repository-wide 300-line guard still reports 29 other files; prototype parity stays 24/25 and shipped/default conversion stays 0/25.
- [x] Workflow 11 subunit: localize the native cache-cleanup panel in English and Korean; the broader visible Workspace/Agent Room bilingual audit remains open.

### Default-build native test feature gates — verification bug
- [x] Red: `cargo check --all-targets` failed in the default build because auto-discovered native UI tests referenced optional `eframe`/`native_ui_probe` without requiring `native-ui-probe`.
- [x] Green: add a regression that scans native UI integration test crates and requires either a crate-level feature gate or an explicit Cargo target requirement; gate the existing native-only test crates and list every explicit native test target.
- [x] Verify: feature-gate regression (2 tests), default `cargo check --all-targets`, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. The 300-line repository gate remains red on 29 existing files.
- [x] Complete: included in `a242d2a`, pushed on the issue #9 branch, and tracked in the same [issue progress update](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218320478). AgentMemory bug record: `mem_mvb3hgge_406665e57871`.

### Native cache-cleanup localization — workflow 11 / DD-160 (active)
- [x] Plan: reuse the existing English/Korean catalogs for the native cache panel's heading, state copy, fallback path, action labels, and accessible name; keep paths and counts as data.
- [x] Red: add a rendered-copy regression for Korean and English preview, removed, loading, and empty states; it first failed because the language-aware renderer was missing.
- [x] Green: keep the existing English `show` entry point, add `show_in_language`, and pass the selected Workspace language through one cache-panel method.
- [x] Verify: 8 tests across native cache and language suites pass; default `cargo check --all-targets`, native-feature all-target Clippy (`-D warnings`), formatting, and the native probe build pass. Changed source/test files are 202 and 233 lines, within the 300-line limit.
- [x] Tooling: Graphify refreshed the canonical repository to 6,373 nodes / 10,593 edges / 487 communities. CocoIndex was retried but its CLI still cannot import `mcp.server.fastmcp`; no fresh CocoIndex index is claimed.
- [x] Complete subunit: refreshed the LLM Wiki and Graphify; saved AgentMemory `mem_mvb3rvo0_9541202c30e2`; sent Claude signal `sig_mvb3sfe1_a8dff65a4949`; and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218462009). Workflow 11 still requires a complete bilingual visible-copy audit; native prototype parity remains 24/25 and shipped/default conversion remains 0/25. Commit/push this verified slice separately.

### Native default launch and activity summaries — issue #9 / workflow 11
- [x] Plan: route the source-build `ui-new` and `ui` commands to the existing Rust eframe surface by default, and localize synthesized Agent Room activity summaries in English and Korean.
- [x] Red: the bilingual render regression first found the probe-only Workspace header and English-only generated session/activity summaries; the default-launch contract also failed before eframe became a default feature.
- [x] Green: remove the probe label, use existing message catalogs for generated session, task-list, usage, file-change, exit-code, and clipped-line summaries, and enable the existing native UI feature in default builds.
- [x] Verify: 14 focused native localization tests, the default build, all all-target suites before the layout guard, every all-target suite after it, native all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. The full locked all-target run stops at `source_layout`: 29 files still exceed 300 lines; every changed Rust/test file is below the limit.
- [x] Complete subunit: refreshed the LLM Wiki and Graphify (6,386 nodes / 10,610 edges / 488 communities), saved AgentMemory `mem_mvb55br8_bd0c3051a682`, pushed commit `1c3413e`, posted [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219183508), and signaled Claude (`sig_mvb55edm_6600327d067f`). The 29-file layout guard, full UI parity, and macOS/Windows package acceptance remain open.


### Quota display split — issue #9 / 300-line source limit

- [x] Plan: preserve the existing `quota` public API while moving rendering and time-label helpers into responsibility-based modules.
- [x] Red: the strict layout guard included `src/quota/display.rs` at 404 lines.
- [x] Green: split into `src/quota/display/mod.rs` (115), `labels.rs` (141), and `badge.rs` (156); existing exports and callers are unchanged.
- [x] Verify: quota display regression passes 8/8 before and after; locked application build, native-feature all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass.
- [ ] Verify: repository-wide 300-line guard remains red on 26 other files. Continue issue #9 until the guard is green and native/release acceptance is complete.
- [x] Knowledge: Graphify refreshed (6,413 nodes / 10,623 edges / 488 communities); AgentMemory `mem_mvb5qjfg_74253af00fa0`; Claude signal `sig_mvb5qlrj_00a813ca9aa4`. CocoIndex CLI is unavailable on this host.
- [x] Progress: posted the verified delta and remaining work to [GitHub issue #9](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219503212).


### Bilingual native UI completion — issue #9 / workflow 11 / DD-160

- [x] Plan: finish the English/Korean copy and accessibility audit for the native Workspace and Agent Room while keeping user/provider values unchanged.
- [x] Red: `native_ui_sidebar` failed on the hardcoded English avatar accessible name; `native_ui_status` failed on the hardcoded visible `Run` status prefix.
- [x] Green: route both labels through the existing English/Korean catalogs and interpolate the profile ID as data.
- [x] Verify: 33 tests across 14 UI/i18n suites pass; default application build and native-feature all-target Clippy (`-D warnings`) pass; rustfmt and diff checks pass. Changed source/test files remain below 300 lines.
- [x] Audit: targeted native UI scan found no remaining hardcoded English status/avatar labels; only the invariant `AgentConductor` product heading remains. Workflow 11 is complete in the source build, bringing native prototype parity to 25/25 (100%).
- [x] Knowledge: Graphify refreshed (6,416 nodes / 10,626 edges / 478 communities); AgentMemory `mem_mvb62w4q_5fedb050b69f`; Claude signal `sig_mvb62yin_0028abb68615`.
- [x] Progress: updated GitHub issue #9 to native source-build parity 25/25 while keeping released v0.1.24 conversion at 0/25; [posted the verification](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219714532).
- [ ] Complete issue #9: the published v0.1.24 remains WebView and release conversion is 0/25 until exact macOS/Windows packages and visual, IME/accessibility, resource, and license acceptance pass. Keep the issue open.

### Native launcher replaces the WebView shell — issue #9
- [x] Plan: keep the Rust eframe launcher as the default application UI and remove the unused Cocoa/WebView launcher and direct `wry`/`tao` dependencies.
- [x] Red: the branding regression referenced the deleted WebView2 implementation; replace it with a native eframe icon contract, which first failed because the native window had no icon.
- [x] Green: set the embedded AC RGBA icon on the native viewport, remove WebView-specific launch errors and dependency requirements, and update public install instructions.
- [x] Verify: focused release, i18n, branding, and Windows suites pass (9 tests); full all-target tests pass except the repository-wide `source_layout` guard, which still reports 25 files above 300 lines. Native-feature all-target Clippy (`-D warnings`), default all-target compilation, rustfmt, and diff checks pass. The optimized release build succeeds.
- [ ] Complete: exact macOS/Windows archive acceptance, visual/IME/accessibility/resource checks, and the remaining source-layout violations are still open.

### Remove the legacy browser document from the application — issue #9
- [x] Plan: remove the unused Rust HTML/JavaScript document assembly from `Room`; keep the existing Rust dispatcher and eframe surfaces as the only application UI.
- [x] Red: add a Rust regression that rejects production source references to browser UI assets; replace the old titlebar test with eframe rendering checks, which first failed because both native surfaces lacked the version label.
- [x] Green: remove `Room::document` and setup document assembly, retain the manual in the native viewer, add small version labels to Workspace and Agent Room, and remove obsolete DOM-only tests.
- [x] Verify: 39 tests across 10 focused native/UI suites pass; all-target compile, native-feature all-target Clippy (`-D warnings`), release build, rustfmt, and diff checks pass. Release binary scan found no browser-runtime or UI JavaScript markers.
- [ ] Complete: old unreferenced browser asset files and Rust tests that inspect those assets still need removal/replacement; the 300-line guard remains red on 25 files.

### Retire the oversized Agent Room script — issue #9 / DD-41
- [x] Plan: delete `src/ui/room.js`; retain quota/state decisions in Rust and use native Agent Room rendering.
- [x] Red: `native_ui_assets` first failed because the 367-line script remained; `source_layout` also reported it above the 300-line limit.
- [x] Green: remove the script and its static-only task-row, quota, routing, icon, idle-poll, and IME assertions; keep Rust quota, room-dispatch, snapshot, and native UI coverage.
- [x] Verify: 21 tests pass and 1 environment-dependent icon test is ignored across 8 suites; all-target check, native-feature Clippy (`-D warnings`), optimized release build, rustfmt, and diff checks pass. The release binary contains no legacy JS asset names or linked WebView engine.
- [ ] Complete: repository-wide `source_layout` now reports 24 remaining files above 300 lines. Other unused startup/MCP/manual browser assets and their static tests remain; real-device Korean IME and exact platform package acceptance remain open.

### Enforce the 300-line authored-file limit — R-154 / DD-164

- [x] Plan: trace the limit through requirements, business goal, feature, use case, quality attributes, design drivers, and design decision; update the global plan rules.
- [x] Red: record the initial layout-check failure on 2026-10-16, which identified 56 authored files above the new limit.
- [x] Green: retain the strict Rust source-layout check and use 300 lines as the repository-wide maximum.
- [x] Verify: `cargo test --offline --locked --test plan_rules --test source_layout` passes (3 tests); `git diff --check` passes.
- [x] Complete this requirement: the current source-layout check reports no authored files above 300 lines and no inline production tests.
- [ ] Complete issue #9: full regression, exact macOS/Windows package validation, and native-window visual/accessibility/IME/resource acceptance remain outstanding.

### Windows native renderer on hosted runners — issue #9

- [x] Red: exact-source Windows UI acceptance failed because the hosted runner exposes OpenGL 1.1, below eframe Glow's OpenGL 2.0 requirement.
- [x] Green: select eframe WGPU only on Windows and retain Glow on other targets; move the renderer regression into `tests/` to preserve test/source separation.
- [x] Verify locally: renderer and source-layout tests, complete all-target run except the renderer test's initial source-layout failure, native-feature Clippy, formatting, diff checks, and Windows-target dependency graph pass. The source-layout failure was corrected and retested independently.
- [ ] Verify remotely: rerun exact-source Windows build, native window/UI Automation, and package smoke on this commit; local Windows target compilation is unavailable because the Windows Rust standard library is not installed.
