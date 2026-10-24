# Native Room failure feedback

The generated instruction for a failed task follows the Room's selected interface language. Captured provider output, test output, and task errors stay verbatim so translated UI does not alter diagnostic evidence.

The native status banner passes the visible Room language to the task-feedback helper. The English and Korean catalogs provide the generated instruction and empty-error fallbacks. `tests/native_run_status_banner.rs` verifies the rendered output in both languages.

## Trace and verification

- Requirement/design trace: R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76.
- Focused Room status, i18n, and language tests pass; source-layout, public-docs, formatting, and warning-denied all-target Clippy pass.
- A broader native all-target run passed 258 tests (4 ignored) across 79 suites, then was stopped while the coordinator suite was still running. Full-suite completion is unverified.
- This component correction does not increment packaged-workflow or Rust UI replacement acceptance.
