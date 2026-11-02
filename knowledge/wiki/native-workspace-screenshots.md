# Current Native Workspace screenshots

## v0.1.54 capture — 2026-11-02

The current screenshot manifest contains three captures from the Rust native Workspace UI at source commit `880cd78d05e89c81f5150d479b32f69f91b24027`, application version v0.1.54. The English, Korean, and Korean dark captures use the existing README/manual asset paths and measure 1140×740 pixels. They were captured with the native eframe framebuffer callback after the initial Workspace snapshot, each under a separate temporary home directory. No user workspace, registered agent profile, provider CLI, or external service was used.

Capture hashes and the exact method are recorded in `docs/evidence/issue9-workspace-screenshots-v0.1.54-20261102.json`.

## Local macOS package

The v0.1.54 arm64 archive built from `2fa033bcd257cd0f3b718b6a8d49d84e039e48d7` passed `validate-package`, ZIP integrity, and packaged CLI help. SHA-256 checks of the three images extracted from the archive match the capture evidence. This validates package contents only; it does not establish interactive package UI acceptance or Windows acceptance.

## Scope

These images establish that the current Rust Workspace screen was captured in each shipped language/theme variant. They do not verify the packaged executable, Agent Room, agent settings, all JavaScript UI parity requirements, or issue #9 package acceptance.

## Freshness-check path handling

Manifest entries are basenames relative to `docs/assets/`. The Rust release checker must prefix that directory when asking Git for the newest screenshot commit. Otherwise, Git ignores the image paths and the checker falls back to the older manifest commit, incorrectly rejecting a valid recapture. `tests/screenshot_freshness.rs` covers image-only recapture after a later UI commit.
