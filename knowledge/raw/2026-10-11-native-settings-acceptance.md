# Native settings acceptance — 2026-10-11

R-67 → BG-25 → SF-55 → UC-55 → FR-51/NFR-25 → QAS-60 → DD-73.

Captured from the actual Rust release binary running in system WebKit, without a UI server or mocked responses. The temporary macOS wrapper uses the same executable bytes and the project AC icon. The master icon is project-owned artwork under the root MIT license.

Accepted observations: installed Claude path /opt/homebrew/bin/claude appears automatically in Edit; the interval is 5 minutes; single-line controls are 40px and paired columns have equal widths. Workspace step labels match the operator request. Claude and Codex account capacity were visible from live authenticated metadata, not fixture values. Quota job loading/completion messages are suppressed. The operator confirmed the OS file picker opened outside the captured application window; an application-scoped capture alone could not establish its visibility. The proposed picker replacement was reverted. No prepared execution plan was approved or run for screenshots.

Images show current operator-entered workspace values and product settings, without exposing the internal task instructions.

- `docs/assets/ui-workspace-current.png` SHA-256: `3728737fb763553c1376f94587de39ec2a963578565a2de79eb117e0fa07cdc0`
- `docs/assets/ui-agent-settings-current.png` SHA-256: `aa3879cafd4a36bf6b50feb6610c17c2145498bf4fe718745f36ef75af0700bf`
- `src/ui/assets/ac.png` SHA-256: `93d5ac2cff88bc00ad7a4116ea69b25fa101e3cb394a7ebc167ff80ad115a44e`

Release binary SHA-256: `3cf6e4232213523dc59ee607b70ac6d6479554a9f97f6012c62160f9a3b0a32d`.

Windows build/resource and visual checks remain unavailable: GitHub run 38117042858 was refused before starting due recent account payment failures or a spending limit. This is not a successful platform verification. Stable publication begins at 0.1.0, followed by 0.1.1; the publish workflow no longer passes --prerelease. No stable release was published in this acceptance step.
