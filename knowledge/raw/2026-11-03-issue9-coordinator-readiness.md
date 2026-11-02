# Workspace coordinator readiness

- Trace: R-206 → UC-197/UC-198 → DD-218 → issue #9.
- The Workspace setup view now shows the coordinator's observed-model selector alongside the coordinator selector. The existing workspace save request persists the model on its AgentProfile; selecting the CLI default removes the optional override.
- Before chat is opened, Workspace checks the currently selected coordinator profile with the same read-only argument validator used by chat dispatch. If the CLI cannot be invoked safely in read-only mode, a localized warning with the reason is displayed inline. The normal write-capable command is never used as a fallback.
- Red/Green coverage: `tests/native_ui_probe/workspace_model.rs` selects an observed model through egui pointer interaction and verifies persistence; it also verifies the compatibility warning is present before chat opens.
- Local verification on 2026-11-03: all 113 library tests and all-target integration suites passed; the 300-line source/test layout test passed; all-target Clippy with warnings denied, rustfmt, and diff checks passed. Tests requiring authenticated vendor CLIs or the exact packaged Windows executable remain intentionally ignored by the local suite.
- The change is on the existing issue #9 branch and PR #22. It adds no separate PR and does not change issue acceptance: 22/50 overall, with UI parity still 0/25.
