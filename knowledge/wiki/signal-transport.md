# Signal transport (DD-100)

Directives are always persisted first in the Rust ledger (`cooperative::issue`). `cooperative::route::deliver` then follows the workspace preference in `signal-transport.json`: Local Rust never contacts agentmemory; agentmemory preferred publishes the directive envelope once through the existing 8 s memory client. Any failure records `effective: local` with a bounded cause; the recipient keeps polling the same directive ID. The route is recorded once per directive (`signal_route/<id>`), so retries never publish twice. A workspace lock allows one external operation at a time; a lock older than 30 s is treated as left by a crash.

Settings shows the preference, port and last route; the briefing shows each session's route. Remote recipients have no route, so local fallback never claims remote delivery.

Trace: R-92, DD-100, QAS-88. Evidence: tests/signal_transport.rs, tests/test_signal_settings_ui.py.
