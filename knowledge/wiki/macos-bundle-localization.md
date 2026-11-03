# macOS bundle localization and LaunchServices

## Decision

When `Info.plist` declares `CFBundleDevelopmentRegion` and `CFBundleLocalizations`, the generated app bundle must include matching `.lproj/InfoPlist.strings` resources. AgentConductor supports English and Korean, so the Rust assembler writes `en.lproj` and `ko.lproj` with the bundle name and display name.

## Verification boundary

The macOS bundle contract test checks metadata and both resources. `plutil -lint` checks the generated plist and strings files. These checks prove package structure, not LaunchServices runtime acceptance or visible UI. Do not use display-name-only UI automation when another app bundle with the same identity may be selected.

## User data and platform limits

An earlier automation attempt selected the existing app and displayed saved profiles and workspace data. No control was activated; automatic quota refresh is unverified. Keep subsequent package tests isolated from the user's app data. The UTM Windows guest is currently unbootable with a blank display; leave it untouched and report Windows runtime verification as blocked.

## Source and evidence

- [Assembler](../../src/macos_bundle.rs)
- [Bundle contract](../../tests/macos_bundle.rs)
- [Raw investigation](../raw/2026-10-24-issue9-macos-localization.md)
- [Verification evidence](../../docs/evidence/issue9-macos-localization-20261024.json)
