---
name: agentconductor-release
description: Update AgentConductor README and bilingual offline manuals from current source, then validate and publish stable macOS and Windows GitHub releases with the manuals included. Use for AgentConductor documentation-and-release requests.
---

# AgentConductor release

Use the Rust packaging CLI and existing GitHub workflows; do not introduce a release framework. Publishing requires the user's release request. Keep user communication in Korean, public README and English manual in English, and maintain the requested Korean manual.

## Plan: establish the release source

- Confirm the Git remote and actual repository directory before edits. A stale backup checkout must not replace the active project. Fetch remote changes and tags; inspect the clean/dirty state, latest commits, published releases, Cargo version and CHANGELOG. Preserve other agents' work and coordinate overlapping ownership.
- Use the requested version when available. If already published, preserve that release and choose the next stable patch; do not overwrite published tags or assets. A failed unpublished tag may only be repaired within the user's authorization, checking its exact remote ref and using a narrow force-with-lease if needed.
- Read new source and its tests, not only commit titles. Map implemented changes to manual sections; exclude pending design requests. Track Plan, Task, Verify and Complete in docs/07-implementation-plan.md. Add requirement-to-design trace for new product behavior; documentation refreshes can reference the existing requirements.

## Task: synchronize public documentation

- Update README.md, docs/user-manual.md and docs/user-manual.ko.md. README stays a short public product introduction (under 120 lines), with no internal plan or decisions. Keep versions consistent with Cargo.toml/Cargo.lock and the matching CHANGELOG tag section.
- Follow docs/design-guide.md when UI captures change. Capture actual native UI from the intended build, not mockups; preserve the operator's workspace, language and theme. Stop only the previous owned app before starting another. Refresh affected screenshots when their UI changed; unchanged screenshots need not be recreated merely for a version number.
- Run `target/release/agentconductor manual render` after any manual or referenced-image edit. The offline Markdown renderer is implemented in Rust and parity-tested. Markdown is the authoring source; both generated HTML files are committed and embedded in the binary. HTML must retain semantic tables and offline images. Do not hand-edit generated HTML.
- README links must include both Markdown and HTML manuals. The Rust package assembler copies README-linked docs/assets into the package; check `src/release_package.rs` if paths change. Keep the Rust renderer, tests/manual_renderer.rs, tests/manual_menu.rs and tests/public_docs.rs connected. No additional Markdown renderer, UI server or runtime dependency is needed.

## Verify: build and package the exact commit

- For documentation-only edits, use meaningful existing docs/renderer/package tests. Product logic changes follow TDD with a failing regression before the fix. Do not weaken or skip tests to obtain a release.
- Run the applicable checks: `cargo test --all-targets --locked --no-fail-fast`, `python3 -m unittest discover -s tests -p 'test_*.py'`, `cargo fmt --check`, and `cargo clippy --all-targets --locked -- -D warnings`. Use bounded Cargo jobs locally to limit resource consumption. Prefix shell commands with rtk where the environment requires it.
- Verify changed native manual behavior with English/Korean and Light/Dark UI as applicable. Distinguish fixture/CI checks from physical-device or authenticated-provider checks; do not claim the latter without evidence.
- Commit the intended source before packaging. `cargo build --release --locked` then `target/release/agentconductor package-release` creates and tests the exact native package. Package manifest source_commit must identify that clean commit; retain licenses, third-party notices, SBOM, manuals, screenshots and SHA256SUMS.
- Push the source and stable tag to invoke .github/workflows/release.yml. Both macOS and Windows validation and exact-package smoke jobs must pass before publication. Inspect failed job logs and repair the cause; retry unchanged work only when evidence supports a transient failure. Do not publish an unverified package or bypass the workflow gates. Preserve relevant failure evidence. If duplicate main/tag validation runs exist for the same commit, keep the release run and cancel only a redundant run owned by this task.

## Complete: verify the published artifacts

- Wait for the entire Release Action, including publish, to succeed. Confirm the GitHub release is stable, not draft/prerelease, with both platform ZIPs and SHA256SUMS. Starting CI is not completion.
- Download those published files to a task-owned temporary directory. Verify archive SHA256SUMS and each package's internal SHA256SUMS against raw bytes, then source_commit and the English/Korean HTML manuals against the intended tagged source. Normalize LF/CRLF only for text-content comparison, never for file hashes. Do not compare against a checkout changed by another agent after tagging.
- Finish the plan checklist with Action/release URLs and honest test coverage; commit/push that completion record separately from the frozen release source. Resolve concurrent documentation edits by retaining both records, never force-pushing main.
- Record the non-obvious release decisions and handoff in agentmemory or the configured local fallback, and maintain relevant wiki knowledge. Send a Claude handoff only when user-authorized. Leave one requested updated app with operator preferences restored; never execute the operator's prepared plan as a release smoke test.
- Report release link, included docs, verification result and any unchecked physical-device/provider/update behavior. Automatic updater testing is separate unless requested; do not silently claim it was tested.
