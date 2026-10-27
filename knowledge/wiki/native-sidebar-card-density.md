# Native sidebar card density

## Finding

At the 248 px sidebar width, a long executable and permission label wrapped to three lines and inflated the native agent card. The detail belongs to profile inspection, but keeping it available in the compact roster is useful for operators.

## Decision and verification

Use egui's one-line truncation for the visible label. Keep the complete value as the AccessKit name; egui displays the elided label's full text on hover. The regression first measured a 48 px label and failed the 16 px row bound. After the change, the card label is one line while the complete command and permission remain accessible. The sidebar suite passes 5/5, density 1/1, bilingual footer 1/1, source-layout 1/1, and focused Clippy passes with warnings denied.

The preceding exact Rust build was inspected in one 1140×772 window using three isolated `/usr/bin/true` profiles. No authenticated CLI, saved user workspace, or external service was used. This improves one detail row; the cards still differ from the shipped v0.1.24 design, and the 25-workflow end-to-end parity count remains 0/25.

### Compact roster follow-up (2026-10-19)

The command and permission now remain in the agent-name accessible label and hover text; they no longer consume a separate row. The coordinator badge shares the action row at the 248 px rail width. Rendered-bound tests cover the one-line name and same-row coordinator placement; the legacy sidebar assertions were updated for the richer accessible label. Local verification: sidebar 5/5, density 2/2, bilingual footer 1/1, and source-layout 1/1. Changed Rust files are formatted and each is below 300 lines.

The exact rebuilt app was visually checked in one 1140×772 window: the coordinator shares the action row, cards are shorter, and the 248 px rail has no overflow. Three isolated `/usr/bin/true` profiles were used; no authenticated CLI, saved user workspace, or external service was touched. Native end-to-end parity stays 0/25 because component tests and this isolated screen do not establish replacement acceptance. Windows packaging, full keyboard/screen-reader review, Korean IME, and all 25 workflows remain open.

Graphify local AST extraction after this increment: 6,700 nodes / 11,612 edges. CocoIndex Code structural grep found the drag-handle label in the changed controls module without initializing a semantic index. AgentMemory: `mem_mveiza4l_07437964dc8a` (previous detail-row increment); `mem_mvej9gh6_c3d1912a7762` (compact-roster follow-up).
