# Native language switching and Agent Room status workflow

The exact v0.1.36 macOS package was used to switch the native Agent Room from Korean to English and back through Settings. The workflow changed shared navigation, phase controls, candidate actions, review evidence, and task-status labels without changing the run or candidate state.

The fixture remained `waiting_approval`, with three completed tasks and three passing tests. Approval, rejection, and merge were not activated. The single package window was retained on the Korean preference after verification.

Current Rust branch verification is 948 passed / 14 ignored across 310 suites, plus warning-denied Clippy, rustfmt, and 13 public-doc/source-layout tests. Graphify refreshed to 8,879 nodes / 15,210 edges. CocoIndex is unavailable in the current environment.

Evidence: [`issue9-6f1111c-language-status-workflow.json`](../../docs/evidence/issue9-6f1111c-language-status-workflow.json).

This is packaged workflow 11/25. It verifies the opt-in native interface only; it does not establish that the native UI has replaced the shipped default UI. Windows UI and Korean IME remain open verification gates.
