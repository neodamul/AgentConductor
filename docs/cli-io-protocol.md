# CLI I/O protocol (acio) — design draft v1

Status: **design for review** (GitHub #205). Nothing here is implemented yet. The schemas and examples under
[`schemas/cli-io/v1/`](../schemas/cli-io/v1/) are real JSON Schema (draft 2020-12) and every example in
`examples/valid` and `examples/invalid` was checked against them.

## 1. Purpose

AgentConductor talks to agent CLIs through stdin, stdout, stderr and the exit code. Today each provider has a
private adapter (`src/protocol.rs`) and a `custom` CLI is described by JSON pointers over whatever it prints.
That works for the CLIs we know and does not scale to an in-house LLM agent CLI: the owner has to learn our
pointer syntax, stderr is read for hints, and exit codes mean nothing.

acio is one small contract that a CLI can implement directly, or that a thin wrapper can implement for it:

| Channel | Direction | Carries |
|---|---|---|
| stdin (or the task file for the document) | supervisor → CLI | first line: one `acio.input/1` document; later lines (only when `control.enabled`): `acio.control/1` messages |
| stdout | CLI → supervisor | JSON Lines, one `acio.event/1` per line; nothing else |
| stderr | CLI → humans | diagnostics; never control information |
| exit code | CLI → supervisor | the coarse outcome; the events win when both exist |
| `<cli> --acio-describe` | CLI → supervisor | one `acio.describe/1` document: versions, capabilities, how to call it |

Goals: schema-first (every message validates), small enough to implement in a shell script, additive evolution,
and no weakening of the supervisor's own guards (read-only means read-only whatever the CLI claims).
v1 is bidirectional in its final form: besides the event stream, the supervisor can send `cancel`, `user_message` and `tool_decision` while the CLI runs (§6). Each CLI declares which of these it honors; the adapters for `claude` and `codex` map them to what those CLIs really support (§9). Non-goal: a network transport.

## 2. Input — `acio.input/1`

One UTF-8 JSON document, at most 1 MiB. Delivered on stdin, or in the file named by the `{task_file}` argument
when `invocation.input` is `file`; the document is the same either way. Schema: `input.schema.json`.

| Field | Meaning |
|---|---|
| `schema`, `request_id` | `acio.input/1`; an opaque id echoed in diagnostics |
| `mode` | `ask`: read-only question or planning, no workspace change. `task`: may change files inside `workspace.paths` |
| `role` | `coordinator`, `worker`, `reviewer`, `assistant` |
| `instruction.text`, `.language` | the standing instruction and the reply language (`ko`, `en`) |
| `instruction.examples[]`, `.guidance_files[]` | few-shot pairs; workspace-relative instruction files read by the CLI instead of inlined text |
| `materials[]` | source documents; the Document block, always before the Task (§2.2) |
| `task` | the kind of work and its conditions (§2.1): `type`, `effort`, `completion`, `stop_conditions`, `evidence`, `uncertainty`, `report`, `open_ended`, `variants` |
| `conversation[]` | earlier turns, oldest first (at most 40 turns of 8 KiB); the last operator turn is the question |
| `workspace` | `root`, `access` (`read_only` or `workspace_write`), `paths` the CLI may change; `ask` forces `read_only` |
| `session.resume_id` | resume a prior session when the CLI declares `resume` |
| `model` | `id` and string `options` (for example `effort`) from the CLI's own catalog |
| `limits` | `timeout_ms`, `max_event_bytes` (default 262144), `max_output_bytes` |
| `response_schema` | optional JSON Schema that `result.data` must satisfy (typed planner or optimizer replies) |
| `accept.versions` | event schema majors the supervisor reads (default `[1]`) |
| `x-*` | vendor extensions, ignored by everyone else |

Unknown fields without the `x-` prefix are rejected, so a typo is an error instead of silent behavior.
Secrets are never in the input or in arguments; a CLI keeps its own login.

### 2.1 Task types and per-model conditions

Presenting every model with the same conditions is not how current models are best used: a classifier wants a
cheap model with examples and a schema, a long-running agent wants high effort, stop conditions and a report
format. The input therefore says what kind of work it is (`task.type`), and the schema requires what that kind
needs. Without a `task` block the CLI gets the neutral defaults and no extra conditions.

| `task.type` | Effort and model | Prompt layout | Required by the schema |
|---|---|---|---|
| `extract` (classify, extract, convert format) | low effort, or a non-reasoning model | few-shot `instruction.examples`; output fixed by schema | `response_schema` |
| `summarize` (report from long material) | medium | facts separated from interpretation; Document, Task, Format as for every type | `materials`, `task.evidence = cite_sources` |
| `analyze` (analysis, decision support) | medium | decision criteria in the instruction; say what is uncertain | `task.stop_conditions`, `task.uncertainty` |
| `code` (write, debug) | high | goal, constraints, completion criteria; tests decide when it is done | `task.completion.criteria` and `.verify` (argv lists) |
| `agentic` (long, multi-step, tools) | high | scope, stop conditions, report format; standing rules in guidance files, not in the prompt | `task.stop_conditions`, `task.report` |
| `creative` (ideas, copy) | free | say there is no single right answer; ask for variety | `task.open_ended = true`, `task.variants` (2–10) |

**Resolution.** The effective model and effort are the first defined of: the operator's choice for this task;
the profile's execution options (for example `model_values`, which already limit effort levels per model);
the CLI's `task_defaults[type]` from `--acio-describe`; the built-in default of the table. The result is
clamped to the model's `traits`: an effort the model does not honor is dropped with a `warning`
(`effort_unsupported`), a non-reasoning model on a `code` or `agentic` task gets a `warning`
(`model_not_suited`) — facts shown to the operator, never a silent change and never a block. The CLI reports
what it applied in `session.data.effective` (`model`, `effort`, `task_type`). One plan can therefore mix models
and efforts per task: classification on a fast model, implementation on a high-effort one, review on `analyze`.

### 2.2 Prompt assembly order: Document, Task, Format

For every task type, whoever turns the input into a prompt (a native CLI, a wrapper or the supervisor's adapter
for a provider CLI) assembles it in three blocks, so the long material is read before the work that depends on
it and the shape of the answer is the last thing the model sees:

1. **Document**: `materials[]` in array order, each delimited by its `id` and `title`, then the earlier `conversation[]` turns, oldest first.
2. **Task**: `instruction.text`, the conditions rendered from `task` (completion criteria and `verify` commands, stop conditions, evidence and uncertainty rules, variants, open-endedness) and the current question (the last operator turn).
3. **Format**: how the answer must look: `instruction.examples[]` (few-shot pairs show the format), `response_schema`, `task.report` (format and sections) and the reply language.

`instruction.guidance_files` are not inlined: the CLI reads them from the workspace, so standing rules stay out
of the prompt and are not repeated per task. A conforming CLI that deviates says so in a `x-` extension of its
describe document; the conformance check (§11) looks at the section order of a debug echo of the assembled
prompt, not at the model's output.

## 3. Output — `acio.event/1` on stdout

JSON Lines: UTF-8, LF, one object per line, no ANSI, no banners. Common envelope: `schema`, `seq`, `ts`
(RFC 3339 UTC), optional `session_id`, `type`, `data`. Schema: `event.schema.json`.

| `type` | `data` (required fields) | Maps to `EventKind` |
|---|---|---|
| `session` | `session_id`; optional `resumed`, `model`, `cli{name,version}` | Session |
| `progress` | `message`; optional `phase`, `fraction` | Progress |
| `message` | `role`, `text`; optional `delta`, `final` (streamed answer) | Progress |
| `tool` | `call_id`, `name`, `status` (`requested/running/completed/failed/denied`); optional `input`, `output_summary`, `exit_code`, `needs_decision` | Tool |
| `usage` | optional tokens, `cost_usd`, `rate_limited`, `retry_after_ms`, `quota{used,limit,unit,resets_at}` | Usage |
| `warning` | `code`, `message` | Warning |
| `error` | `code`, `category` (`auth/quota/capacity/input/tool/timeout/internal`), `message`, `retryable`; optional `retry_after_ms` | Error |
| `heartbeat` | optional `activity` | Progress |
| `ack` | `control_seq`, `status` (`applied/ignored/unsupported`); optional `message` (answers one control line) | Progress |
| `result` | `status` (`success/failure/partial/cancelled`), `summary`; optional `data`, `artifacts`, `changed_paths` | Result |
| `x-<vendor>` | free object | Unknown (kept raw) |

Stream rules (checked by the future validator, not by JSON Schema alone):
1. `seq` starts at 1 and grows by exactly 1; `ts` never goes backwards.
2. The first event is `session` when the CLI has a session (required when resuming).
3. The last line is exactly one `result`. An `error` event is informative; it does not end the stream.
4. A silent CLI sends a `heartbeat` at least every `capabilities.heartbeat_seconds`, so silence is a fact and not a guess.
5. A line longer than `max_event_bytes`, an invalid line or a duplicate `result` ends the run as a protocol failure (the raw line is kept as evidence).
6. If `response_schema` was given, `result.data` must satisfy it, otherwise the run is a failure with `category: input`.

## 4. stderr

Free text for humans. A CLI may print `acio.log/1` JSON lines (`level`, `message`, `ts`) but the supervisor never
branches on stderr content. It stores a bounded tail (the first 1500 characters already shown for coordinator
failures stay as they are), redacts known secret shapes, and shows it next to the failure. Anything that must
change behavior (quota, retry, auth) is an `error` or `usage` event.

## 5. Exit codes

| Code | Meaning | Matching event |
|---|---|---|
| 0 | finished; `result.status` is `success` or `partial` | `result` |
| 1 | the task failed | `result.status = failure` |
| 2 | invalid input or usage (schema violation) | `error.category = input` |
| 3 | unsupported schema version | `error.code = unsupported_version` |
| 69 | backend unavailable (EX_UNAVAILABLE) | `error.category = capacity`, `retryable` |
| 75 | temporary failure: quota or capacity (EX_TEMPFAIL) | `error.category = quota/capacity`, `retry_after_ms` |
| 124 | killed by the supervisor on timeout (set by the supervisor, never by the CLI) | none |
| 130 / 143 | interrupted / terminated by signal | `result.status = cancelled` when possible |

Precedence: a valid `result` event decides the outcome; the exit code is the fallback when the stream has no
`result`; a code that contradicts the `result` (0 with `failure`) is recorded as a protocol warning.

## 6. Control channel, cancellation, timeout and sessions

When `input.control.enabled` is true and the CLI declares any `capabilities.control` flag, stdin stays open after
the input document and carries `acio.control/1` JSON lines (`control.schema.json`); otherwise stdin is closed
after the document and only signals can cancel.

| `type` | `data` | Effect |
|---|---|---|
| `cancel` | optional `reason` | Stop at the next safe point, flush a final `result` with `status: cancelled`; the supervisor still follows with the process-group signals (SIGTERM, grace, SIGKILL; on Windows the console break) |
| `user_message` | `text` (at most 8 KiB) | New operator input while the CLI runs, as a further user turn |
| `tool_decision` | `call_id`, `decision` (`allow`/`deny`), optional `reason` | The answer to a `tool` event with `status: requested` and `needs_decision: true` |

Rules: control lines carry `seq` from 1 and the CLI answers each with an `ack` event (`applied`, `ignored`,
`unsupported`). A tool request that gets no decision within `input.control.decision_timeout_ms` (default 120 s)
is denied; a silent supervisor never grants anything. A control type the CLI did not declare is not sent; if it
is sent anyway the CLI acks `unsupported`. `limits.timeout_ms` is enforced by the supervisor. Resuming uses
`session.resume_id`, taken from the earlier `session` event. The supervisor's own guards (permission mode, scope,
read-only) stay in force whatever the CLI is told.

