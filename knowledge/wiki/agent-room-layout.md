# Agent Room review layout

The review surface must retain room for task evidence while keeping the summary and inspector usable. At the 1140 px reference viewport, the summary occupies 18% (180–240 px), the inspector occupies 24% (280–320 px), and the center review column must retain at least 550 px. Narrow windows use the existing fallback sizing.

Activity cards take their minimum width from the center review surface. An egui group otherwise sizes itself to intrinsic content, leaving unused space and making task evidence appear cramped. The rendered frame rectangle is tested directly: at least 540 px in a 620 px test surface.

## Verification checkpoint

Source commit `1f152729e0c3babd0beb2b6338536b667ea1671c` was packaged for macOS arm64 and inspected in one v0.1.36 window. The Activity cards span the review column; the inspector stays bounded. Candidate `3db8840b02ce28558701698c327a5820b7dc7311` remained `waiting_approval`, with 3/3 tasks and tests; no approval, rejection, or merge occurred. This is a component layout check and does not increase the packaged workflow acceptance count (11/25).

The locked/offline serial all-target test suite passed 929 tests (14 ignored); warning-denied Clippy, the optimized release build, and formatting passed. Graphify reported 8,886 nodes, 14,036 edges, and 689 communities. CocoIndex was unavailable in the environment. Package checksums and detailed evidence are in [`issue9-1f152729-agent-room-layout.json`](../../docs/evidence/issue9-1f152729-agent-room-layout.json).
