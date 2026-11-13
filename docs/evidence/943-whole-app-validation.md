# #943 — Phase automation and whole-application validation

## Scope and provenance

The implementation uses the supported desktop WebView and shared Rust backend; the retired alternate Rust UI is not reintroduced. Approved reference: Claude branch `claude/issue-943-phase-replan-mock`. Native tests run the actual Rust Startup API and subprocess workers against disposable repositories. Browser tests assemble the production UI with a simulated native bridge. They are distinct from the installed app observations below.

## Plan → task

Amend DD-62 and its requirement/Goal/Feature/use-case/QAS trace. Re-plan only the next Phase from the Goal, original analysis, remaining tasks, merged diff, tests and normalized final reports. Preserve the original index and later Phases. Compare all task execution fields, including order, models and options. Changed execution profiles require review even with identical task text.

Automatic merge is default-off, bound to the executed plan and the exact verified candidate. Existing scope, tool, permission, review, test and repository/evidence binding gates remain authoritative. Stop prevents automatic merge, re-planning and next-run launch; automatic approvals are distinct audit records. Keep slow repository work and Phase adoption in owned child processes, outside snapshot locks. Fixed human request revisions are preserved: resumed work hands automatic merge to a fresh job.

## Local verification

- Native three-Phase end-to-end: changed plan waits, unchanged plans continue, opt-in merges finish the Goal, auto-merge off waits for human merge, and Stop during re-planning prevents execution.
- Compare/adoption: additions/changes/removals, models/order, unsafe/reused IDs, original-index preservation and no edits to later Phases.
- Evidence: altered diff, test output or final report is refused. Stream full test-output hashes with bounded excerpts. Recover old receipts only from exact retained Goal-root links.
- Automatic gates: failed/blocked/stopped/restricted states, permission errors, blocked reviews, incomplete tests, Stop markers and stale evidence cannot merge. A paused chain starts no worker.
- Browser re-plan UI: change tags/reasons, read-only removed tasks, native option saving, edited task packet exclusion, English/Korean translation and unchanged-run information. Checked light/1280 and dark/840.
- Design comparison: initial full matrix ran 406 entries in each of four theme/width cells. Only three new tag font sizes/weights differed; corrected to the reference and all nine re-plan entries passed all four cells (36 checks).
- Whole Rust sweep excluding repeated design-matrix work: 971 passed, 17 ignored, two failed targets. Both failures expected superseded behavior (a duplicate reviewing-Phase box, and no next-Phase coordinator). Assertions were updated without weakening historical navigation or cancellation coverage. The complete Node target subsequently passed (all discovered scripts), the updated slug lifecycle passed, and final focused tests passed.
- Python configuration/package/UI contracts: 47 passed.

## Installed-app ad hoc (v0.2.8)

Read-only CUA checks: restored saved workspace/plan, Workspace and Goal sections, agent list and profile dialog (fallback agent/model and five-minute usage refresh), local-Rust signal settings, MCP server registration and OpenCode JSON guidance, Codex remaining percentages and reset timestamps. Closed dialogs without saving and returned to the original Plan screen. No real task execution, approval, external MCP fetch, settings save or user file deletion was performed.

Found collapsed rows wrapping long verification commands: registered #948 as Type Bug, milestone 0.2.9, separately from this feature. The known duplicate Phase box in installed v0.2.8 is already fixed by #944 and is not reported twice. Private workspace content and screenshots are not published.

## Final acceptance

Final full-UI rerun passed (297.91 s). Focused native re-run passed 12 tests across automatic merge (3), comparison/adoption (2), Phase runtime (5), slug lifecycle (1) and source layout (1). Clippy completed with warnings denied; formatting and diff checks passed. A production build passed; the exact final source is rebuilt before handoff. Windows packaged execution is not claimed by macOS-only local checks. CI is handed to Claude as one batch rather than dispatched per fix. The existing installed app is preserved.
