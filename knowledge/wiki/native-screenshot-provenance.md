# Native screenshot provenance

Status: the English, Korean light, and Korean dark Workspace captures are current source-build
screenshots from `a75152d028ec2731be2736e7f2f92e011fde9de2`. The English capture is suitable for the
README and English manual; the Korean captures are used by the Korean manual. All three show an
empty Workspace without agent or demo-task data. They are documentation assets, not packaged-workflow
parity evidence. The Agent Room manuals now use matching English and Korean source-build captures;
exact-package verification remains open.

The earlier Workspace images are superseded. They showed the workspace-sharing panel even though
the current Rust app exposes that feature from Settings. The current captures show the Workspace
form without that panel. Settings → Workspace settings sharing opens the separate Export/Import
window. No settings were saved during inspection.

## Historical Workspace captures — superseded 2026-10-23

All three images are 1140×772 PNG captures from CUA's screenshot API while attached to the exact
Rust process built from clean `codex/issue-9-native-ui-parity` HEAD `392aa20`. The binary SHA-256
was `c4eb698b307ed3900dcd4c5bc37d56f08aa91cf920697d7454818b45be6a385f`.

- Former `docs/assets/ui-workspace-current.png` — English, light theme; replaced by the 2026-10-23 blank Workspace capture. Former SHA-256 `27a77fa35456427261bf214ca1828df92e989a7d163fa7fbfc0ece3d7531e2c6`.
- `docs/assets/ui-workspace-korean-current.png` — Korean, light theme; SHA-256
  `a6084f81585d81140b2c30d5dc7ed8f7bad300577cd0db84a92117cfa14bd78e`.
- `docs/assets/ui-workspace-dark-current.png` — Korean, dark theme; SHA-256
  `d8936c6aad3299b5a4e8310f9fa34fbf2f261b597a2e8e75ebcef529130aaeb7`.

## 2026-10-23 current Korean Workspace captures

Both images were rendered by the opt-in `manual-screenshot-capture` Rust binary built offline in
release mode from exact issue-9 HEAD `a75152d028ec2731be2736e7f2f92e011fde9de2` (binary SHA-256
`fbb0d14358c60a12da3a904d49b105f652d646df82d330da0fac3034b66c44b7`). Each run used an isolated
HOME, an empty default Workspace, Korean language, no registered agents, and the requested theme.
The eframe framebuffer callback closed the probe after its first frame. Both captures are 1140×740:

- `docs/assets/ui-workspace-korean-current.png` — light; SHA-256
  `662a32c0665c3b7fd12b92f2364f5ad2e634c4e0fe2aa0eeddb48a9f4ddbd442`.
- `docs/assets/ui-workspace-dark-current.png` — dark; SHA-256
  `a7b30513aa14ecd57be6aedf0fce499471dbed7a97610bfd595be4047eeb8ec1`.

The app did not start a provider CLI or MCP server. The automatic update lookup was not instrumented.
The Rust manual renderer regenerated both HTML manuals after the asset refresh. Evidence:
`docs/evidence/issue9-korean-workspace-screenshots-20261023.json`.

## 2026-10-23 current bilingual Agent Room captures

The English and Korean screenshots were captured by the opt-in Rust probe at source commit
`3b88f277b90db786d1c3e650cc9361e776bad8d2` plus worktree source changes with SHA-256
`06c7af3c168b8451b2447d61b7b35a1914ada73e80217c72376f91742fcd7f09`. The release-profile binary
SHA-256 is `9ccc5619e0e1f98ad7f7cea86ac85a1030c9163ed52149cec5d71209bdab51c5`. Both images are
1140×960 light-theme captures from one isolated deterministic run: three tasks completed, three
tests passed, and candidate `b744fbfd2a859d13e45d8571fe7b4c361c8ed045` remained `waiting_approval`.
The candidate was not approved or merged. The Korean screenshot shows the localized `결과` label.

- `docs/assets/ui-agent-room-native.png` — English; SHA-256
  `d3ed1422f21d0a1ee2f821cd2beb98b80cb200f37e29aace5ef611f8e25dd50d`.
- `docs/assets/ui-agent-room-native-ko.png` — Korean; SHA-256
  `133b480123de5431de6effb24606149878e85b41ef1e47f03cc4ee014a6a8ee9`.

