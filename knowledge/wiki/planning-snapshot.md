# Isolated planning snapshot

Trace: R-31 → BG-13 → SF-26 → UC-23 → QAS-24 → DDR-02 → DD-33.

## Contract

Company CLIs cannot prove read-only behavior, so the product isolates planning instead of
trusting a label. `Repository::export_tree` writes only regular tracked blobs of the
reviewed base (via `ls-tree` and one `cat-file --batch`) into `<plan-dir>/snapshot`,
outside the repository; symlinks, submodules, untracked and ignored files are never
materialized. The coordinator runs there; the snapshot is removed even on failure, and
HEAD, porcelain status and the ignored-file listing of the target must be unchanged.
Verified templates (Codex read-only sandbox, guarded Claude read-only tools) keep their
restrictions. Residual risk: absolute-path writes and network access are not sandboxed;
changes to the target are detected and refuse the proposal.

## Evidence

Red: an `unverified` coordinator was refused. Green: a hostile mock planner that asserts
the ignored `secret.env` is absent, deletes `base.txt` and writes `hostile.txt` leaves
the target unchanged and the snapshot removed.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/git.rs` | `e5bce851c46ac26d51f4dcd0c61c1b8d3cb7692a60a80ced72589d39a971ad35` |
| `src/planner.rs` | `f6b5f27a45f8716d91b20ade1d742d3420bec966c0de7ab1fc8b10bca1222331` |
| `src/coordinator.rs` | `d7bf0dc4482f3bef138efb1678c8f83719454d2d23b932ff1b89d816c768adfd` |
| `tests/planner.rs` | `5ca0ed983a33c54777b0f7b79a7d9bf5999030106f036ed98c43ce2cb0820def` |