## 7. Discovery — `acio.describe/1`

`<cli> --acio-describe` prints one JSON document and exits 0 within 5 seconds: `name`, `version`,
`acio.versions`, `capabilities` (`read_only`, `streaming`, `resume`, `tools`, `response_schema`,
`heartbeat_seconds`, `languages`, `task_types`, `task_defaults`, `models` with per-model `traits`: `reasoning`,
`efforts`, `context_tokens`, `structured_output`, `best_for`), and `invocation` (`input` = `stdin|file`, fixed `args`). Registering
a CLI then needs only its path: the profile is derived from the document instead of being written by hand. A CLI
that does not answer is registered with the declarative profile as today. Schema: `describe.schema.json`.

## 8. Versioning and extension

- The `schema` string carries the major: `acio.event/1`. Within a major, changes only add optional fields, enum values consumers may treat as unknown, and `x-*` event types.
- Consumers ignore unknown optional fields inside `data` and unknown `x-*` events; a producer never relies on a consumer understanding them.
- A breaking change is `/2`. The supervisor sends `accept.versions`, the CLI lists `acio.versions`; they run on the highest common major or the run fails before starting with `unsupported_version`.
- Vendor data goes in `x-<vendor>-…` fields and `x-<vendor>` events (the in-house tenant, ticket or audit id in the examples).

