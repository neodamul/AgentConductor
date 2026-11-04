# Milestone release verification

Trace: R-210/R-211/R-212/R-213 → UC-202/UC-203/UC-204/UC-205 → QAS-209/QAS-210/QAS-211/QAS-212 → DD-222/DD-223/DD-224/DD-225. GitHub #420.

A passing source-string assertion is not a UI integration flow. A fake bridge
proves UI requests and rendering; native controller tests prove filesystem and
approval effects. Pixel comparison proves the measured element, not the entire
workflow. Read test bodies before assigning evidence in docs/14. In particular,
Windows workspace saving does not prove Goal results or Explorer folder opening.

`tests/verification_evidence.rs` rejects nonexistent nested test references and
traversal. `tests/parity_matrix.rs` compiles the actual Rust parity driver and
captures subprocess requests: all four theme/width cells and separate artifact
paths must exist, and CI cannot silently skip a missing browser. Responsive
entries cannot exclude 840. Dedicated fullscreen/short-height/breakpoint cases
are additional fixed-purpose checks. Operator-owned names must use identical
fixture values on both sides; otherwise wrapping can create false differences.

`tests/goal_action_safety.rs` races real dispatch requests and tests aliases to the
project root. Windows adds a junction case; `tests/windows_goal_folder.rs` observes
only the uniquely owned Goal path in Explorer, closes that window and verifies
permanent deletion preserves project source. These platform-only tests need a
Windows run; macOS compiling zero applicable tests is not evidence of success.

Before publishing v0.2.0, run:

```sh
cargo test --locked --test release_readiness milestone_020_release_gate -- --ignored --nocapture
```

The check only reads repository files, GitHub milestone state and the existing
screenshot freshness verifier. It never tags or publishes. Normal offline tests
exercise missing/inconsistent versions, missing release notes, open issues and
stale captures; the real check is explicit because it needs GitHub access. The
v0.2.0 tag workflow invokes it before publication. Currently it correctly refuses
the unready candidate. #428 owns deferred screenshots/release preparation and
#472 tracks the unenforced 300-line rule; neither can be silently waived by a
coverage table.

## Joined UI boundary checks — #420

Run `node tests/goal_lifecycle_runtime.cjs` and
`node tests/goal_surfaces_runtime.cjs` for actual assembled-page sequencing
through a deterministic native-protocol bridge, both themes at 840/1280.
The former checks 15 lifecycle assertions per cell; the latter checks 18 cached
quota, bounds, focus and panel assertions. Fake replies cannot prove filesystem
delete, Explorer launch or provider execution; retain the Rust controller and
Windows-specific evidence alongside these UI checks. The lifecycle found #475
and now waits for the target Goal controls to settle before entering new text.

## Enumerated entries are not executed coverage

The inherited parity runner counted 14 behavior entries at 840 while skipping
their execution. Four successful subprocesses did not prove every entry ran.
`tests/parity_assertion_runtime.cjs` injects a forced-false assertion and requires
nonzero failure at both widths in both themes; removing the bypass makes it pass.
The real assertions must then pass, rather than being hidden again. The approved
9px narrow titlebar version is a selector-specific type-scale exception, tested
against rejection of 9px body text and 9px desktop version text.

## Stable evidence and navigation

Equal PNGs are insufficient if the target disappeared during every capture.
The runner now rejects exhaustion of its existing five attempts.
`tests/parity_capture_runtime.cjs` proves always-unstable captures fail, a stable
fifth attempt succeeds, an immediately stable target succeeds, and evaluation
waits for the new document. A white 2x2 PNG deliberately cannot hide instability.
Preset setup uses the existing awaited goto helper instead of an unchecked
reload plus a 1200ms sleep. Pixel and geometry tolerances are unchanged.

The Windows junction fixture must pass its static cmd command via raw_arg: cmd
uses different decoding from ordinary CRT arguments. Paths remain quoted
environment values, with delayed expansion disabled. This does not prove
Windows execution from macOS. See the [Rust CommandExt reference](https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html#tymethod.raw_arg).

## Compare the client viewport, not the preview canvas

At physical viewport840 the preview's outer 16px gutter and 1px window border
leave a client width806. The full app client is840. After a 230px rail and
18px main padding on each side, their tab rows are540 and574. Windows fonts
made only the preview shrink, producing false size differences.
`tests/parity_reference_runtime.cjs` reproduces the mismatch and requires equal
actual titlebar widths in both themes at840/1280. The harness resizes only the
reference frame width, retaining its border and height; source mock styles and
interior controls are unchanged.

Count specifications with PARITY_THEME set: the unfiltered170 definitions
include10 entries explicitly owned by the opposite theme. Each actual cell has
160 relevant entries, including14 default behavior assertions. Do not report
the unfiltered catalog as executed per-cell coverage.
