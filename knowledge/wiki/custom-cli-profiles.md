# Declarative custom CLI event and quota behavior

GitHub #12 found that custom profiles already accept arbitrary executables, arguments,
environment and parser selection. The remaining provider-specific branches were the
custom JSONL event vocabulary and quota eligibility/query path. Built-in Claude, Codex
and OpenCode adapters keep their existing behavior.

## Profile contract

### Models, execution arguments, and results

`catalog` is the authoritative model list. Keep an operator-managed catalog by setting
its origin to `manual`, or configure `model_discovery` with direct argv, a bounded
timeout, and JSON Pointers for the returned model array and each model ID. The system
does not infer a model endpoint or slash command from an OpenAI-compatible interface.

`execution_options` declares generic argument groups. Each group has a unique name,
argument templates, allowed values, an optional default, and optional per-model values.
Templates substitute `{value}`, `{model}`, `{session_id}`, and `{task_file}`. A task
override takes precedence over the profile default; an unset optional group contributes
no argv. Model values and resume IDs come from coordinator-owned task/session state.
Task overrides persist in the plan and are checked again against a replacement agent.

`result_pointer` extracts the selected JSON value from a successful result event.
`error_condition` maps a matching JSON Pointer/value pair to an error event before result
extraction. An error event is preserved even if it has no result field. When extraction is
unset, legacy custom `payload` wrappers continue to work. These fields are editable in the
Rust profile editor and survive workspace JSON serialization.

A custom profile may map raw JSONL `type` names to the standard event kinds consumed by
the coordinator. `session_id_pointer` optionally selects a session ID with JSON Pointer;
without it, the decoder uses the legacy `session_id` field. Existing profiles therefore
keep their old contract. Invalid event names, pointers, or provider use are rejected at
plan validation.

An optional `quota_command` enables custom account refresh. Its `args` run directly with
the profile executable, without a shell. `{workspace}` and `{agent_id}` are expanded.
The timeout defaults to 30 seconds and is bounded to 1–300 seconds. Process stdout and
stderr are retained in the quota process log directory. A timeout, failed exit, malformed
JSON, oversized output or invalid value is surfaced as a failed observation.

The command must print one JSON object that matches the normalized quota `Snapshot`
schema: `source`, positive `observed_at`, `windows`, `budgets`, with optional `plan_type`
and `restriction`. `output_pointer` can select that object from a wrapper. Window and
budget counts, string sizes, percentages and timestamps are bounded before display.
Provider-specific quota formats are not inferred: the CLI or a local wrapper normalizes
them. No parser DSL, HTTP transport, or custom inline editor is introduced. Operators
manage these settings with the existing workspace JSON export/import flow.

Example profile fragment:

```json
{
  "id": "aipro",
  "parser": "custom",
  "executable": "aipro",
  "args": ["--jsonl"],
  "event_types": {"working": "progress", "finished": "result"},
  "session_id_pointer": "/meta/session",
  "quota_command": {
    "args": ["quota", "--json", "--workspace", "{workspace}"],
    "timeout_seconds": 30,
    "output_pointer": "/account"
  }
}
```

## Verification and limits

Rust fixtures cover configured event/session mapping, legacy profile defaults, coordinator
result parsing, quota pointer parsing, invalid percentages, direct executable invocation,
retained logs, scheduler eligibility and malformed settings. The custom quota executable
fixture uses `/bin/echo` on Unix. Rust's quota-due snapshot and `quota-refresh` command
support configured custom profiles. The current production WebView still gates quota
controls to Claude and Codex; exposing custom quota controls requires the Rust UI migration
tracked by R-151. No JavaScript or Python source/test changes are included. Full Rust
verification and cross-platform process behavior remain subject to local and platform CI
before merge. No claim is made about an untested private quota CLI.

Design trace: R-154 → BG-109 → SF-141 → UC-144 → FR-139 → QAS-150 → DDR-17 → DD-164.
Implementation checklist and evidence: [docs/07](../../docs/07-implementation-plan.md).
