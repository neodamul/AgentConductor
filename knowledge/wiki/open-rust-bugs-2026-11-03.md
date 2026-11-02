# Open Rust package bugs — 2026-11-03

The current open bug queue contains three issues whose Rust fixes are already in the root issue #9 branch. The source-level regressions pass locally, but exact Windows package behavior remains the acceptance gate. Keep each issue open until its current-head package journey runs and its persisted state is checked.

| Issue | Rust path | Local verification | Remaining package evidence |
| --- | --- | --- | --- |
| [#189 coordinator chat](https://github.com/neodamul/AgentConductor/issues/189) | Localized AccessKit name; UI Automation finds the visible, enabled chat editor; transcript persistence and reconnect logic | 11 tests across accessibility, Windows script contract, worker lifecycle, chat UI, and source layout | Windows package must pass persistence/reopen and reconnect journeys. Explicit Job Object breakaway skips do not count. |
| [#226 deleted workspace returns](https://github.com/neodamul/AgentConductor/issues/226) | Workspace archival, registry restoration, and default workspace launch | 15-test bug batch includes `launch_workspace`, `workspace_delete_default`, `workspace_registry`, and source layout | Verify the exact Windows package/update scenario and preserve the deleted/default workspace state. |
| [#310 schedule test omits run path](https://github.com/neodamul/AgentConductor/issues/310) | Forward `AC_RUN_DIR` to the Windows UI driver and assert the matching run report | 15-test bug batch includes schedule dispatch/marker, stop/resume, and source layout | Verify the exact Windows packaged scheduling journey persists the requested `available_at` and `manual_schedule` values. |

Current root PR #22 head when checked: `397069af17d33a6296397ace706016907e065579`. Platform Validation run `41500210315` was in progress on macOS and Windows. This local evidence does not increase the 22/25 packaged workflow count and does not justify closing the three bugs.
