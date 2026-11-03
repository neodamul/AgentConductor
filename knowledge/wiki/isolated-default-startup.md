# Isolated default startup validation

The Rust macOS executable's no-argument path calls the default Workspace startup. Validate it with a temporary bundle ID, disposable `HOME`, and empty workspace so global profiles, preferences, and workspace history are not read. The startup window should expose the three planning steps and registration prompt with no CLI agents configured.

This startup check is narrower than an end-to-end UI workflow. It does not count toward packaged workflow acceptance and does not prove the production bundle ID resolves to the intended app when duplicate app entries exist. Do not bind by the shared display name alone.

The UTM Windows guest is currently unbootable with a blank display. Leave it untouched; Windows runtime/UI validation remains blocked.

Evidence: [issue9-default-ui-isolated-startup-20261024.json](../../docs/evidence/issue9-default-ui-isolated-startup-20261024.json).