## 9. How this fits today's code

- `protocol::Provider` gains `Acio`. `Decoder` for `Acio` needs no mapping tables: `type` → `EventKind` per the table above, `session_id` from the envelope, the whole line is the `payload`. Persisted `Event { seq, raw_offset, session_id, kind, payload }` is unchanged, so the planner, guard, room and replay keep working.
- `claude`, `codex`, `opencode` keep their adapters. Their normalized events are the same shape, so fixtures and the validator can be shared.
- The declarative `custom` profile (pointers) stays for CLIs nobody can change.
- `ExecutionOption` (`values`, `model_values`, `default`) already limits options per model; `traits.efforts` is the same fact declared by the CLI itself and feeds it. A plan task's `test_command` becomes `task.completion.verify`.
- Today `chat::build_prompt` puts the rules first, then the manual, then the conversation, and mixes the output format into the rules. Moving to §2.2 puts the manual and the conversation first (Document), the rules and the current question next (Task) and the reply language plus the JSON shape last (Format); the planner and optimizer prompts need the same review. This is a phase 3 change with its own tests.
- Task types are not tied to acio: a task profile derived from `task.type` also selects prompt templates and defaults for the `claude`, `codex` and `opencode` adapters, so existing CLIs get per-task conditions without implementing the protocol.
- `InputMode::Stdin|File` already exists and carries the input document unchanged. `adapters::session_args` keeps building model and resume flags for the known providers; an `Acio` CLI declares its flags in `invocation.args`.
- Agent characteristics are settings: see the end of §14.
- Read-only is enforced by the supervisor (permission mode, guard) whatever `capabilities.read_only` claims; the capability only says the CLI will not try.

