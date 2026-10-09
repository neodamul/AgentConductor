# Session recovery and model catalogs

Observed: 2026-10-10 KST. Local CLI metadata only; no paid generation smoke.

## Native contracts

| Provider | New session | Resume and model | Catalog query |
|---|---|---|---|
| Claude Code | `claude -p --output-format stream-json --verbose` | Append `--resume SESSION_ID --model MODEL` | Stream-json control initialization; use returned `response.response.models[].value` |
| Codex | `codex exec - --json --sandbox workspace-write` | `codex exec --sandbox workspace-write resume SESSION_ID - --json --model MODEL` | Local `codex app-server` stdio: initialize, initialized, model/list |
| OpenCode | `opencode run --format json --file TASK_FILE MESSAGE` | Append `--session SESSION_ID --model PROVIDER/MODEL` | `opencode models`; no configured models means selection remains unavailable |
| Custom CLI | Explicit registered argv and parser | `{session_id}` and `{model}` placeholders; without a session placeholder a retry starts a fresh context-bearing invocation | Operator-observed catalog import |

Explicit provider session IDs are stored per task. Avoid global latest-session selectors.
Independent processes use independent sessions; serialize an individual resumed session.
Account quotas are shared across sessions. Set quota groups explicitly for profiles that
share credentials but use different executable paths.

## Evidence and limitations

- Claude Code 2.1.286: native control initialization returned a nonempty catalog using inherited login. MCP servers and tools were disabled for this metadata-only query.
- Codex CLI 0.162.0: model/list returned a nonempty catalog using inherited login. No turn/start was sent. Local exec resume help confirms model selection and exec sandbox option parsing.
- OpenCode: installed version probe succeeded, but configured-provider listing returned no model entries. No model IDs were invented. The user must configure a provider using the native CLI before discovery can succeed.
- A listed model does not promise remaining subscription entitlement. Headless allowances, provider policies, and pricing can differ from interactive usage. Authentication is inherited; the orchestrator does not convert or replace it.
- Concurrent provider execution has been verified with mock processes, not paid live tasks. CLI upgrades require re-query and contract verification.

## Recovery rules

Recognize explicit reset fields and offset-bearing message timestamps. Preserve original
reset, observation time, source, and raw logs. Wait at least 60 seconds before automatic
recovery and stop after three automatic retries. For several exhausted windows, wait for
the latest reset. Ambiguous local text remains waiting for a manual schedule. Manual
scheduling resets the retry allowance and applies to waiting sessions in the account group.

`watch` is foreground and polls persisted changes no more frequently than necessary;
exit retains schedules. `resume-due` processes overdue work under the run lock. A crashed
coordinator with a stale lock requires explicit recovery investigation; do not delete a
lock or resume an unknown live process blindly.

A role transfer retains partial owned files, scoped diff, evidence paths, and source session
identity. Never send one provider's session ID to another provider. Reject model edits while
running. Reassignment/model edits invalidate affected dependent work, test evidence, and
integration approval. Git refs retain snapshots against garbage collection.

## Primary sources

- [Claude CLI reference](https://code.claude.com/docs/en/cli-reference)
- [Claude SDK control initialization implementation](https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/_internal/query.py)
- [Codex App Server models and rate limits](https://learn.chatgpt.com/docs/app-server)
- [OpenCode CLI models, sessions, and run flags](https://opencode.ai/docs/cli/)

These sources informed protocol behavior; no SDK/framework code is bundled.
