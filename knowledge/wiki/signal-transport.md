# Signal transport (DD-100)

Directives are always persisted first in the Rust ledger (`cooperative::issue`). `cooperative::route::deliver` then follows the workspace preference in `signal-transport.json`: Local Rust never contacts agentmemory; agentmemory preferred publishes the directive envelope once through the existing 8 s memory client. Any failure records `effective: local` with a bounded cause; the recipient keeps polling the same directive ID. The route is recorded once per directive (`signal_route/<id>`), so retries never publish twice. A kernel-owned workspace lease serializes the receipt check, external dispatch and final route record; it releases on process exit without age-based lock stealing. A durable attempt is recorded before network I/O. After a crash with an uncertain receipt, recovery retains the same local directive and does not republish it. A memory receipt means stored transport evidence, not recipient acceptance.

Settings shows the preference, port and last route; the briefing shows each session's route. Remote recipients have no route, so local fallback never claims remote delivery.

Trace: R-92, DD-100, QAS-88. Evidence: tests/signal_transport.rs, tests/test_signal_settings_ui.py.

Review correction: the original post-v0.1.10 adapter recorded only the final receipt. A Red regression reproduced a second external publish after a simulated restart. Durable pre-dispatch claims and kernel leases address that failure; tests also exercise a competing owner and run on both native platforms. v0.1.10 predates this adapter; further delivery remains separately tracked.

[Review source fingerprints](../raw/2026-10-12-signal-transport-review.json) preserve the tested code and correction.
