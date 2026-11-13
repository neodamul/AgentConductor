# Alternate Rust UI retirement verification

Issue #940, milestone 0.2.9. Trace: R-151 / BG-106 / SF-138 / UC-141 / FR-136 / QAS-147 / DD-160 / DD-161.

## Removal

- Deleted 39 alternate-UI source/test files, the probe binary, optional Cargo feature and dedicated CI build/test/Clippy job. No replacement Rust UI-only test suite remains.
- The production macOS WKWebView and Windows WebView2 shell, shared backend, native permission callbacks and their verification remain supported.
- Removed eframe and 230 unreachable package-version identities from Cargo.lock: 508 -> 278. Retained dependency identities were unchanged; no dependency upgrades were performed.
- Actual offline Cargo resolution: macOS 60 package nodes, Windows 117. Neither contains eframe, egui, winit, glutin, clipboard-win or libloading.
- Removed two unused upstream license texts/records and prototype-only license tests. Retained WebView2 provenance and all generic license/SBOM/package checks remain. ISC/BSL prototype-only allowances were removed from the package review list.
- Updated active developer/CI commands and removed nonexistent test citations from architecture evidence. Archived migration rationale is explicitly retired.

## Local validation

A temporary removal-contract test was observed failing while the old feature existed; it was removed after Green verification because the user requested no alternate-UI-only tests. Final Cargo metadata/source/CI searches confirm absence of the retired targets and feature.

- cargo build --all-targets --locked --offline: passed.
- cargo clippy --all-targets --locked --offline -- -D warnings -D clippy::undocumented_unsafe_blocks: passed.
- 61 retained Rust tests: library, architecture, current native room/window, project licenses, package assembly, release workflow contracts, source layout and public docs passed.
- 47 Python validation tests passed.
- Changed-file formatting, 300-line limits and whitespace checks passed. Existing generic package fixtures moved unchanged to support; focused package/architecture checks passed again.

These are local macOS checks and both platform dependency/package fixtures. No physical Windows desktop run or release was performed. UI-only global Cargo caches and another agent's working tree were not removed.
