# Bounded source and test modules

DD-41 traces R-35 through BG-05, SF-12, C-04, NFR-07, QAS-12 and DDR-04/DDR-06.
A compiled layout regression first failed on eight files over 500 lines and three
inline test bodies. Responsibility-based modules now preserve the existing public
paths and single-executable distribution. Production dependencies are unchanged.

Workflow separates execution, scheduler, permissions, approval, review, editing,
assignment and recovery. Process I/O separates control and platform containment.
Native shells separate Cocoa and WebView2; UI controller and startup dispatch have
separate modules. Private unit-test bodies use cfg/path modules under tests/unit/
without widening production APIs. Cooperative memory tests use a nested test module.
Under DD-160, the agent-tools-guide Python unittest contracts were ported to
`tests/agent_tools_guide.rs`; remaining Python contract modules are still scheduled for
incremental Rust conversion, so this is not evidence that the verification migration is done.
The MIT license and manifest contract is also in `tests/project_license.rs`; it checks
the installed-facing LICENSE, Cargo metadata and README link without invoking an interpreter.
The stable GitHub release workflow contract is now co-located with Rust release-note tests and
checks tag-only triggering, validation dependency, publish permissions, archives and pinned actions.
Native icon container, titlebar asset identity and Cocoa/Windows shell icon checks are covered by
`tests/app_brand.rs` without a Python or base64 runtime dependency.
UTF-8 contract checks for source text handling and embedded pages are in `tests/utf8_sources.rs`;
the Rust version inspects only the legacy Python files that remain during the migration.

The CI layout test checks source, test, script, example, style and markup files and
an optional build script. It rejects production test bodies. Maximum file: 485 lines.
Full macOS regression before path migration: 213 passed, 0 failed, 9 ignored across
57 suites. New canonical checkout's focused layout and format checks passed.
CI 38039813985 on 47eb2d7 completed successfully on native macOS and Windows, including build, layout, applicable tests, formatting and Clippy. This proves the module-split checkpoint, not full-product acceptance. Earlier CI 38038926444 passed Windows
tests but found two fixture pointer-argument lints, corrected in this checkpoint.

Evidence: `.agentconductor/verification/source-layout/regression.json` and
`source-manifest.json` (SHA-256 `7faf487cc8ab1f6b26b3666890af12854990f570c57d4850054ffcfb6128f232`).
[Design](../../docs/06-design-decisions.md), [layout test](../../tests/source_layout.rs),
[continuity](../../docs/12-development-continuity.md).
Graphify: 1,701 nodes / 3,420 edges, structural extraction without LLM. CocoIndex
structural API located continuation definition, recovery call and re-export with
no failed languages. Its MCP entrypoint lacks FastMCP in this host environment;
no dependency or daemon was installed. Ponytail has no callable capability in this
session; no plugin execution is claimed. Agentmemory: mem_mv25wmpa_f63ae0e2a551; native launcher split: mem_mvauuagn_a1ab837f01f4; poller split: mem_mvav68nr_0af980275ba7.


## Operator amendment — 2026-10-16

The maintained source/test ceiling is 300 lines. The canonical checkout is `/Users/neodamul/Development/AgentConductor`, branch `codex/issue-9-accessible-workspace-menu`, based on the latest `origin/main`. TDD red is recorded: the plan-rules test rejected the old 500-line default; the layout test first identified 57 files over 300 lines. Responsibility-based splits preserve public paths and behavior. The launcher split preserves app/probe/worker exports; the poller split preserves pending-state routing across settings, workspace refresh, and errors. The 88 native UI tests across 16 suites pass for those slices. Launcher, poller, review/recovery dispatch, and task-activity commits are already pushed. The layout regression caught `tests/i18n_contracts.rs` at 307 lines; moving its independent legacy-content assertion to `tests/i18n_legacy_compatibility.rs` reduced it to 294 lines and left the new file at 14 lines. It later caught `tests/native_ui_profiles.rs` at 326 lines; moving metadata-refresh and catalog-preservation tests to `tests/native_ui_profile_refresh.rs` reduced it to 293 lines and left the new file at 32 lines. The 315-line `tests/guard_policy.rs` has also been split: missing-workspace-file denial is now in `tests/guard_policy_missing_file.rs`, leaving 287 and 65 lines. All 24 tests across the six focused suites pass, along with native-feature all-target Clippy, formatting and diff checks. The 381-line `tests/native_ui_task_controls.rs` was split into task controls and timezone-aware scheduling suites; 6 tests across both suites pass, and the resulting files are 217 and 183 lines. The 335-line `tests/process.rs` was split into lifecycle and permission/protocol suites with a shared 40-line fixture; all 12 tests pass and the resulting files are 159, 146, and 40 lines. The 363-line `tests/release_package.rs` was split into assembly/failure and reviewed-license/notice suites with a shared 58-line fixture; all 7 tests pass and the resulting files are 104, 211, and 58 lines. The 366-line `tests/cooperative_worker.rs` was split into 74 lines of behavioral cases and a 297-line process harness in test support. All 7 tests pass. The 356-line `tests/cooperative.rs` now separates the company CLI packet integration check into `tests/cooperative_cli.rs` and shares setup/issue fixtures; all 12 tests pass and 1 remains ignored. The 370-line `tests/cooperative/memory.rs` now keeps the local-daemon round-trip test in a separate 98-line sibling module; all 11 non-ignored tests still pass and one remains explicitly ignored. The 452-line `tests/native_room_render.rs` now keeps stop/confirmation and candidate-decision cases in nested `tests/native_room_render/actions.rs`; all 12 tests pass and files are 293 and 163 lines. The 476-line `tests/native_ui_launch.rs` now keeps its launch/tool-decision action contracts in nested `tests/native_ui_launch/actions.rs`; all 12 tests pass and files are 295 and 185 lines. The 493-line `tests/native_ui_probe.rs` now puts persistence/profile/team and Room-probe cases in nested `tests/native_ui_probe/persistence.rs`; all 13 tests pass and files are 239 and 258 lines. The 355-line `src/workflow.rs` moved report/SQLite, Git-reference, notification, and run-knowledge persistence into `src/workflow/persistence.rs`, preserving `workflow::save` call paths; files are 297 and 65 lines, with 14 workflow-related tests passing. The current strict guard reports 39 remaining files above 300 lines, and still fails as expected. The implementation is not complete until it passes with no exceptions. Do not claim a completed refactor or merge before then.
