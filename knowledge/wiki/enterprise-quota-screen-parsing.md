# Claude Enterprise quota screen parsing — GitHub #295 / issue #9

## Context

The latest `main` fix for #295 showed an Enterprise allocation using grouped amounts such as `$1,351.23 / $5,000.00`. A terminal may encode the spaces between words as CSI cursor-forward sequences instead of literal whitespace. The Rust port previously rejected the grouped allocation, stripped cursor movements without restoring spaces, and discarded parser errors while waiting until the generic `/usage` timeout.

## Decision and behavior

- Accept only valid comma-separated thousands groups and normalize them before the existing monetary parser.
- Preserve CSI cursor-forward as bounded spaces (at most 200 per sequence) before parsing the screen.
- Reject malformed grouping and preserve the most recent parse error in the bounded timeout diagnostic, in both platform collectors.
- Keep the existing CLI login and no-generation command. The bounded wait is 180 seconds for managed environments that exceed the prior 60-second limit; it stays within the 300-second ceiling for configured custom quota commands. Keep native inline error display. Do not infer account usage or contact a provider from tests.

This refines R-202 → BG-157 → SF-190 → UC-193 → FR-194 → QAS-200 → DD-214. The Rust branch already owns that requirement trace; no new requirement ID or stacked PR is needed.

## Slow managed CLI startup — GitHub #315

Some company-managed hosts exceed the prior 60-second wait before Claude finishes startup and `/usage`. Keep the same shared deadline for both macOS and Windows collectors, but raise it to 180 seconds. This remains within the 300-second cap already allowed for custom quota commands. Parsed quota still returns early, and timeout messages retain the existing stage and duration.

TDD changed the production contract test to require 180 seconds first; it failed with the existing 60-second value. After changing the shared constant, `claude_wait` passes 3/3, `claude_prompt` 4/4, and `source_layout` 2/2. Warning-denied all-target Clippy, rustfmt, diff hygiene, and the 300-line source/test guard pass locally. The Windows MSVC target is not installed here, so no Windows build claim is made.

## Verification

TDD first failed on `$1,351.23 / $5,000.00` and on a readable invalid screen whose parse reason disappeared into the generic timeout. The parser, macOS collector, Windows collector, and deterministic CLI fixture now preserve those distinctions.

Focused macOS verification passes 41 quota/parser/wait tests (1 ignored) and 6 native quota error/profile/room tests. The full local suite passes 1,240 tests across 379 suites (16 ignored), including source layout and the 300-line guard. `cargo fmt --all -- --check` and warning-denied all-target Clippy pass.

The local Windows cross-check could not start because this machine has no `x86_64-pc-windows-msvc` Rust standard library installed; `rustup` is also unavailable. Windows-only collector compilation and exact packaged Windows behavior remain unverified and must not be reported as passing.

The full suite exposed a test-only `PATH` race: `startup_chat`'s real Git fixture could overlap another test's slow fake Git executable. Both tests now share the same environment mutex, and the suite passes with the normal parallel runner. This prevents an unrelated scheduling failure from obscuring the quota regression result.
