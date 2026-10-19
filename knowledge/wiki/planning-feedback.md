# Windows picker and planning feedback

Windows PowerShell writes picker output in the OEM code page and may add a UTF-8 BOM, so a picked Korean path broke while a typed one worked. The owner script forces UTF-8 without BOM and `picked()` strips a leading BOM (DD-90).

Generate plan copies an ordinary folder before replying; the 15 s bridge default produced a false timeout. Planning-entry actions now wait up to 300 s. The review area reports the coordinator's real output (bytes and last write of its stdout/stderr) and warns after two minutes of silence; reduced motion keeps an opacity-only blink (DD-91).

Trace: R-82, R-83, DD-90, DD-91, QAS-78, QAS-79. Evidence: tests/picker_output.rs, tests/windows_console.rs, tests/planning_progress.rs, tests/test_planning_feedback.py.

## Native failed-plan retry — issue #36

The Rust Review page treats `Prepare` and the snapshot requested after its reply as one in-flight transition. A distinct `PrepareRefresh` state prevents the old failed snapshot from being presented as the current attempt; localized preparation feedback remains visible until the authoritative snapshot arrives. A timed-out dispatch keeps its current error in the shared feedback rail and makes the explicit Retry plan generation action available from Review when workspace settings are current and task edits are clean. The action dispatches `prepare`, not `snapshot`, and does not start an automatic retry. Regression coverage is in `tests/native_ui_probe/review_width.rs` and `tests/native_ui_review.rs`.

Trace: R-155, DD-165, QAS-151. Evidence: issue branch `codex/rust-issue-36-plan-retry`; PR #37 merged to `main` as `aa5606b` and issue #36 closed. Rust parity port: `ce1e9ab` on `codex/issue-9-native-ui-parity`; focused review tests and the full local suite pass.
