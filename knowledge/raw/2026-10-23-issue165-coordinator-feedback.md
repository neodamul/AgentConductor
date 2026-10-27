# GitHub #165 coordinator feedback verification

Date: 2026-10-23

The Rust branch now has a coordinator-specific request action and draft in the native Agent Room. The path is UI → revision-bound Rust action → local `coordinator-request` subprocess → existing frozen-evidence reviewer → deterministic amendment/charter validation → audit-derived coordinator judgments. Task feedback remains unchanged. A request is bounded to 8 KiB, includes the complete bounded operator message in the audit, and never grants approval or merge authority.

TDD evidence:

- Red: focused workflow regression failed because `review_request` was not exported.
- Green: lifecycle tests verify queued/processing/completed, visible failure, retained task attempts, and refusal to merge.
- Render regression verifies the named recipient, processing status, and coordinator send control.
- Validation rejects blank, NUL-containing, and over-limit requests.
- Full local run after the lease-wait refinement: 920 passed, 14 ignored, 0 failed, 310 suites, serial execution, 369.64 seconds.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`, and the 300-line source layout test passed.
- A settings-dialog visibility test failed once in the parallel full-suite run, then passed alone and in the final serial full suite.
- The running local v0.1.36 app was not replaced. Packaged UI and Windows end-to-end checks are still outstanding for issue #9.

## Follow-up: coordinator room-lease race

Claude's review found a race after the initial room-lease check: the room can reacquire its lease before the review worker reads the snapshot. The coordinator request now retries only errors containing `lease is held`, sleeping 100 ms between attempts and sharing the original bounded request deadline. Other review errors still fail immediately; timeout remains a recorded failed request.

- Red/green tests cover a transient lease conflict that later succeeds, immediate propagation of unrelated errors, and deadline expiry after retries.
- Focused verification: the 3 retry tests, 3 coordinator workflow tests, 2 source-layout tests, and the 2-test CLI workflow suite passed. Warning-denied all-target Clippy, rustfmt, and `git diff --check` passed.
- Full-suite verification: `cargo test --locked --offline -- --test-threads=1` passed 923 tests, 14 ignored, 0 failed across 310 suites in 367.20 seconds. The first attempt had one transient `DirectoryNotEmpty` cleanup failure in `binary_schedules_reassigns_and_selects_models_with_native_commands`; that test and its full suite passed when run alone, and the final complete run passed.
- Exact packaged UI at commit `b298ae79774e` rendered the recorded `fixture-planner` recipient and coordinator request draft. The send control was disabled for empty input, enabled for a draft, and disabled again after clearing. Separately, exact packaged CLI dispatch was exercised against a copied run with a deterministic read-only Rust reviewer: request `rq-7c80bcc153` completed with an evidence-backed accept response, and the copied candidate remained `waiting_approval`, with all three tasks/tests complete and no approval or merge. Three earlier fixture setup attempts failed before this successful request; they did not affect the product candidate. The successful dispatch lifecycle was confirmed in the copied run's SQLite audit and was not rendered in the UI. Windows UI checks remain open issue #9 gates. See `docs/evidence/issue9-b298ae7-coordinator-dispatch.json` for package and run details.
- Render follow-up: the coordinator input panel intentionally shows the newest request's status; request history is shown in the Activity view. A new render regression covers queued, processing, completed, and failed as the newest status. The coordinator feedback and judgment render suites pass 10 tests total. This verifies status rendering for fixture snapshots, not a lifecycle dispatched and observed through the active packaged UI.
- Exact packaged UI follow-up: opened the v0.1.36 package at `b298ae79774e` on a copied run in one window, entered a coordinator request, and clicked Send. The UI visibly showed queued → processing → completed and the deterministic review response. The copy stayed `waiting_approval`, with 3/3 tasks/tests complete, attempts unchanged, and no human approval or merge. The original run's audit still has zero coordinator-request rows. No provider CLI or external service was called. Evidence: `docs/evidence/issue9-b298ae7-packaged-ui-coordinator-request.json`. This is distinct packaged workflow 10/25; Windows UI remains open.
