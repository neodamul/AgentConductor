# CI test readiness

GitHub issue #156 exposed tests that advanced on a file marker alone or expected a child to publish its PID within a short startup window. Under runner load, the cooperative directive could arrive before the workflow persisted an active attempt and session, and a timeout could fire before a Windows child marker existed.

Use observable readiness: wait for every required fixture marker and the corresponding recorded workflow state before issuing a directive. For process tests, await a marker, process completion, or a bounded readiness deadline; allow the process supervisor to finish cleanup before reporting missing readiness. Keep the timeout behavior under test distinct from the readiness deadline.

The Windows process-tree test now uses a small compiled Rust fixture instead of PowerShell startup. It spawns its own descendant, records the PID, then waits, so the test isolates Job Object containment. Timeout process tests use unique fixture directories and wait for `ready`/`grandchild.pid` before asserting termination. Touched test files and helpers stay below 300 lines.

Main verification: 20 tests passed across five suites on three consecutive runs; macOS and Windows validation and release-package CI both passed. On the issue #9 Rust branch, 19 tests passed across five focused suites with `test-fixtures`, and warning-denied Clippy, rustfmt, source layout and diff checks passed. The Windows-only containment test was executed successfully in Windows CI; it cannot run on this macOS host.

Trace: R-172 → BG-127 → SF-160 → UC-163 → FR-158/NFR-68 → QAS-169 → DDR-32 → DD-183. Evidence: `docs/evidence/issue156-ci-test-stability.json`.

2026-10-23 follow-up: on the Rust issue #9 branch, locked/offline serial runs of `cooperative_worker`, `process_timeout`, and `windows_containment` all pass on macOS. This revalidates the focused source suites after later Rust ports; it does not replace Windows execution or UI acceptance. The local machine has no installed Windows Rust standard library or MinGW linker. A hidden disposable UTM Windows boot was attempted; the VM's QEMU guest agent was unavailable, so `utmctl exec` and IP discovery failed. The VM was stopped and its temporary window closed without claiming Windows validation.
