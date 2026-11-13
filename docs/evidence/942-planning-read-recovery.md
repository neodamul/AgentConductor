# Read-only planning recovery regression

Issue #942, release blocker for 0.2.8. Trace: R-122 / BG-77 / SF-109 / UC-112 / FR-107 / QAS-118 / DD-131 / DD-33.

## Cause and correction

The #931 shared decoder treated all nonempty permission_denials as failed completion. This was correct for guarded worker completion but too broad for a read-only coordinator that recovered denied probes. In the reported log, original-directory Glob/Read were denied, then all required files were read successfully from the exported snapshot and StructuredOutput returned a successful plan. The application rejected that recovered proposal.

The coordinator has an explicit decoding context, separate from worker profiles. Bounded matching read call/results retain observed denial IDs and subsequent successful read kinds. Unresolved, late, write, unproven, malformed and incomplete-history denials remain failures. Accepted output still has to parse as the requested JSON; normal plan/role/scope/test-command/phase checks still run. Worker effective-mode and denial handling is unchanged. No extra directory permission is granted. The planning prompt directs reads to snapshot-relative paths.

## Local evidence

- Red: recovered coordinator fixture rejected a valid plan; planning error incorrectly recommended worker-only native_callback configuration.
- Green: 45 focused tests cover coordinator parsing, real planner subprocess/snapshot execution, unresolved denial, strict worker behavior, malformed/overflow history, existing planning, prompt order, source bounds and architecture trace.
- A temporary private-log replay of the actual user failure passed four phases and ten tasks through role application, Plan validation, vacuous-check rejection and phase-tree writing. It sent no generation request, wrote no user workspace files, and removed temporary replay source/output. Private paths/goal/file content are not committed.
- Additional protocol, native callback, denial and generated-manual regressions passed (36 tests). Clippy with warnings denied and formatting/source bounds passed.
- Existing worker fixtures continue to block actual permission failures and quota-noise confusion.

## Release containment

Release workflow 43394962612 was cancelled, but publication raced cancellation: v0.2.8 was published briefly after the initial not-found check. The release was then returned to draft, preserving its original tag and assets. A cancelled workflow is not proof that publication did not happen; always inspect the release resource too. No automatic republish or tag overwrite is authorized by this fix.

Physical Windows and authenticated CLI acceptance are distinct from fixture/private-log replay; record further acceptance separately.

Local application acceptance: the signed patched build was installed over the existing app after stopping the prior app. The same saved Workspace/Goal/roles were restored and planning was restarted through the UI. The real Claude planning request completed successfully and the app displayed four phases, the current phase task list and an enabled Approve and run button. No task execution approval was performed. The final CLI result reported is_error:false and an empty permission_denials array; explicit snapshot guidance avoided the original-path probes on this retry.
