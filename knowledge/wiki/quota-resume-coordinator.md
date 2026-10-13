# Quota resume coordinator — DD-136

Workflow action: act_mv6xyu2f_3d17a233ad64.

Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.

Coordinator directive: reload the durable run reservation before each action; never reuse an assignment from a previous timer. Wait up to five hours, resume only the same waiting task/agent at its recorded deadline, and use only the fallback in the reviewed plan when the horizon is exceeded. Honour Stop and exact run leases. Persist cancellation and completion evidence.

Plan → failing tests → implementation → verification → main/push → Claude signal.

Verification (2026-10-14, Windows): Rust39 focused cases and Python65 passed; fmt, all-targets Clippy and locked offline build passed. Red evidence included missing reset timestamp/API/UI helper and the same-account manual-resume state regression. Native fixtures executed due work without a UI, killed and restored the owned coordinator with the same deadline, and cancelled reservations on same-account reassignment and Stop. First restriction history persists across retry/metadata refresh. Stop regression allows a bounded Windows preparation interval independently of its agent timeout.
