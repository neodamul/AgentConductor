# Native sidebar local-first footer

## Finding

The Rust CLI-agent sidebar omitted two existing localized messages: the local-first heading and the summary that explains independent agent sessions and human control of execution and final merge. The English and Korean catalog entries already existed.

## Change and verification

A rendered-text regression uses the production 248 px egui left panel and checks both languages. It failed before the footer was rendered and passes after it was added below the independently scrolling agent list. The list reserves space for the footer so it remains visible. The native sidebar suite passes 5/5, the footer regression passes 1/1, and the source-layout test passes. Formatting and whitespace checks pass.

Graphify local AST extraction updated the code graph to 6,680 nodes and 11,590 edges. CocoIndex Code's no-index structural grep found the workspace sidebar entry point; project index initialization remains unavailable. AgentMemory record: `mem_mvedynhg_5787cdf797cd`.

This is component-level evidence only. The exact v0.1.24 visual comparison, packaged macOS/Windows validation, constrained-window checks, and full keyboard/screen-reader traversal remain open. The accepted workflow count remains 0/25.
