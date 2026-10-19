# Native screenshot provenance

Status: the screenshots below are verified against the exact Rust Workspace window captured on
2026-10-19. They document this UI surface only; native workflow parity remains 0/25.

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
