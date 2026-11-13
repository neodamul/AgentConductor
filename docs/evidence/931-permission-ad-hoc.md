# Claude effective-permission ad-hoc verification

Issue: #931, milestone 0.2.8. Trace: R-122 / BG-77 / SF-109 / UC-112 / FR-107 / QAS-118 / DD-131.

## Execution boundaries

The portable Rust subprocess fixture is named claude and uses the real full-access preset, controlled process containment, event decoder, live observer, Git checkpoints and scheduler. It returns deterministic events and creates only disposable files. No API key, model service, desktop window or external MCP server is required by these process tests.

A separate installed Claude CLI probe sent only a control initialize request with tools/MCP disabled, read the response field names and stopped the owned process. It confirmed current_permission_mode is available without a user-generation prompt. No organization policy failure was reproduced live and no physical Windows acceptance is claimed.

| Scenario | Required behavior | Evidence |
| --- | --- | --- |
| Requested full access, actual default | Block before forwarding user prompt; no late file | claude_policy_runtime |
| Actual acceptEdits/plan/dontAsk/auto/empty | Refuse full-access claim | claude_permission_policy |
| Missing init permission metadata | No unverified successful completion | claude_permission_policy, claude_policy_runtime |
| Successful exit plus permission_denials | permission_denied, blocked run, no candidate/test/repair loop | claude_policy_runtime |
| Delayed denial | Same blocked outcome after streamed progress | claude_policy_runtime |
| Partial file before denial | Keep owned partial work | claude_policy_runtime |
| Shared and worktree modes | Same safety outcome in both | claude_policy_runtime |
| Unrelated quota text on stderr plus explicit permission denial | Keep blocked; do not schedule availability retry | claude_policy_runtime |
| Read-only coordinator using full-access worker profile | Accept restricted success, reject actual denials | claude_permission_policy |
| Available fallback on permission denial | No automatic quota handoff; human chooses | claude_policy_runtime |
| Explicit reassignment then resume | Fresh target session; partial file preserved; approval still required | claude_policy_runtime |
| Confirmed bypass, ordinary success | Completed task and review candidate; no automatic merge | claude_policy_runtime |
| Mode changes to default after init | Normalize as permission-policy error | claude_permission_policy |
| Malformed denial metadata | Fail closed | claude_permission_policy |
| API usage-limit response | Preserve real quota error instead of inventing permission failure | claude_permission_policy |
| Custom provider-specific field | Do not apply Claude semantics to another provider | claude_permission_policy |
| Denial tool input contains private content | Normalized message omits tool input; owned raw spool remains local | claude_permission_policy |
| Metadata query | Initialize only, same requested permission mode, record actual mode and profile fingerprint | claude_policy_runtime |
| Asynchronous registration verification | Save stays nonblocking; settled snapshot shows mismatch | claude_policy_runtime |
| Stale/different profile observation | Cannot verify the current profile | claude_permission_policy |
| Explicit callback selection | Pinned callback argv and mode survive repeated saves | claude_callback_profiles |
| Callback with unsupported argv | Save rejects, never silently becomes automatic approval | claude_callback_profiles |
| Editor in both themes and 840/1280 | Checkbox geometry, save and reopen work | claude_permission_setup.cjs |
| Approved reference | Callback control matches typography and stays usable | claude_permission_spec.cjs |

## Broader application coverage

Workspace registration/default coordinator, Goal rules/settings/deletion, project directory selection, run binding, return after completion, workflow/folder execution, Stop/resume, failure recovery, quota fallback and phase progression are included in the additional local regression run. The full root Node suite exercises UI navigation, planning feedback, MCP previews, approvals, result transitions and error paths. Final command results are recorded in docs/07 and the issue/PR.

## Limits

Fixtures do not prove live Enterprise policy behavior, real subscription limits, company wrappers or an installed Windows desktop. Remote CI status must be reported separately. No release is part of this issue.

## Final local results

- 68 focused Rust tests passed, covering actual contained subprocesses as well as parsers and configuration.
- 47 broader Workspace/Goal/workflow tests and 19 documentation/architecture/i18n tests passed.
- Library, Stop, fallback and workflow checks passed again after the final denial-priority change.
- All 103 root Node UI scripts passed; the callback editor was rechecked separately in light/dark at 840/1280 after tightening checkbox-size assertions.
- Eight focused parity checks passed; reference interaction checks passed 183 assertions in each of four theme/width combinations. Six affected reference images were refreshed, and both callback images were inspected visually.
- Clippy with warnings denied, changed-source formatting, 300-line limits and design-manifest integrity passed. English/Korean offline HTML was regenerated by the Rust manual renderer.

The Stop test used a 500ms worker, which could finish before the separate Stop CLI started under load. It now uses the existing slower fixture so the test exercises stopping a running process rather than racing completion. The policy-plus-quota-noise scenario reproduced an actual incorrect waiting_availability classification; explicit Claude denial now takes precedence over unrelated stderr quota text.
