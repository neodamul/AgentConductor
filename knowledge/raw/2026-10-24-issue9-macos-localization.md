# Issue #9 macOS localization investigation — 2026-10-24

The package had `CFBundleDevelopmentRegion=en` and `CFBundleLocalizations=[en, ko]`, but it did not contain the matching `Contents/Resources/{en,ko}.lproj/InfoPlist.strings` resources. The expanded bundle contract test failed on the missing English file. The Rust bundle assembler now creates both localized files with the bundle display and bundle names.

The focused `macos_bundle` suite passed 3/3. An offline release build was assembled as a disposable `.app`; `plutil -lint` accepted `Info.plist` and both localized strings files. The corrected bundle was not opened, so LaunchServices runtime acceptance remains unverified.

An earlier UI attempt bound to the older app in `dist` and showed existing user profiles and a saved workspace. No controls were clicked and no settings, workspace, approval, or merge were changed. Automatic quota refresh during that window is unverified. Do not relaunch by the shared display name/bundle ID until the UI target can be isolated reliably.

The UTM Windows guest is currently unbootable with a blank display after reset. Leave it untouched; Windows runtime/UI acceptance is blocked.

Evidence: [issue9-macos-localization-20261024.json](../../docs/evidence/issue9-macos-localization-20261024.json).
