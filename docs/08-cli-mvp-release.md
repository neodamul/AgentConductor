# CLI MVP release verification

Date: 2026-10-10 KST. Source commit: da9f353 (Rust implementation).

## Delivered behavior

- Three native CLI profile templates and custom CLI registration; inherited authentication.
- Three-task planning via a read-only coordinator, validated DAG and disjoint scopes.
- Parallel subprocess execution in a shared workspace or optional Git worktrees.
- Local stdout/stderr, normalized JSONL, provider session IDs, attempt artifacts, diff, tests, SQLite audit/outbox, and local recall.
- Exact-plan consent and exact-candidate approval before merge; candidate/test/index changes invalidate evidence.
- Explicit reset parsing, persisted manual schedules, foreground due-work execution, retry delay/ceiling, and account quota groups.
- Independent task sessions, per-profile concurrency cap, audited role transfer, bounded context handoff, and catalog-validated session model changes.
- One native executable with bundled SQLite; optional memory/indexing tools are not runtime requirements.

## Verification

- `rtk cargo test`: 59 tests across 17 suites pass.
- `rtk cargo clippy --all-targets -- -D warnings`: clean.
- `rtk cargo fmt --check` and `rtk git diff --check`: clean.
- `rtk cargo build --release --locked --offline -j 2`: succeeds.
- Packaged release-binary E2E with three mock profiles and Git worktrees: concurrent work, all tests pass, unapproved merge denied, exact approval merges candidate, original workspace remains clean.
- Native Claude Code 2.1.286 and Codex CLI 0.162.0 metadata catalogs: queried successfully using existing login, no generation prompt. OpenCode v2.0.25 run/model/session flags were verified; its configured model listing returned no entries. Its default shared service and optional `--standalone` mode must be included in operator sandbox configuration.
- Startup/help measurement: maximum RSS 2,195,456 bytes and peak footprint 1,343,896 bytes; this excludes native agent workloads.
- Binary is 3,170,736 bytes; system dynamic dependencies are libiconv and libSystem only. No external SQLite library is required.
- Packaging inventories 42 target-resolved Rust dependencies, verifies reviewed license declarations, includes license texts and SQLite/Unicode notices, and emits a CycloneDX component inventory.

Package: `dist/agentconductor-aarch64-apple-darwin/` contains the binary, SHA256SUMS,
manifest, verification evidence, README/images, Cargo.lock, SBOM, and license notices.
Runtime requires only the executable, Git, and selected authenticated CLIs. Python is used
only by the developer packaging script.

## Practical limits

The UI remains a labeled mockup. Paid live generation and concurrent native model execution
have not been smoke-tested. Metadata discovery does not guarantee entitlement or quota.
The initial profiles require verified permission/sandbox configuration. Git worktrees isolate
code state, not host secrets, network access, or subprocess permissions.

Crash/stale-process ownership recovery, explicit user stop, source revalidation and runtime
lessons remain unverified backlog items. A stale run lock is retained for investigation;
never resume an unknown live process or delete a lock blindly. The private root project has
no public license grant; third-party notices do not grant one. Provider terms and any optional
service distribution require their own organizational review. Windows process containment
and cross-platform release artifacts are not delivered by this macOS ARM64 package.
