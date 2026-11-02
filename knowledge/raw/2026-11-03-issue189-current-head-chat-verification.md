---
source: local Rust tests and exact-head GitHub Actions
retrieved_at: 2026-11-03T12:00:00+09:00
revision: 397069af17d33a6296397ace706016907e065579
hash: git commit 397069af
---

# Source observation: issue #189 coordinator chat accessibility

The Rust workspace chat editor exposes its localized placeholder as the AccessKit name for its multiline input. The Windows UI Automation driver selects a visible, enabled `ControlType.Edit` whose name begins with the English placeholder. The test contract also covers an app window that has not appeared yet.

At current root PR #22 source `397069af`, the local command

```text
cargo test --locked --offline --test native_workspace_chat_accessibility --test windows_chat_script_contract --test chat_worker_lifecycle --test native_chat_terminal --test source_layout
```

passed 11 tests across five suites. This verifies source rendering, script contracts, transcript interactions, worker behavior, and source/test layout. It does not execute the Windows package.

Exact-head Platform Validation run `41500210315` was still running when this record was written. The Windows packaged chat journeys can report an explicit skip when the GitHub host Job Object refuses child-process breakaway. Such a skip is not a pass; keep #189 and the related packaged workflow acceptance open until the exact UI journey runs and passes on a host that permits the required worker lifecycle.
