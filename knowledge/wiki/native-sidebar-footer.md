# Native room and sidebar static labels

## Finding

The native Room showed a fixed “Local · CLI authentication” label, and the sidebar repeated a fixed local-first tagline. These statements were removed from the Rust UI because they duplicated configuration context and occupied space without reflecting per-agent state. CLI authentication remains owned by each registered CLI profile; the product's local-first behavior is unchanged.

## Change and verification

A regression checks both languages and fails if the removed labels return. The updated Room header, sidebar footer, language, and static-label suites pass 7 tests. Warning-denied Clippy, rustfmt, and diff checks pass. The issue branch now contains the latest Rust #9 base at `f123f2c`; exact-head GitHub Actions run `41287908205` is pending.

Graphify is being refreshed for the current branch. CocoIndex CLI is not installed in this environment. AgentMemory records this task's branch update, verification, and pending CI separately.

This is component-level evidence only. Exact packaged UI acceptance, constrained-window checks, and full keyboard/screen-reader traversal remain open. The accepted workflow count remains 0/25.
