# #959 — Automatic merge through reviewed fallback

Read-only local evidence confirmed enabled run consent and worker errors stating that consent no longer matched the executed plan. Reviewed-vs-executed field changes were task agent, fallback and prompt; audit records classified the handoffs as reviewed automatic fallbacks, not manual reassignments. No Goal, path, account, message or source content is reproduced here. A separate review-round gate remains authoritative.

A pre-fix regression reproduced the digest error in shared/worktree execution and successful exit on a blocked worker. The fix carries exact opt-in only across trusted reviewed automatic quota/checkpoint fallback transitions, audits both digests, and leaves manual edits and mismatched/disabled consent untouched. Stop, permission, tests, candidate binding and review gates remain.

Local verification: 32 focused Rust tests, including unchanged/changed three-Phase chains (Phase 1 auto-merges; changed Phase 2 receives plan approval and merges automatically), fallback protection and a real failed background worker exposed in the Room snapshot. The DOM notice appears once per failed job, independently of the human action job. All-target Clippy and formatting passed. No remote CI or physical Windows/provider execution is claimed here.
