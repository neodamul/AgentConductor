# Native screenshot provenance

Status: visual acceptance remains open. Updated 2026-10-19 for issue #9.

The checked-in Workspace and Agent Room images are not evidence of the Rust renderer. The
operator identified the recent captures as the JavaScript UI. The old `*-native` filenames
and the previous public-doc test only checked names, not the process or binary that produced
the pixels. README and bilingual manual captions therefore no longer call those images Rust
native. Keep them out of native-migration acceptance and replace them before a release of
the migrated UI.

Current source evidence after the correction:

- The default Cargo feature is `native-ui-probe`, which opens the eframe Rust UI.
- The tracked application/test inventory contains no JavaScript, TypeScript or Python source;
  `src/ui/room.js` is absent and the 300-line Rust source-layout test passes.
- The actual local preview process was `/Users/neodamul/Development/AgentConductor/target/release/agentconductor ui-new /private/tmp/agentconductor-issue9-preview`.
- The rebuilt release binary, including the Workspace heading and localized empty-state hints,
  has SHA-256 `d12f7befcbf3b27500c0ef8e0d64e2f11f50dcf0fead215f3760a9b85581103a`.
- CUA reopened that release binary and visually inspected Workspace/Team in English and Korean,
  in light and dark themes. The rendered Workspace stage title appears once; empty name, folder,
  and goal fields have localized hints; an empty agent rail explains how to get started.
- The duplicate heading and empty-state guidance are covered by native UI regressions. Focused
  tests pass (22 across four integration targets), all-target Clippy and the release build pass.

The updated native app screenshot is saved under `docs/assets/ui-agent-settings-current.png`.
It was captured from the exact Rust process at `/private/tmp/ac-native-agent-screenshot/AgentConductor.app/Contents/MacOS/agentconductor ui-new /private/tmp/ac-native-agent-screenshot/workspace`, built from `HEAD` `dbd44aa` using `cargo build --release --locked` (binary SHA-256 `e3c37bb7ad0e02535a390bd21316cd406c25659b80bebd162627c7e63e634fd5`). The PNG is 1140×772 with SHA-256 `b9d68874b7a47aa3fdfa3fb40cdaf2faf03fbf6bdd09403bde15814e495ea866`; it shows the native Agent roles page with the agent-profile editor open. The app used an isolated HOME and workspace; no provider CLI, quota query, or user workspace was accessed. The README and both manuals reference this image, and both HTML manuals were regenerated with the Rust renderer.

The capture covers only the profile editor. Exact-build Workspace and Agent Room screenshots are still missing. Keep issue #9 at 0/25 until all packaged workflow checks pass; Windows UI and Korean IME acceptance remain separate gates.


The public-doc regression now rejects unsupported “Native Rust” captions. This checks truthful
documentation language; it does not prove image provenance.

### Exact packaged Workspace spot-check — 2026-10-19

The `7df4a93d5d347e7a80e523cd2826a170ba646c12` macOS package was launched once with an explicit
isolated `ui-new` workspace path. Its process command line was checked before CUA attached to the
same process. At 1140×772, the screenshot and accessibility tree showed the empty Workspace setup
fields and disabled Coordinator. No save action or user workspace data was involved. This is a
packaged empty-state check only; it does not replace the old public screenshots, validate provider
quota cards or Windows, or advance the 0/25 workflow acceptance. No screenshot file was retained.

The exact package's sidebar card was also visually inspected in a separate isolated run using a
Claude-parser fixture backed by `/usr/bin/true`. Its screenshot showed the quota refresh control and
compact layout but no provider quota value; the refresh required a trusted project folder before
launching a CLI. No screenshot bytes were retained for README/manual assets.
