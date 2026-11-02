# Issue #226 Windows package regression

- Source finding: Rust `workspaces::launch_root` already selects the most recently saved workspace after the default workspace is deleted; the source integration test covers that selection.
- Gap: issue #226 remained open because that source test did not prove the packaged Windows executable uses the same persisted workspace registry after a user updates the app.
- Test design: create and save the default workspace, create a surviving workspace, delete the default, then hold a kernel lease on the survivor while launching the exact packaged binary. The package passes only if it attempts to open the survivor, does not recreate the default, and stays alive until the assertion observes its lease contention.
- TDD: the workflow inclusion contract failed before the Windows package test was added to the package job. The contract, existing source launch regression, source layout, formatting, and warning-denied focused Clippy now pass locally. Windows package execution remains pending; keep #226 open.
- Relevant files: `tests/windows_workspace_reopen.rs`, `.github/workflows/platform-validation.yml`, `docs/07-implementation-plan.md`.
