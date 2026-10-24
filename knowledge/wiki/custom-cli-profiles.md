# Declarative custom CLI event and quota behavior

GitHub #12 found that custom profiles already accept arbitrary executables, arguments,
environment and parser selection. The remaining provider-specific branches were the
custom JSONL event vocabulary and quota eligibility/query path. Built-in Claude, Codex
and OpenCode adapters keep their existing behavior.

## Profile contract

### Models and execution options

`catalog` remains the authoritative model list. Operator-entered catalogs use
`"origin": "manual"`; a CLI discovery command runs only when `model_discovery` is
explicitly configured with direct argv, a timeout, and JSON Pointers selecting the
model list and each model ID. An OpenAI-compatible CLI is not assumed to have a discovery
endpoint or slash command.

Custom profiles may declare `execution_options`. Each group has a unique name, argument
templates, allowed values, an optional default, and optional model-specific allowed values.
Templates substitute `{value}`, `{model}`, `{session_id}`, and `{task_file}`. The engine
uses a task override before the profile default; it omits the entire group when no value
is available. The model group resolves from the selected/catalog model, and resume/session
groups use the session ID held by the coordinator. Task options are part of the saved plan,
survive workspace JSON export/import and resume, and are checked again against a new
profile/model before reassignment.

`result_pointer` selects the payload from JSONL result events. `error_condition` contains a
JSON Pointer and a JSON value; a matching result is normalized as an error even when its
event type is `result`. Omitted fields retain legacy custom-profile decoding. These
declarations are editable in the Rust profile dialog; built-in adapter contracts are
unchanged.

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
tracked by R-151. This PR adds no JavaScript or Python source/test changes. Built-in provider
regressions and cross-platform process behavior remain subject to regular platform CI
before merge. No claim is made about an untested private quota CLI.

Design trace: R-160 → BG-115 → SF-147 → UC-150 → FR-145/NFR-55 → QAS-156 → DDR-20 → DD-170.

The model/argument/result contract extends the profile fields under GitHub #62. Its Rust
fixtures cover JSON persistence, model restrictions, omitted optional argv groups, resume,
reassignment validation, result extraction and conditional errors. See the requirement and
implementation checklist in [docs/01](../../docs/01-requirements.md) and [docs/07](../../docs/07-implementation-plan.md).
Implementation checklist and evidence: [docs/07](../../docs/07-implementation-plan.md).

On the issue #9 Rust parity branch, the declarative profile editor and task-option controls
are integrated into the existing modular native UI. Profile edits are validated before they
replace the saved candidate; task model selection resolves planner overrides before profile
defaults, and model-specific option lists use that effective model. Local verification is
tracked in the issue #9 implementation checklist.
