# Custom CLI JSONL event normalization

Custom agent profiles may map vendor-specific JSONL event names to the shared event kinds and
declare a JSON Pointer to their session identifier. The metadata is used only for profiles whose
provider is `custom`; built-in Claude, Codex, and OpenCode parsing remains unchanged.

`Decoder::for_profile` applies the mapping in live worker progress, task result parsing, interrupted
run recovery, and coordinator document parsing. The original JSON object remains attached to each
normalized event, preserving the evidence used by the existing review and audit paths. A missing
or non-string session value does not create a session identifier.

Plan validation limits mappings to 64 events, limits event names to 128 bytes, rejects control
characters, and limits the session pointer to 256 bytes. Empty pointers refer to the full JSON
value; non-empty pointers must start with `/` and use only valid `~0`/`~1` escapes. This slice does
not add the machine-local custom profile registry or quota command declarations; those remain in
implementation-plan Task 2.21.

Regression coverage includes custom event mapping, session extraction, coordinator terminal-event
recognition, plan validation, and a complete parallel mock workflow that reaches human review.
