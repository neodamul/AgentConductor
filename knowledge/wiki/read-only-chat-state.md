# Read-only chat and mutable state

#961: A chat question is not consent to mutate a reviewed plan. Rejecting it against a digest of mutable job/phase/chat-result state blocks normal questions. Startup polling pauses in Room; chatSync refreshes it only after chat becomes busy, so it cannot fix an immediately rejected first send. Bind the read-only worker to current context instead, preserving coordinator/profile validation, input bounds, one-chat concurrency and mutation revision guards.

Local evidence was inspected read-only without publishing goals, chats, paths or secrets. Retained SQLite audit does not contain historical Saved snapshots; local diagnostics do not retain the reported failure. Therefore the exact drifting field in the original report cannot be proven. The regression deterministically changes chat_result and Goal context and verifies the latest-context answer.
