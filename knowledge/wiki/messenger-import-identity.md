# Messenger identity-safe workspace import

Shared workspace files are untrusted configuration. They must not authorize a sender or redirect inbound decision polling. Export and import force messenger `enabled` and `decisions` to false, clear `approvers`, and remove `slack_channel`. Machine-local secrets are never shared.

The end-to-end regressions set hostile values before export and then modify the file before import; both the shared JSON and the persisted snapshot must have all four fields disabled or empty. This protects existing installations where local messenger secrets are already stored. See GitHub issue [#523](https://github.com/neodamul/AgentConductor/issues/523) and DD-59.
