# Native startup workspace

Status: verified local fixture checkpoint; full-product acceptance remains open.

## Problem and decision

The native application previously required pre-existing CLI-created runs. DD-25 adds a private local profile registry, goal setup, read-only planning, three editable scoped tasks and exact plan consent through the existing direct native Rust message bridge. No UI HTTP server or new runtime dependency is introduced. Shared workspaces remain the default; worktrees are optional.

## Verified contracts

Registry values persist locally while private environment values are omitted from UI snapshots. Stale revisions, shell executables, unregistered replacement profiles and altered approval digests are rejected. Execution rechecks the approved repository baseline under its workspace lease before creating the run. Unused registered profiles do not block a plan. A canonical-path regression reproduces and fixes oversized settings leaving an orphan pending file.

Opening a run accepts no arbitrary path and acquires its window lease. Returning home releases that lease while retaining the original startup job broker for reaping. The native fixture exercised empty startup, custom worker/planner registration, goal entry, planning, editing all three test commands, explicit plan consent, automatic same-window run opening, diff/tests, candidate approval, final merge and returning home. Three independently identified fixture sessions and three exit-zero tests were collected. Repository HEAD equals candidate `128fc14b461f84d633e793c98a22eb8acc22c772`. Owned native process 54885 exited zero; a kernel probe confirmed absence. A rebuilt follow-up window (PID 85343) verified persisted setup and literal JSON typing after disabling smart quote substitution, then exited zero.

The complete suite passed 148 tests, with four ignored and zero failures, in 88.40 seconds. Final text-input changes were separately verified in the rebuilt native app. Clippy and formatting checks passed before those text-only edits; final checks remain part of the checkpoint closure. Raw local artifacts are retained in `.agentconductor/verification/ui-startup-live` and excluded from distribution.

## Limits

Fixture output is deterministic; it does not prove authenticated provider planning, entitlement, native tool containment or all-provider execution. External sandbox/read-only profile labels alone do not enforce OS isolation. Native dialogs were visually operable but their accessibility contents disappeared when opened after the page scrolled; accessibility remediation remains open. The public README contains actual runtime images only.

## Source hashes

- `src/ui/setup.rs`: `b72c19c7e86a4ddfdfedcc79e0224ba90c6370dd921fbfcd596e367f8dd91c28`
- `src/ui/startup.html`: `cf4f9d1443243f247e0a7069eb3b75a8b1c7880a7cfcc3f40634fa0361c352e4`
- `src/ui/startup.js`: `f9a1368688cf195b8ccb86508c86e2a5585540c28e787a07e317bdcc5ff8c95a`
- `src/ui/startup.css`: `171a4b10483c95319c30fe4a765f09ca9c702ea17a982289a62a667da461976f`
- `src/ui.rs`: `073d24ab6905096eb169f8273eabba91e4dc9a0f46fa0adc6429d52170e265fc`
- `src/workflow.rs`: `710a76d60725a94f039a6d2dccf47d549def06ed2daf12edbe44a753c1967e48`
- `src/planner.rs`: `6dc88d26829a8610f45e0a3d98ae6f72f25212d7fdcab021eb4a35c915785972`
- `tests/startup.rs`: `afb9a7f25662672a854fd9b284b1841c7d751e4251593cb3bacc071bc742dcfc`
- `docs/assets/ui-startup-native.jpg`: `a3faae975c8212d596c57b5826eebb406a36a6d58aa2524503f039108cee11c0`
