# Messenger conversation threat boundary

Status: draft for GitHub #519. This design does not authorize phase 3 implementation.

## Trust boundary

Messenger payloads are untrusted, including text from an authenticated sender, quoted text, attachments, thread metadata, provider display names, and relay response fields. Authentication establishes sender identity only; it does not make message contents safe. The coordinator may receive a reply only when it is bound to one active question and a locally configured workspace/run/thread. Keep that reply as ordinary user input, never as a system instruction. Any tool action remains subject to the existing approval path.

## Authority and routing

- Use stable provider sender IDs and local approver mappings; for a company relay, require a valid signature, freshness bound and unique message ID.
- Resolve events by local channel-to-workspace binding and an explicit run/question thread. Reject missing, stale or ambiguous routing instead of guessing from prose or the latest run.
- Allow/deny only the exact pending, unexpired tool request through DD-32's request binding and one-time consumption. Plan consent and merge remain desktop-only.
- Resume/stop must name one run in the authorized workspace and pass core state checks; they cannot target arbitrary processes. Repeated commands must be idempotent or refused.
- A free-form answer is valid only for its active coordinator question. It cannot implicitly change settings, authorize tools, delete data, deploy, expose secrets, or merge.
- Ignore the application's own messages. Persist cursors and handled IDs so provider retries or restarts cannot replay an action.

## Configuration and evidence

Exports must omit credentials, provider identity, sender authorization and active routing. Imported settings are inert until locally reviewed and rebound; an imported file cannot add an approver or redirect notices to a file-selected identity. Send only opted-in, redacted notice fields. Record minimal actor ID, action, target, provider message ID, time and outcome; do not log credentials. Bad signatures, stale events, parsing failures, ambiguous routes and unavailable state fail closed.

The existing DD-32/81 approval path and local architecture trace support these boundaries. `Channel::Capabilities::conversation` in the in-progress #520 branch is a declaration only; it does not prove conversation behavior. The security model needs coordinator review before implementation.

## Open decisions

The provider's conversation API, exact message grammar, sender-to-operator enrollment, run/thread lifecycle, confirmation for resume/stop, retention/redaction of free-form answers, and required audit fields remain unspecified. Do not infer these from the channel interface.

## Sources and revalidation

- [Issue #519](https://github.com/neodamul/AgentConductor/issues/519)
- [DD-32, DD-51, DD-79 and DD-81](../../docs/06-design-decisions.md)
- [Source observation](../raw/2026-11-05-messenger-threat-model.md)

Revalidate when the phase 3 issue contract, provider API, channel interface or DD-32 authority changes. Confidence is high for the listed threat boundaries and provisional for any unimplemented interaction detail.