Both Markdown manuals and generated HTML manuals reference their matching language image; the
native manual viewer embeds both PNGs. Regression tests compare generated HTML with checked-in
output. The fixture used only a local Rust test executable; no provider CLI or MCP ran, and the
automatic update lookup was not instrumented. The disposable repository, isolated HOME, and run
were removed. This documentation refresh adds no packaged workflow acceptance. Evidence:
`docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.

The window was assembled with the Rust `bundle-macos` command and run with a disposable HOME and
workspace. CUA's live accessibility tree and screenshot were checked on the same process. Its
bundle identifier was `dev.agentconductor.desktop`. The Settings sharing window was opened to
verify the Export/Import route. The app was then closed and its isolated data removed. The host's
`screencapture -l` could not capture the window; CUA's screenshot API succeeded instead.

README and both offline HTML manuals reference the refreshed Workspace assets. Both manuals were
regenerated with the Rust renderer. This does not establish complete native parity: exact Windows
UI, accessibility/IME, packaged workflows, and all 25 workflow acceptance checks remain open.

## 2026-10-23 current README Workspace capture

The screenshot was captured by the opt-in `manual-screenshot-capture` feature from a v0.1.36
release-profile binary built from issue-9 HEAD `09f28db80576ebcc58cacc2da91f03a81642fc68` plus
capture-only changes to `src/native_ui_probe.rs` and `src/native_ui_probe/app.rs` (source diff
SHA-256 `bc87da0ba1365088f521b228edbd9b0ff8bc85ea10f02a5b27051235f567a87c`). Binary SHA-256 is
`a5669454b366ead3fa8a9ed3dec805de97d16fa28859027933d718c29a2ea813`. The Rust app opened a
disposable HOME and empty `ui-new` workspace; the first eframe frame wrote
`docs/assets/ui-workspace-current.png`, 1140×740, SHA-256
`824044b44a29920ded08631b649cde53d4634ef6ed3606be1c2c5649a9d34abf`. The image was inspected
visually: it has blank workspace fields and no registered agents, task fixtures, or private paths.
No provider CLI or MCP server ran; the automatic release lookup was not instrumented. README and
the English offline HTML manual now reference the image. Evidence: `docs/evidence/issue9-readme-workspace-screenshot-20261023.json`.

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

## 2026-10-23 stale public-screenshot audit

At the time of this audit, the README's former image embeds showed v0.1.24 Workspace and Agent Roles
screens, v0.1.21 Agent Room screens, and a prepared Agent Room demo rather than the exact current
package. All README image embeds were removed until a current capture was available. The English
and Korean manuals also referenced historical screenshots then; the source-build captures above
later replaced them. Exact-package/manual verification remains open.

## 2026-10-23 exact current Agent Room verification

The v0.1.36 macOS arm64 package built from `6200e49b2d9f7cef5f11c56e96adeb6c840ed234` ran a
disposable three-task Rust fixture. Its Agent Room showed 3/3 tasks completed, 3/3 task tests
passing, and the candidate waiting for approval. No approval or merge occurred. CUA displayed the
current screen. `CGWindowList` identified the 1140×772 app window as ID 10535; `screencapture -x
-l 10535` still failed with `could not create image from window`. Its `kCGWindowSharingState` was
0; that observation does not establish the cause. No image file was retained. Evidence:
`docs/evidence/issue9-6200e49b-current-agent-room-20261023.json`.
This confirms the current packaged screen renders but does not refresh public manuals or increase
the 14/25 workflow acceptance count.

Release packaging previously discovered image assets only through README links. Removing those
links exposed a regression: offline manuals lost their referenced images in the release archive.
`src/release_package.rs` now independently gathers and validates local image links from both
manuals. The release-package test proves a manual-only image is included while an unused mock
image is excluded. AgentMemory record: `mem_mvk6p1gq_83dadc591c2f`.


## 2026-10-23 prior retained Agent Room JPG — superseded

The then-current Agent Room JPG was rendered by the Rust app's
eframe framebuffer callback from source commit `25556157`. The callback is available only with
the opt-in `manual-screenshot-capture` feature and `AGENTCONDUCTOR_MANUAL_SCREENSHOT_TO`; the
normal release build does not enable it. Capture waited for the disposable run snapshot, then
closed the viewport after saving the PNG. The committed manual asset is the visually checked
1140×960 JPEG `docs/assets/ui-agent-room-native.jpg`, SHA-256
`f65b84b8dd37090d2ead0ce44428f72ac4f791fb8fac2945653e940a1f53053b`. It shows three completed
tasks, three passing task tests, and a candidate waiting for human approval. No provider CLI ran,
and no approval or merge occurred. This single-language JPG was later replaced by the English and
Korean PNGs described above. This source-built fixture image is not exact-release-package evidence
and adds no packaged workflow acceptance.

## 2026-10-24 current v0.1.37 Workspace documentation captures

Built `agentconductor-native-ui-probe` from the current Rust branch using the opt-in `manual-screenshot-capture` feature. The framebuffer callback captured the exact current Workspace view at 1140×740 and closed each temporary window. Each run used a fresh HOME under `/private/tmp` with no profiles, user workspaces, goals, provider CLI credentials, or MCP servers. The first-frame updater may perform only the public unauthenticated latest-release GET at `https://api.github.com/repos/neodamul/AgentConductor/releases/latest`. English light replaces `docs/assets/ui-workspace-current.png` (SHA-256 `7e01d71a1f365e0c450928c0b149523716cf57ad2532bb7a9ddbd2c0d31292f8`); Korean light is `docs/assets/ui-workspace-korean-current.png` (`709a6c9b2ea3ff00118598b123fa60216ba7127c10750db764e0ee08867f47d2`); Korean dark is `docs/assets/ui-workspace-dark-current.png` (`caa351cd89a56faa0e9a1e92178140f1cd3974cfa5f166469d784852809ff3ab`). README and the bilingual manuals identify v0.1.37; HTML was regenerated with the Rust manual renderer. These are isolated source-build documentation captures, not exact release-package or Windows evidence, and add no workflow acceptance credit. See `docs/evidence/issue9-current-workspace-docs-20261024.json`.
