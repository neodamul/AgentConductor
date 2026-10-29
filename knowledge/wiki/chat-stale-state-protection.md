# Chat requests and stale workspace state

Coordinator chat is read-only with respect to repository files, but it still depends on the saved workspace context: the selected coordinator, CLI profile, repository, goal, and reviewed proposal. A delayed UI request must not silently launch against settings different from the snapshot the user saw.

The general startup revision includes the current repository `HEAD`. Recomputing that revision during chat dispatch can invoke Git and delay the chat request. The chat path therefore uses a separate `chat_revision` derived from normalized saved state and proposal review evidence, without invoking Git for `HEAD`. Chat dispatch checks this token before checking existing jobs or launching a child process. A changed saved context is rejected with the same refresh-before-acting error. Mutating operations continue to use the full repository-bound revision.

Regression coverage in `tests/startup_chat.rs` checks that a stale saved goal is rejected and that chat dispatch does not wait for a slow `rev-parse HEAD`. `tests/chat_runtime.cjs` confirms the page sends the chat-specific token.

This implementation does not prove that an external coordinator answered promptly. The user's `~/.agentconductor/request-diagnostics.log` still distinguishes native dispatch delay from child-process wait time.
