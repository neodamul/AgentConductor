# Native sidebar card density

## Finding

At the 248 px sidebar width, a long executable and permission label wrapped to three lines and inflated the native agent card. The detail belongs to profile inspection, but keeping it available in the compact roster is useful for operators.

## Decision and verification

Use egui's one-line truncation for the visible label. Keep the complete value as the AccessKit name; egui displays the elided label's full text on hover. The regression first measured a 48 px label and failed the 16 px row bound. After the change, the card label is one line while the complete command and permission remain accessible. The sidebar suite passes 5/5, density 1/1, bilingual footer 1/1, source-layout 1/1, and focused Clippy passes with warnings denied.

The exact current Rust build was inspected in one 1140×772 window using three isolated `/usr/bin/true` profiles. No authenticated CLI, saved user workspace, or external service was used. This improves one detail row; the cards still differ from the shipped v0.1.24 design, and the 25-workflow end-to-end parity count remains 0/25.

Graphify local AST extraction: 6,692 nodes / 11,604 edges. CocoIndex Code structural grep found the `Label::new` detail row without initializing a semantic index. AgentMemory: `mem_mvee8psy_d5316bb3aac0`.
