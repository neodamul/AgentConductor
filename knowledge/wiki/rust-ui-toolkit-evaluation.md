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

No GUI dependency has been added to the application. An isolated macOS eframe 0.37.0 probe
builds offline with an 8.3 MiB executable and 291 entries in its normal dependency tree. A
Rust AccessKit-tree test caught and then verified the accessible name for the goal input.
This is only a small prototype: sandbox policy denied process inspection, so its idle RSS and
CPU were not measured; CUA could not attach to the ad-hoc executable, so visual layout and
Korean IME were not checked; Windows and the exact transitive license closure remain
unverified. Keep the historical WebKit baseline in `resource-budget.md` as the only measured
runtime evidence. If any adoption gate fails, revisit the toolkit before migrating production
screens.

Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160 → DD-161.
