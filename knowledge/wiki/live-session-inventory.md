# Live session inventory

#955: Startup polling stops in Room, while agent cards are retained. Feed the shared session observer from both snapshot paths. Reuse the report loaded for Workspace run_state to derive a narrow revision; Room already has task fields. A session may start after status becomes running, so include started/finished timestamps.

Unchanged snapshots cause no inventory scan. Coalesce changes during a request into one follow-up and discard superseded responses. No new timer is needed.
