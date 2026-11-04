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
