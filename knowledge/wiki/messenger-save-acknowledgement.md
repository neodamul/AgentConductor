# Notification settings require a save acknowledgement

Source: [fingerprinted investigation](../raw/messenger-save-881.json), GitHub #881.

The previous dialog saved from its close event. That made local persistence failure look like a successful dismissal, with the old checkbox value restored on reopening. Saving while another action held the mutation guard could also be silently skipped.

Handle confirmation before closing. Keep the form and its draft on failure or contention, show an inline error, and close only after acknowledgement. Block duplicate submission and cancellation while pending. Do not implicitly enable notifications or weaken sanitized imports.

The delayed/failing/busy-save browser regression covers both themes. Rust persistence tests reopen the existing workspace through a replacement executable path and retain opt-in; ordinary saves and sanitized imports remain covered. An updater-specific reset has not been reproduced. No external messages were sent.
