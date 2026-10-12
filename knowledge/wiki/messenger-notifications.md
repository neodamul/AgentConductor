# Messenger notifications (phase 1)

Trace: R-43 → BG-18 → SF-37 → UC-34/UC-61 → FR-28/NFR-17 → QAS-35/QAS-66 → DD-79 (phase 2 stays DD-51).

## How it works
- Settings: `Config.messenger` {enabled, kind slack|webhook, events, template, include_goal}; exported with the workspace. URL: `~/.agentconductor/messenger-secrets.json` (0600) keyed by the workspace folder digest — never exported, never returned to the page.
- Sources: `workflow::save` → `messenger::notify_run(dir, state, done, total)` for every report save; the desktop writes `<run>.messenger.json` before `startup-run`; `<run>.messenger-sent` suppresses repeats. `Startup::settle` → `notify_plan` when planning completes.
- Delivery: system `curl --silent --show-error --fail --max-time 10 --retry 2 --config -`; URL/header/body only on stdin. One delivery at a time; `messenger::flush(35 s)` at CLI exit so the final state is not lost. Failures go to `<run>.messenger.log` with the URL masked.
- Text follows the interface language (`ui::language`).

## Lessons
- Secrets on argv are visible in process lists; curl's `--config -` keeps them on stdin.
- A detached delivery thread dies with a short-lived CLI process — flush before exit.
- Local live check: self-signed cert + `CURL_CA_BUNDLE` + a Python HTTPS receiver (see the ignored `live_delivery_through_the_system_curl` test).

## Phase 2 — decisions from the messenger (DD-81)
- `src/messenger/inbound.rs`: started by `startup-run` for a linked run with `decisions`; wakes every 15 s, touches the network only while the run's tool ledger has a pending, unexpired request.
- Code = first 8 hex chars of the request SHA-256 (upper case). Replies `allow|approve|deny|reject <code>`; Slack bot/edited messages ignored; author must be in `approvers`.
- Slack: `conversations.history` with `Authorization: Bearer <bot token>` (`AGENTCONDUCTOR_SLACK_API` overrides the base for local tests). Company: relay `?since=<ts>` with HMAC-SHA256 signatures (RFC 2104 on `sha2`, constant-time compare).
- Apply = `current_exe tool-approve|tool-reject <run> <id>` (unchanged DD-32 binding, expiry, single use); audit `messenger_decision/<message>` in the run store; confirmation posted. State in `<run>.messenger-inbound.json` (announced IDs, cursor, last 500 handled messages).
- Limitation: stop-and-resume requests (run process already exited) are answered in the desktop.

