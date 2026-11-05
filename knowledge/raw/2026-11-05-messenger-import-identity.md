# Messenger identity must remain local

Issue #523 identified that a shared workspace file could import messenger activation, decision mode, approver identities and a Slack channel into an already configured workspace. Imported JSON is untrusted and must not establish who can approve a pending tool request or where the reply poll reads.

The exporter and importer force messenger notifications and decisions off, clear approvers and remove the Slack channel. Local secrets remain machine-local and are not imported. Regression fixtures set all four hostile fields and verify both the shared file and saved workspace contain none of that authority.

Verification belongs to the issue-linked implementation PR; this note records the security invariant and its evidence.
