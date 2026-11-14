# Slack setup guide (#980)

## Plan

Existing R-43 / DD-79 notification settings gain discoverable setup guidance. Business goal: let an operator configure outbound Slack notices without product-source knowledge. Feature: a collapsed, localized guide beside the Slack webhook setting. Use case: open Messenger settings, expand the guide, create a Slack app with only incoming-webhook permission, authorize a channel, paste the secret locally, test, then enable and save notifications. QAS: at 840px in both themes and languages, the expanded guide remains readable without horizontal overflow and generates no network request. Design: static translated disclosure, hidden for company webhooks, existing masked input and save/test paths unchanged. Official setup address is selectable text so the embedded application does not navigate away.

## Privacy

Only generic instructions and a public Slack setup address are included. No operator account, channel, project, webhook, token or personal configuration is used as fixture data. Credentials remain machine-local and are excluded from release assets.

## Workflow

- [x] Plan: scope, use case, QAS and design recorded above.
- [x] Task: regression failed because the guide did not exist (Red).
- [x] Verify: localized disclosure, channel switching, no horizontal overflow at 840px in both themes/languages, hand cursor, no save/send side effects, and 24 existing settings checks. Localization runtime checks and all eight message bindings also passed. A simulated dark/Korean screenshot was inspected; it contains no operator settings. Exact local credential-value scanning found no matches in tracked or new files.
- [ ] Completed: reviewed change and merged linked PR.
