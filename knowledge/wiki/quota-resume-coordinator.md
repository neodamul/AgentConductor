# Quota resume coordinator — DD-136

Workflow action: act_mv6xyu2f_3d17a233ad64.

Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.

Coordinator directive: reload the durable run reservation before each action; never reuse an assignment from a previous timer. Wait up to five hours, resume only the same waiting task/agent at its recorded deadline, and use only the fallback in the reviewed plan when the horizon is exceeded. Honour Stop and exact run leases. Persist cancellation and completion evidence.

Plan → failing tests → implementation → verification → main/push → Claude signal.

Verification (2026-10-14, Windows): Rust39 focused cases and Python65 passed; fmt, all-targets Clippy and locked offline build passed. Red evidence included missing reset timestamp/API/UI helper and the same-account manual-resume state regression. Native fixtures executed due work without a UI, killed and restored the owned coordinator with the same deadline, and cancelled reservations on same-account reassignment and Stop. First restriction history persists across retry/metadata refresh. Stop regression allows a bounded Windows preparation interval independently of its agent timeout.

Release follow-up v0.1.17: macOS CI exposed an idle watcher/foreground CLI lease race. Idle polling reads durable state without claiming the workflow lease; due continuation retains exclusive ownership and current assignment checks. Portable CLI schedule/reassign/model regression now runs on Windows too.

Windows portable CLI regression also exposed inherited capture pipes: standard process spawning inherits unrelated inheritable handles even with explicit child logs. Watcher spawn now serializes a scoped guard that clears inheritance only on caller standard pipe handles, restores their original flags on every exit, and preserves explicit child Null/log streams. The detached watcher skips console attachment. Native CLI schedule/reassign/model/due continuation passed after the previously reproducible captured-EOF timeout.
Primary context: https://github.com/rust-lang/rust/issues/73281 and https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html.
