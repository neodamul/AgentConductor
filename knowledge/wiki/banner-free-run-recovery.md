# Banner-free run recovery

Trace: R-210 / UC-202 / QAS-209 / DD-222. Issue: #927 (0.2.8).

The approved run layout has no run-status box above the card. Deleting the box must not delete its responsibilities: title pill for run state, outline for counts/phase, card diagnostics for errors and attention, card footer for feedback repair/scope review, composer for Resume and existing review/result footer for the next action. A failed integration follows the verification row; repair opens the exact failed task and drafts its output as plain text. Scope changes still need the exact reviewed digest.

Keep diagnostics in a stable card region outside replaceChildren-managed transcripts and summary fragments, so tab/review changes cannot detach them. Preserve busy guards, warning history, live runner/stale-state refresh and owned-file warnings. Full output is scrollable, never HTML.

The renderer is a fragment of the same startRoom closure (room_render.js), not a new service or event loop; real embedding and the simulator load identical fragments. Source-contract tests inspect both fragments. All changed code files remain at most300lines.

Red: the actual failed-state runtime still found run-banner. Verification covers light/dark at840/1280, failed-output repair, scope review, attention/safety warnings, approval/merge/result continuity, and reference parity. Local browser simulation is not physical Windows acceptance.
