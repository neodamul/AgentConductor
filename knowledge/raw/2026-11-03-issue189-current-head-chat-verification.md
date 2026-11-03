---
source: local Rust tests and GitHub Actions status inspection
retrieved_at: 2026-11-03
revision: d5731f11152b473b374119dbf541e11afbef9c11
hash: git commit d5731f11
---

# Source observation: issue #189 coordinator chat accessibility

The Rust workspace chat editor exposes its localized placeholder as the AccessKit name for its multiline input. The Windows UI Automation driver selects a visible, enabled `ControlType.Edit` whose name begins with the English placeholder. The test contract also covers an app window that has not appeared yet.

At current root PR #22 source `d5731f11`, the local command

```text
cargo test --locked --offline --features native-ui-probe --test native_workspace_chat_accessibility --test native_workspace_chat_reset --test native_workspace_chat --test native_chat_terminal --test windows_chat_script_contract --test windows_package_workflow --test source_layout
```

passed 23 tests across seven suites; warning-denied all-target Clippy also passes. This verifies source rendering, script contracts, transcript interactions, explicit host-skip diagnostics, and source/test layout. It does not execute the Windows package.

Platform Validation run `41500210315` is cancelled. Current-head run `41553629403` is compiling/testing macOS and Windows targets, but its job list contains no exact packaged chat UI journey. The Windows packaged chat journeys can report an explicit skip when the GitHub host Job Object refuses child-process breakaway. Such a skip is not a pass; keep #189 and the related packaged workflow acceptance open until the exact UI journey runs and passes on a host that permits the required worker lifecycle.

Graphify was refreshed after this verification: 13,202 nodes, 19,775 edges, 967 communities. CocoIndex CLI is not installed in this host.

Read-only UTM inspection found `Z:\issue9-windows-package-ac3f8b.zip`, SHA-256 `57d45e14390264bd471855220238afd0b6e86421401f89efd8387d97355f64bc`. Its embedded package manifest names source commit `ac3f8b327bd8c076f53c6198b1fd14e8a3f6f5bb` (2026-10-26), which predates the chat accessible-name fix `fbc2280a1f2f3c919f051d512f4f56a24fa11e74` (2026-10-30). Its smoke summary covers four CLI merge cases, not chat UI. The guest has no `cargo` or `rustc`, and the host has no Windows cross-linker/toolchain. The old archive was not launched and is not current-head package evidence.
