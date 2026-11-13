# Verifying Slack integration without posting externally

Issue: #972.

A workspace invitation is not application transport configuration. Outbound notices use an Incoming Webhook; inbound messages need the saved bot token, channel ID and configured identities. Verify per-workspace enablement and update persistence separately from secret presence; exported settings intentionally do not carry local secrets.

Use deterministic messenger tests first, then explicitly run the opt-in curl tests against a disposable local HTTPS receiver with its own trusted certificate. Verify both the decision notice and acknowledgment, so their expected extra message is not mistaken for a duplicate. Live Slack installation and membership are separate evidence.
