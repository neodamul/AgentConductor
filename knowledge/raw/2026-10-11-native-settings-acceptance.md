# Native settings acceptance — 2026-10-11

R-67 → BG-25 → SF-55 → UC-55 → FR-51/NFR-25 → QAS-60 → DD-73.

Captured from the actual Rust release binary running in system WebKit, without a UI server or mocked responses. The temporary macOS wrapper uses the same executable bytes and the project AC icon. The master icon is project-owned artwork under the root MIT license.

Accepted observations: installed Claude path /opt/homebrew/bin/claude appears automatically in Edit; the interval is 5 minutes; single-line controls are 40px and paired columns have equal widths. Workspace step labels match the operator request. Claude and Codex account capacity were visible from live authenticated metadata, not fixture values. Quota job loading/completion messages are suppressed. The operator confirmed the OS file picker opened outside the captured application window; an application-scoped capture alone could not establish its visibility. The proposed picker replacement was reverted. No prepared execution plan was approved or run for screenshots.

Images show current operator-entered workspace values and product settings, without exposing the internal task instructions.

- `docs/assets/ui-workspace-current.png` SHA-256: `3728737fb763553c1376f94587de39ec2a963578565a2de79eb117e0fa07cdc0`
- `docs/assets/ui-agent-settings-current.png` SHA-256: `aa3879cafd4a36bf6b50feb6610c17c2145498bf4fe718745f36ef75af0700bf`
- `src/ui/assets/ac.png` SHA-256: `93d5ac2cff88bc00ad7a4116ea69b25fa101e3cb394a7ebc167ff80ad115a44e`

Screenshot-source release binary SHA-256: `3cf6e4232213523dc59ee607b70ac6d6479554a9f97f6012c62160f9a3b0a32d`.

Windows build/resource and visual checks remain unavailable: GitHub run 38117042858 was refused before starting due recent account payment failures or a spending limit. This is not a successful platform verification. Stable publication begins at 0.1.0, followed by 0.1.1; the publish workflow no longer passes --prerelease. No stable release was published in this acceptance step.

Final integrated macOS binary SHA-256: `d314a705141c28925d51b81b6bd134c76c37efa68a4439bed246af74bc1082d5`. Runtime replacement was confirmed at PID 1371 with live Claude/Codex capacity and Ready status. Exactly one owned app executable was observed.

Package source: `d4bd212fb44284db12532d5d206d1dbc022bc1db`; aarch64 macOS archive SHA-256: `860f0ebe3d2cd59c56391435edbe48300aa9adeec7f1f13c33a8a90a5ffb7287`. The 5,979,456-byte executable includes bundled SQLite and reviewed notices for 42 dependencies. Packaged smoke passed both shared-directory and Git-worktree modes: three concurrent fixture CLI tasks, three tests, unapproved merge refused, approved merge verified. These are deterministic local CLI fixtures, not an assertion of all paid-provider behaviors.

The final merge of R-66 empty-workspace defaults exposed a quota test fixture assuming implicit Codex registration. Its existing provider template registration helper now establishes that precondition explicitly; focused startup_quota tests passed 4/4 after the change. No product behavior was changed by this test repair.

Final all-target regression: 293 passed, 0 failed, 12 ignored across 89 suites. The ignored provider/OS checks are not represented as passes. Python verification: 22 passed. Formatting, Clippy with warnings denied, source/test line bounds, JavaScript syntax and optimized macOS build passed. Full regression log: `/private/tmp/agentconductor-final-regression.log`; packaged smoke log: `/private/tmp/agentconductor-final-package.log`.
