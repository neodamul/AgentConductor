# Native Rust UI toolkit evaluation

## Product constraint

AgentConductor exists to reduce the resource cost of running several AI desktop
applications. The distributed app must not retain authored JavaScript or Python runtime
behavior. The current Wry/Tao window embeds the operating system WebView, so migrating
business logic alone cannot satisfy the Rust-only UI requirement; the WebView shell itself
must eventually be replaced.

## Candidate: eframe/egui

The first prototype candidate is eframe/egui because the project is licensed MIT OR
Apache-2.0 and documents native AccessKit roles, accessible names, and widget-tree tests.
Its feature manifest permits disabling defaults and selecting a renderer. The prototype
should evaluate only the features needed for native OpenGL rendering, AccessKit, and
English/Korean fonts; verify the exact locked dependency/license inventory before any
production adoption.

Official references:

- https://github.com/emilk/egui#readme
- https://github.com/emilk/egui/blob/main/crates/eframe/Cargo.toml
- https://github.com/emilk/egui/blob/main/docs/accessibility.md

Slint is not the first candidate. Its framework is offered under GPLv3, a royalty-free
license that requires disclosure in proprietary desktop distributions, or a commercial
license. Adding that condition is unnecessary while a permissive candidate is available.
Reference: https://github.com/slint-ui/slint/blob/master/LICENSE.md.

## Adoption gates

DD-161 requires a one-window prototype and evidence before replacing production UI:

- Idle CPU remains below 1%; idle app RSS stays below the existing 100 MiB budget. Measure
  both the application and helper-process tree after 60 seconds with no work.
- Keyboard navigation, focus visibility, screen-reader roles/names, and Korean IME input
  work on macOS and Windows.
- English and Korean layouts preserve the selected AgentConductor design and fit without
  clipping at supported window sizes and display scaling.
- The app uses one native process/window, opens no local web server, and starts no provider
  CLI or refresh loop while idle.
- Both release targets build; the exact dependency closure, notices, archive size, and
  generated assets pass the existing license/package contracts.

The repository now has an optional `native-ui-probe` Cargo feature, a required-feature probe
binary, and integration tests in `tests/native_ui_probe.rs`. The default application and
release builds do not enable or link eframe. The probe calls the production `Room::dispatch`
through a bounded Rust worker and tests the AccessKit name for the goal input. The native
probe compiles with eframe 0.37.0 using system fonts, no bundled default fonts, and no wgpu
feature; this avoids the OFL-1.1/Ubuntu-font-1.0 package expression. GitHub Platform
Validation now builds and tests this opt-in target on macOS and Windows.

The exact eframe target dependency closure is permissive but includes ISC on macOS, plus
ISC and BSL-1.0 on Windows. Their exact upstream texts are pinned by revision and SHA-256;
Rust package tests verify these notices are included, including the BSL text omitted from
the `clipboard-win` crate archive. This licenses the candidate only; release-target CI must
still verify the exact native UI build and package before production adoption.

After 65 seconds idle, macOS `footprint` measured 47 MB physical footprint and `ps` sampled
0.0% CPU. The launch peak was 156 MB. Use physical footprint rather than RSS for comparison:
the existing WebKit baseline documents RSS inflation from shared framework pages. CUA could
inspect the packaged native window and verify the named goal input/checkbox, Tab focus, Space
activation, and Korean text rendering. Physical Korean IME composition remains untested, and
the skeletal sample still needs the product's visual/responsive layout. Exact-source run
38777444017 for commit `64446d4` passed all macOS and Windows platform validation, including
the optional probe tests/build/Clippy and package contracts. Its exact default-app package
jobs are running, but those packages do not contain eframe. Candidate-package license/resource
comparison and native IME/layout acceptance remain open. Treat the current evidence as partial
only. If any adoption gate fails, revisit the toolkit before migrating production screens.

Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160 → DD-161.