## 10. Adopting it for an in-house LLM agent CLI

An in-house CLI is covered by the per-agent settings that already exist (the declarative `custom` profile: argv
template, `event_types`, `session_id_pointer`, `result_pointer`, `error_condition`, execution options). Nothing
new has to be built for it. The protocol adds two optional upgrades the owner can adopt at their own pace:

| Level | What the owner does | Result |
|---|---|---|
| 0 per-agent settings | nothing beyond today's profile | works as now: partial status, no streaming text, no exit-code meaning |
| 1 `parser: acio` | the CLI prints `acio.event/1` lines (and may answer `--acio-describe`); the agent setting is `parser: acio` and its argv | full contract, richer data (usage, tools, typed `result.data`), no pointer mapping |

`examples/minimal-cli.sh` shows a conforming CLI in 25 lines. An agent that returns structured plans or reviews
uses `response_schema` and `result.data`; the supervisor validates the reply instead of scraping text.

## 11. Tooling to build (phases 1 and 6)

- `agentconductor io schema <input|event|describe|control>` prints a schema from the binary, so an agent without the repository has it.
- `agentconductor io validate <input|events|describe|control> <file|->` checks a document or a captured stream, including the stream rules in §3, and prints the first violation with its line.
- `agentconductor io conformance <exe> [--ask|--task]` feeds canned inputs to a CLI, validates what comes back and reports a pass/fail table (including cancel within the grace period and the oversize-line case).
- A shared fixture set in `schemas/cli-io/v1/examples` used by Rust tests; the manual gets an "Integrating a CLI" chapter.

## 12. Alternatives considered

| Option | Why not (v1) |
|---|---|
| Keep pointer-only profiles | the owner must reverse-engineer our mapping; no exit-code, usage or typed-result semantics |
| JSON-RPC 2.0 over stdio (the style of MCP and agent-client protocols) | needs request/response id correlation and a framed transport; harder for a shell script. Bidirectional control is covered here by plain control lines and acks. Not evaluated in depth |
| Plain text with markers | ambiguous with model output; cannot be validated |
| One huge JSON on exit | no progress, no streaming, no heartbeat; a crash loses everything |

## 13. Phases

Decided on 2026-10-28: implement the final form directly, not a one-way first step. Each step is its own issue and PR.

