# Issue #9 duplicate Agent Room branding — 2026-10-23

## Change

The shared navigation shell already displays the application mark, name, and version. The Agent Room window bar duplicated those labels. Removed only the duplicate branding from the Room while preserving its workspace path, phase, authentication state, and action controls.

## TDD and verification

- `native_ui_room_header` regressions were updated first and failed against the old rendered screen because the Room still exposed `AC`, `AgentConductor`, and the package version.
- The implementation then removed those duplicate labels.
- Focused Room-header and titlebar tests passed: 3 tests.
- Warning-denied library Clippy and targeted test Clippy passed.
- `cargo fmt --check` and `git diff --check` passed.
- Release package built from commit `6f1111c5ae61c91933e03a20fb04a4df513193de`.
- Package: `dist/agentconductor-aarch64-apple-darwin-6f1111c5ae61.zip`.
- Package SHA-256: `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`.

## Open verification

The package binary launched, but the desktop automation bound to the main Workspace screen instead of the Agent Room window. No post-fix Agent Room screenshot was captured; do not count packaged visual acceptance or claim the issue is complete. The disposable fixture remains `waiting_approval`; no approval or merge was created. Windows UI and exact-package rendering remain unverified.
