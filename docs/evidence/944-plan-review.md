# #944 — Plan review duplicate Phase summary

## Plan → task

The ready proposal rendered inside a second Phase details box, preceded by a generated task-ID brief and an editing instruction. The runtime regression failed with one duplicate Phase box, one brief and the instruction present.

Render the reviewing task nodes directly. Reuse the Plan eyebrow for Phase position/name and retain the existing task-position line. Keep historical Phase navigation and meaningful descriptions; remove task-ID-only briefs before rendering, including the left outline. No extra request, thread, dependency or polling loop.

## Verify

- Local Phase runtime regression: light/1280 and dark/840 passed. Includes no duplicate box/instruction/brief, history selection/expansion, retained input on return, English/Korean heading and visible meaningful history.
- Goal lifecycle runtime: 26 checks in each light/dark × 840/1280 combination passed (104 checks).
- Design parity: Plan eyebrow/title and phased single-list state passed; dark/840 eyebrow also passed. Full assembled UI screenshot inspected; it is simulator evidence, not a packaged app screenshot.
- Rust source/layout/architecture, i18n/phased-planning and manual tests passed (24 distinct cases). Formatting and diff checks passed.

Windows packaged-app execution and release acceptance are not claimed by this local UI fix. The existing installed application was not restarted or replaced.
