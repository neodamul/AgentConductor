# Run finalization visibility

GitHub #50 adds an explicit Verify → Merge → Workspace path to the Agent Room. The workflow report can remain `approved` while the coordinator is still applying the candidate, so the UI must also read the existing job snapshot: `job.kind == "merge" && job.state == "running"` is live merge progress, not a finished or stalled run.

Keep latest-candidate review, human approval, and final apply as separate gates. During apply, show localized status and a busy primary action in the run banner; after completion, offer the existing return-to-workspace action. Respect reduced-motion preferences for the active-stage indicator. The main-branch behavior still needs equivalent native Rust UI coverage before #50 can close.

Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166.


2026-10-20 current-head verification: issue #9 branch `842744d` passed the two native finalization regressions and the 300-line/source-test separation guard locally. The existing host needs a temporary Z3 5.1 soname lookup shim for LLVM. This is headless component evidence only; packaged visual review remains open, and the default JavaScript UI remains on `main` via PR #51.
