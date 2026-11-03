# Release Screenshot Provenance Audit — 2026-11-02

## Observation

PR #336 updates `docs/assets/screenshots-current.txt` to claim the README images match v0.1.56 and
were recaptured on 2026-11-02. The PR does not change the referenced PNG files. Visual inspection
shows the app title bars in `ui-workspace-current.png`, `ui-agent-room-current.png`, and
`ui-agent-settings-current.png` as v0.1.54, v0.1.49, and v0.1.24 respectively.

## Action

Commented on PR #336 (comment 6575187579) requesting current captures with provenance or accurate
historical labels. Updated the #9 progress comment and the Rust migration PR description to reflect
the actual open-PR and CI state. GitHub refused a formal `REQUEST_CHANGES` review because the PR
belongs to the authenticated user; no review state was changed.

## Verification boundary

This is a visual file inspection, not a runtime capture of the v0.1.56 release binary. The mismatch
is sufficient to reject the current claim of version-matched screenshots, but it does not establish
which visual differences are present in the released application.
