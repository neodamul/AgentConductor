# Slack integration verification (#972)

Date: 2026-11-14 (Asia/Seoul). Scope: existing R-43 / DD-79 / DD-81 behavior, before stable 0.2.10.

## Result

- 22 focused native messenger tests passed; covers settings surviving executable replacement/plain saves, workspace enablement, event selection, channel registry, JSON payloads, local secret isolation/export boundaries, burst folding, notification re-entry, authorized decisions/conversations, and stranger/bot/edit rejection.
- Messenger-save browser regression passed 12 checks in each theme (24 total).
- Both opt-in system-curl tests passed against a disposable localhost HTTPS receiver, with a generated trusted test certificate: outgoing Slack/company payloads, connection refusal, authenticated conversations.history, listed-user deny, and acknowledgment. Four JSON messages were received locally (two test notices, a decision request and its denial acknowledgment).
- No product-code change was needed for these verified paths. Manuals now distinguish workspace invitations from Incoming Webhook configuration.

## Reproducible checks

Focused targets: `tests/messenger.rs`, `tests/messenger_settings.rs`, `tests/messenger_channels.rs`, `tests/messenger_events.rs`, `tests/messenger_inbound.rs`, `tests/messenger_incoming.rs`, `tests/messenger_import_security.rs`, `tests/messenger_renotify.rs`, `tests/messenger_conversation_flow.rs`, `tests/messenger_decision_parse.rs`.

The ignored `live_delivery_through_the_system_curl` and `live_round_trip_through_the_system_curl` checks were explicitly enabled using `LIVE_MESSENGER_URL`, `LIVE_SLACK_DIR`, `AGENTCONDUCTOR_SLACK_API` and `CURL_CA_BUNDLE` targeting only the disposable loopback fixture. The receiver was stopped and its certificate/key/data were removed afterwards.

## Limits

The supplied workspace invitation grants no webhook/bot configuration. The user subsequently requested an actual Slack message. Opening the supplied invitation reached an email/identity-provider signup page, not an authenticated channel; login and target channel are pending user input. No real Slack account was joined and no real channel has yet received a message. Actual Slack installation, bot scopes/channel membership and service rate limits remain operator-specific checks. These localhost transport tests ran on macOS; Windows package/CI checks are tracked by the release workflow.
