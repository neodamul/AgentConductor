# Messenger conversation threat-model source

- Retrieved: 2026-11-05 09:41 UTC
- Repository revision: `7ee7826e47e91298d1c64c855d597af329fc01ca` (issue branch; current main ancestor integrated)
- Primary source: https://github.com/neodamul/AgentConductor/issues/519
- Related interface change: https://github.com/neodamul/AgentConductor/pull/520, head `24e70fef056401e7789aeb487912198720783ee4`
- Existing decisions: `docs/06-design-decisions.md` DD-32, DD-51, DD-79 and DD-81

The issue asks for a threat model before messenger-based coordinator conversation. Messenger text is untrusted; the review must cover prompt-injection assumptions, allowed actions (approve, deny, resume, stop, answer a question), forbidden settings/destructive operations, sender authorization, routing, ignoring own messages, and import/export identity redirection. The interface PR is in progress; its `conversation` capability is not evidence that phase 3 behavior is implemented.

No credentials, company messages, or private workspace data were collected for this observation.
