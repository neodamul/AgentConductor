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

## Live Slack delivery

At 13:49 KST on 2026-11-14, the production Rust `messenger::test` implementation from the 0.2.10 release build successfully sent a notice through a real Slack Incoming Webhook. Slack displayed the notice as the installed notifications application in the operator-approved channel. This was an application Webhook post, not a browser-authenticated user message.

A disposable Rust verifier linked the existing release library, saved the Webhook using `messenger::save_secret`, then called `messenger::test` without passing a URL, proving the saved local credential was used. The verifier exited zero; receipt was separately observed in the Slack channel. The existing 0.2.9 desktop window was busy, so its settings button was not used for this live check.

The credential is stored outside the repository in the machine-local messenger secret store with Unix permissions `0600`. An exact-value check found no Webhook in Git-tracked files or the local macOS release archive. The archive also contains no messenger secret-store file. No credential or personal workspace content is included in this evidence. Temporary credential and verifier files were removed after verification.

## Limits

Live outbound delivery is verified on macOS. Real inbound bot replies, service rate limits, Windows live delivery, and the desktop settings-button interaction were not exercised against Slack; deterministic authorization, transport, settings and UI regressions remain the evidence for those paths. Notification event preferences were not changed and no operator Goal was started.
