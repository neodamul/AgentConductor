# Native screenshot provenance

Status: the screenshots below are verified against the exact Rust Workspace window captured on
2026-10-19. They document this UI surface only; native workflow parity remains 0/25. They are
not current-main screenshots and must not be cited as current-main visual parity evidence.

The earlier Workspace images are superseded. They showed the workspace-sharing panel even though
the current Rust app exposes that feature from Settings. The current captures show the Workspace
form without that panel. Settings → Workspace settings sharing opens the separate Export/Import
window. No settings were saved during inspection.

## Current Workspace captures

All three images are 1140×772 PNG captures from CUA's screenshot API while attached to the exact
Rust process built from clean `codex/issue-9-native-ui-parity` HEAD `392aa20`. The binary SHA-256
was `c4eb698b307ed3900dcd4c5bc37d56f08aa91cf920697d7454818b45be6a385f`.

- `docs/assets/ui-workspace-current.png` — English, light theme; SHA-256
  `27a77fa35456427261bf214ca1828df92e989a7d163fa7fbfc0ece3d7531e2c6`.
- `docs/assets/ui-workspace-korean-current.png` — Korean, light theme; SHA-256
  `a6084f81585d81140b2c30d5dc7ed8f7bad300577cd0db84a92117cfa14bd78e`.
- `docs/assets/ui-workspace-dark-current.png` — Korean, dark theme; SHA-256
  `d8936c6aad3299b5a4e8310f9fa34fbf2f261b597a2e8e75ebcef529130aaeb7`.

The window was assembled with the Rust `bundle-macos` command and run with a disposable HOME and
workspace. CUA's live accessibility tree and screenshot were checked on the same process. Its
bundle identifier was `dev.agentconductor.desktop`. The Settings sharing window was opened to
verify the Export/Import route. The app was then closed and its isolated data removed. The host's
`screencapture -l` could not capture the window; CUA's screenshot API succeeded instead.

README and both offline HTML manuals reference the refreshed Workspace assets. Both manuals were
regenerated with the Rust renderer. This does not establish complete native parity: exact Windows
UI, accessibility/IME, packaged workflows, and all 25 workflow acceptance checks remain open.

## Other Rust captures

The Agent Roles screenshot `docs/assets/ui-agent-settings-current.png` was captured from the exact
Rust process at `/private/tmp/ac-native-agent-screenshot/AgentConductor.app/Contents/MacOS/agentconductor ui-new /private/tmp/ac-native-agent-screenshot/workspace`, built from `dbd44aa`. Its PNG SHA-256 is
`b9d68874b7a47aa3fdfa3fb40cdaf2faf03fbf6bdd09403bde15814e495ea866`. It shows the profile editor;
no provider CLI, quota query, or user workspace was accessed.

The Agent Room image `docs/assets/ui-agent-room-native.jpg` was captured at 1140×772 from the Rust
binary built from `dbd44aa`; JPEG SHA-256 is
`45a11761b25c3e8465dfad67e5adff8794ba7067a6f7dc387cda0e1809380584`. It depicts a prepared UI
demonstration with a passing verification result. No agent/provider CLI ran, and no merge candidate
was created. It is not evidence of end-to-end execution.

The old status-light screenshot was removed from public and in-app manuals because its source
process was unrecorded. Do not restore it until running, waiting, and failed states are captured
from a verified Rust build.

## 2026-10-20 screenshot refresh audit

The current Workspace image hash matches the Rust-native capture from commit `392aa20`; its visible `0.1.24` label is historical, not evidence of WebView. The open desktop window is Rust v0.1.30 from package commit `fd177ea`, but the latest pushed issue-9 head changes 139 lines across 13 native UI files after that package. The open window also contains a populated workspace and profiles, so do not use it for public documentation. Refresh screenshots only from a clean isolated window built from the exact current HEAD; the current README/manual images remain authentic Rust captures but are not exact-current evidence.

## 2026-10-21 exact-package UI check

The v0.1.32 Rust package from `cf92f39` was opened in one window. The Workspace view showed the full-width numbered stage row, active underline, and lower-right primary save action. No workspace fields were changed or saved, and the screenshot was not retained as a reusable asset. This confirms that one Rust surface renders; it does not prove parity with a current-main screenshot or add packaged workflow acceptance.

## 2026-10-21 current issue-9 worktree UI check

The debug executable from `codex/issue-9-native-ui-parity` was opened in a temporary app bundle, with an isolated HOME and one app process. CUA screenshots showed Workspace settings, Agent Roles, and Review the plan. A temporary custom `/usr/bin/true` profile was registered, a workspace was saved against an empty non-Git project folder with a separate run-artifact root, and that profile was assigned to development. No provider or fixture CLI ran; plan generation was not invoked. The test interaction is recorded in `docs/evidence/issue9-f7584ed-rust-ui-spotcheck.json`. The images were displayed in-session but not saved as repository assets; this debug fixture check does not increase packaged UI acceptance (8/25).

## 2026-10-21 current packaged Rust UI screenshot

Opened the exact-source macOS package from `codex/issue-9-native-ui-parity` commit `34ca477` in one app window. The displayed Workspace settings screen belongs to the packaged Rust executable, not a retained screenshot asset; its isolated profile contains only a temporary `test-agent` and a `/private/tmp` workspace. No provider or fixture CLI ran. Package smoke and archive checks pass, but this single rendered screen is not a complete packaged workflow and does not change issue #9 acceptance (8/25). Package provenance: [evidence](../../docs/evidence/issue9-34ca477-macos-package-ui.json).

The same packaged window also showed Agent Roles and Review the plan. Saving the current workspace settings persisted its name, goal, shared-folder mode, and plain non-Git project path to the isolated run's `startup.json`. The `Saved Workspaces` pane still read `No saved workspaces`; this visual check records the observed state but does not treat that pane as part of the save assertion. All screenshots were displayed during the session and remain unretained as files.
