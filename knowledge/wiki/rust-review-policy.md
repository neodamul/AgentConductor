# Rust review policy — GitHub #169

## Decision

Use a short review checklist derived from the Google Rust Style Guide for new and changed Rust code. Apply rules to touched code without rewriting the repository or adding a lint gate that fails on the existing baseline.

## Rules that affect implementation

- Keep recoverable process, filesystem, and provider failures in `Result`; isolate and document `unsafe` behind small platform wrappers.
- Prefer borrowing and meaningful named types over clones, broad glob imports, and ambiguous positional booleans.
- Use standard-library traits for standard behavior, and document non-obvious public contracts and invariants.
- Keep rustfmt defaults, warning-denied Clippy, the source-layout guard, and the Rust branch's 300-line changed-file limit authoritative.

## Exclusions and upgrade path

Chrome/Fuchsia-specific advice, unstable options, library-only guidance, and retroactive cleanup are excluded. Add an executable lint only after measuring the gap, testing it red-first, and opening a dedicated issue. `undocumented_unsafe_blocks` is a possible later gate after existing unsafe blocks receive reviewed `SAFETY:` comments.

## Traceability

Requirements through design decision: R-174 → BG-129 → SF-162 → UC-165 → FR-160/NFR-70/C-06 → QAS-171 → DDR-34 → DD-185. The policy is in `docs/13-rust-review-policy.md` and was ported from `main` for Rust migration PR #22; no source behavior changed.
