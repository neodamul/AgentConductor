# Quota, fallback and native status evidence

Trace: R-22/DD-26, R-25/R-26/DD-28, R-29/DD-31.

## Contracts

Rust remains the execution authority without a coordinator LLM. Codex quota metadata uses registered `app-server` stdio initialization followed only by `account/rateLimits/read`; it requests no generation or reset. Independent reported windows retain used percentage, derive bounded remaining percentage and clear stale/future/expired capacity. Missing values and context/USD fields never become subscription balance. Private observations remain in ignored local evidence.

Per-profile fallback targets/models are part of exact plan consent. Unknown targets, cycles, same-account substitutes and unobserved models fail validation. Reachable fallback permissions are checked before execution. Restricted source accounts remain durable after task reassignment. Owned jobs drain and recorded process groups must be absent before scoped transfer. Handoff preserves partial files/logs, clears source native identity and requires a new target session. Restricted targets wait; ordinary failures and human stops do not cause policy transfer. Integration still needs tests and exact human approval.

Native session lists map running to pulsing green, completed to steady green, waits/stops/unknown to yellow, failed/blocked to red. Exact state labels and reduced-motion CSS retain accessibility. Registered account selection permits quota inspection independently of assigned tasks.

## Verification

- Red: missing quota module/query/persistence and missing fallback profile field failed before implementation.
- Final full regression: 162 passed, 4 ignored, 0 failed, 43 suites, 96.16 seconds. Clippy/rustfmt/whitespace and JS syntax checks pass.
- Eight fallback tests cover shared/worktree operation, preserved partial files, policy absence, malformed chains/model/account/permissions, ordinary errors, restricted targets, same-account renaming and operator stop.
- Actual Codex authenticated metadata queried through CLI and displayed via native account selector/refresh with independent reset/source/freshness. No generated task was requested by that query.
- Actual native fixture showed waiting/failed/running simultaneously; two screenshots show different green pulse intensity while yellow/red stayed steady. UI Stop produced stopped state; coordinator/UI and three worker PIDs were all kernel-absent after terminal exit 0. Evidence retained under `.agentconductor/verification/status-lights-live-20261010/`.
- Graphify AST extraction: 1,129 nodes/2,601 edges, no LLM. CocoIndex structural grep identified apply_fallbacks. No Cargo dependency/license closure changed. Ponytail was unavailable as a callable capability; no plugin execution claimed.

## Remaining acceptance

Claude status-line quota parser is tested, but authenticated headless capture is not implemented. Windows CI run 38022076235 passed macOS checks and failed Windows compilation at Unix-only run_controlled_async; native Windows process/filesystem/lease/desktop safety and releases remain open. Native fallback-registration/model/consent interaction, metadata error/timeout boundaries, measured resources and live reduced-motion settings need more acceptance evidence. Cooperative agentmemory directives (R-27/DD-29), on-demand briefings (R-28/DD-30), fine-grained idle detection and optional designated coordinator-session lifecycle remain unfinished. This page is checkpoint evidence, not full-product completion.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/quota.rs` | `135d1d84d6dcea3843b165e866991d97d7ffb0b9c483cfb091e668605ad83a83` |
| `src/plan.rs` | `95a31e59e1d0b46d114f8c11797eb8a4a2e9970292859db0b735607b40348cd6` |
| `src/workflow.rs` | `4d33e1137e99ef85a5c24db9e6112278ee2e8a245adf1d49cda5374d00d80083` |
| `src/ui.rs` | `72841c0376ed8fc8c537f66c2317d39e5572ed3c8b8306537e4d8d8ad412e376` |
| `src/ui/room.js` | `538e7e6d656b2e5973afc4759551ae5aa21d678fb60939d59a22e972fdb47cd2` |
| `src/ui/room.css` | `749c8034ed086b6e5c5ef795aef3be3154de0b0a7c4652bf4f0048c94c90e28e` |
| `src/ui/startup.js` | `c73ab97e5abcb8c3a0fc9747a28f03a5bac78175c45f79bbe9c4f0242ae9a7f5` |
| `tests/quota.rs` | `ef372d6a19068550965a5ba72107fddd365123ef6aa5ed8e9d36dd6fbff7ff94` |
| `tests/fallback.rs` | `2b8dc59cfd52f3dee39d1ce7675b57d908b2e64ae984b9635b968fe461dd298b` |
| `tests/cli_workflow.rs` | `f3a192758615151f6c0849e8e37e67935f69656860f87fff8fb0e76245d68613` |
| `tests/recovery.rs` | `4107a72fdaa2a9ec5981d6cec4916b94ba135cf291cc0812f2a7f6620aae6831` |

Additional ownership-loss regression: removing the missing-owner guard reproduced the failure; restoring it passed the focused test. The preceding full regression covers the same production guard; the additional unit case was then run independently. Missing attempted-worker ownership now refuses fallback instead of assuming process absence.
