# Run-scoped Room tabs

Issue: #960.

A recreated Room initializes JavaScript state but reuses existing DOM, so setting activeTab alone does not reset visible tab selection. Call the existing changeTab on its first accepted snapshot and whenever run_dir changes; preserve the selected tab for unchanged run identity. Clear loaded review and reject evidence responses captured for another run. The stopped-Room guard handles responses from an obsolete Room closure.

Verification uses actual assembled Room DOM for same-run polling, a changed run, a recreated Room and late evidence. No agent CLI or user run is executed.

Background coordinator evidence refresh passes activate=false to loadReview; only an explicit review action selects that tab.
