# CLI subprocess contracts

Examples follow official documentation and local Codex help checked on 2026-10-09.
Execute only profiles passing version-specific contracts. Shell redirection below is illustrative;
Rust uses Command, argv, and pipes, without interpolating tasks into a shell. Use each task's
worktree as cwd.

## Claude Code

```sh
claude -p --output-format stream-json --verbose \
  --permission-mode acceptEdits \
  < task.txt > stdout.jsonl 2> stderr.log

claude -p --resume SESSION_ID --output-format stream-json --verbose \
  --permission-mode acceptEdits \
  < feedback.txt > resume.jsonl 2> resume.log
```

Write UTF-8 stdin then EOF. Add --include-partial-messages for token deltas.
Parse system/init.session_id, assistant content blocks, stream_event text_delta,
and final result fields result/is_error/session_id/usage/cost. Record stdout tool_use/tool_result
as well as stderr; tools do not necessarily log only to stderr. For single JSON output,
use --output-format json; distinguish .result and .structured_output from the event envelope.
acceptEdits is not a general sandbox or dangerous-command approval gate. Verify read-only
planning tool policies, implementation tool restrictions, and CLI sandbox separately.
Do not default to bypass flags.

Sources: [headless](https://code.claude.com/docs/en/headless),
[CLI reference](https://code.claude.com/docs/en/cli-reference).

## Codex

```sh
codex exec - --json --sandbox workspace-write \
  -o final.txt < task.txt > stdout.jsonl 2> stderr.log

codex exec resume SESSION_ID - --json \
  -o final.txt < feedback.txt > resume.jsonl 2> resume.log
```

Initial and resume options differ. Local resume help lacks --sandbox and -C; do not reuse
initial argv unchanged. Set Command.current_dir to the same cwd and verify resumed permissions
against the installed version. Avoid ephemeral because saved sessions are required.
Parse thread.started.thread_id, turn.started, item.started/updated/completed,
agent_message.text, command_execution exit/status, file_change, turn.completed.usage,
and turn.failed/error. Check process exit and turn success. -o stores the final message.
--output-schema result.schema.json structures the final response separately from JSONL events.
Do not assume noninteractive exec provides general synchronous human approval.
Reuse existing authentication; avoid --last during parallel execution.

Sources: [noninteractive](https://developers.openai.com/codex/noninteractive/),
[CLI reference](https://developers.openai.com/codex/cli/reference/).

## OpenCode

```sh
opencode run --format json --file task.md \
  "Follow the attached task scope and completion conditions" \
  > stdout.jsonl 2> stderr.log

opencode run --session SESSION_ID --format json --file feedback.md \
  "Revise this session according to the attached feedback" \
  > resume.jsonl 2> resume.log
```

Use documented --file instead of assuming stdin behavior. Verify JSONL framing and
sessionID/part shape with pinned-version fixtures. Normalize text, tool_use, step_start,
step_finish, error; one step_finish does not complete the entire task. Check exit and final
text/session export. Specify result.json separately if a standard final result is required.
opencode export SESSION_ID can collect additional evidence. Avoid --continue and default
--auto; smoke-test noninteractive permission denial/stop behavior.

The orchestrator has no API-key/endpoint registration. OpenCode's own provider authentication
is separate. Never copy Claude Code login tokens to OpenCode. Current official provider docs
restrict Claude subscription bypass; route Claude tasks through native Claude CLI. OpenCode
uses an already legitimately configured supported authentication route or local provider.

Sources: [CLI](https://opencode.ai/docs/cli/), [providers](https://opencode.ai/docs/providers/).

## Custom CLI and retry

Profiles use executable+argv without a shell, stdin task JSON or task files, and standard
JSONL {type,session_id,payload}. Self-hosted HTTP is a later optional transport, without requiring
an OpenAI-compatible completion endpoint. Follow [DD-07](../../docs/06-design-decisions.md)
for timeouts/process groups/attempts. Never auto-retry auth/config/permission failures.
Inspect diffs before resuming partial writes. Feedback supplies failing tests, conflict files,
and exact revision scope; use bounded attempts instead of unlimited agent conversations.
