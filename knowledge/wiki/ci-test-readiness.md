# CI test readiness

GitHub issue #156 exposed tests that advanced on a file marker alone or expected a child to publish its PID within a short startup window. Under runner load, the cooperative directive could arrive before the workflow persisted an active attempt and session, and a timeout could fire before a Windows child marker existed.

Use observable readiness: wait for every required fixture marker and the corresponding recorded workflow state before issuing a directive. For process tests, await a marker, process completion, or a bounded readiness deadline; allow the process supervisor to finish cleanup before reporting missing readiness. Keep the timeout behavior under test distinct from the readiness deadline.

The Windows process-tree test now uses a small compiled Rust fixture instead of PowerShell startup. It spawns its own descendant, records the PID, then waits, so the test isolates Job Object containment. Timeout process tests use unique fixture directories and wait for `ready`/`grandchild.pid` before asserting termination. Touched test files and helpers stay below 300 lines.

Local verification: 20 tests passed across five suites on three consecutive runs; warning-denied Clippy, rustfmt, source layout and diff checks passed. The Windows-specific test remains unverified locally on macOS and must pass the issue-linked Windows CI before #156 closes.

Trace: R-171 → BG-123 → SF-158 → UC-161 → FR-156/NFR-65 → QAS-167 → DDR-31 → DD-181. Evidence: `docs/evidence/issue156-ci-test-stability.json`.