1. Core: the four schemas embedded in the binary, Rust types, a subset JSON Schema checker (no new dependency), stream rules, `agentconductor io schema|validate`, tests over the shared examples.
2. Runtime: `Provider::Acio` decoder, `parser: acio` in agent settings, `--acio-describe` registration, the control channel in the process supervisor (stdin lines, acks, decision timeout, cancel).
3. Agent settings instead of code: `acio.agent/1` merged from preset, describe and profile; the `claude` and `codex` presets as embedded data; the argument branches of `src/adapters.rs` moved into those settings; control, effort and response schema through templates (§14).
4. Task types in plans: the coordinator sets `task.type` per task, resolution against `traits` and `task_defaults`, warnings, effective settings in the session event, the Agent roles table shows per-model effort and fit.
5. Prompt assembly Document, Task, Format for chat, planner and optimizer.
6. `io conformance` and the manual chapter.

## 14. Decisions (2026-10-28, user)

1. Names stay: `acio`, `--acio-describe`, `x-<vendor>`; the company has no naming rule.
2. Bidirectional control is part of v1, implemented in its final form. Claude and Codex differ, so the mapping is decided here (below).
3. In-house CLIs are covered by per-agent settings (§10); no wrapper deliverable.
4. Validation in Rust: a small checker for the JSON Schema subset the four schemas use (`type`, `enum`, `const`, `required`, `properties`, `patternProperties`, `additionalProperties`, `items`, bounds, `pattern`, `if/then`, `allOf`, local `$ref`), checked against the shared examples. No new dependency, in line with the minimal-resources goal.
5. `result.data` is mandatory for a successful `result` when the input role is `coordinator`; the stream validator enforces it (it needs the input, so JSON Schema alone cannot).
6. The coordinator sets `task.type` per plan task; no operator control for now. A task without a type gets `code` for a writing worker, `analyze` for a reviewer or chat, `agentic` for a multi-step task.
7. The Agent roles table shows, per model, the efforts it honors and the task types it suits.

8. Agent characteristics are settings, never code (decided 2026-10-28). Everything that differs between agent CLIs is data in the agent's settings: capabilities, control encodings, argument templates for model, resume, effort, response schema and full access, models with traits and task defaults. No Rust branch says "if the agent is Claude".

### Agent characteristics as settings — `acio.agent/1`

`schemas/cli-io/v1/agent.schema.json` is the shape of those settings. It is the same shape as the capability part
of `acio.describe/1`, so a CLI's own `--acio-describe` can fill it. Sources, later overriding earlier:

1. a **shipped preset** (`schemas/cli-io/v1/presets/claude.json`, `codex.json`; embedded in the binary as data files, not as match arms) selected by the profile's `preset` name;
2. the CLI's `--acio-describe` document, when it answers;
3. the **operator's profile settings** for that agent, edited in the agent dialog or the settings file.

An in-house CLI has no preset: its owner fills the same fields in its agent settings (see `examples/valid/agent-inhouse.json`).

| Field | What it replaces in code today |
|---|---|
| `invocation.args`, `control_args`, `full_access_args` | the per-provider argv templates and `full_access_args(Provider)` in `src/adapters.rs` |
| `invocation.session.model_args`, `.resume` (`append` or `replace_argv`) | the `match profile.parser` branches that add `--resume`, `--session`, `exec … resume` |
| `invocation.options` (effort, …) | already settings as `execution_options` (`values`, `model_values`, `default`); same shape, now also carries Claude's five effort levels and Codex's `-c model_reasoning_effort` |
| `invocation.response_schema` (`args` or `file_args`) | nothing yet; Claude `--json-schema`, Codex `--output-schema` |
| `invocation.control` (`cancel`, `user_message`, `tool_decision` encodings) | nothing yet |
| `capabilities` (`control`, `streaming`, `resume`, `heartbeat_seconds`, `task_types`, `models[].traits`, `task_defaults`) | nothing yet |

Event decoding (turning a CLI's own JSON into normalized events) stays code for `claude` and `codex` because it
needs conditions a settings file cannot express; a new CLI never needs it, it prints `acio.event/1` or uses the
declarative pointer mapping. The supervisor reads the merged settings: it renders arguments from templates,
filters the control messages by `capabilities.control` and `unless: full_access`, and checks the response schema
flags. The shipped presets record what was found on this machine (Claude Code 2.1.308, Codex 0.165.0): Claude
takes stream-json input, five effort levels, `--json-schema` and `--permission-prompts host` (usable only when the
profile is not full access, because the default profile bypasses prompts, DD-131); Codex `exec` is one-way, takes
`--output-schema`, effort through `-c`, and is cancelled by signal; its two missing controls are one setting
change when `app-server` leaves the experimental stage.
