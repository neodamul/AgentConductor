# Internal distribution license review

Checked: 2026-10-09 KST. **Preliminary** top-level license and selected notice review,
not a complete dependency/binary/model audit or distribution clearance. Distinguish
using development tools, copying code, and bundling products.

| Component | Finding | Current treatment |
|---|---|---|
| conduct / Agetor / agent-deck | MIT | Preserve version-specific license/copyright when copying/bundling; review dependencies separately |
| Vibe Kanban | Apache-2.0 | Candidate reference/reuse after change/notice/NOTICE checks |
| ao | API license=null; no LICENSE in inspected tree | Exclude copying/redistribution pending permission |
| [agentmemory](https://github.com/rohitg00/agentmemory/blob/main/LICENSE) | Apache-2.0 | Optional external service; inspect engine too |
| [iii](https://github.com/iii-hq/iii/blob/main/LICENSE.spdx) | Apache-2.0 AND Elastic-2.0 | Not permissive-only; requires internal OSS/legal policy review |
| [RTK](https://github.com/rtk-ai/rtk/blob/develop/LICENSE) | Apache-2.0 | Development aid; not bundled by default |
| [Ponytail](https://github.com/DietrichGebert/ponytail/blob/main/LICENSE) | MIT | Preserve notices if redistributing skills; currently read and applied |
| [Graphify](https://github.com/Graphify-Labs/graphify/blob/v8/LICENSE) | Apache-2.0 with earlier MIT contributions | Review preservation of [NOTICE](https://github.com/Graphify-Labs/graphify/blob/v8/NOTICE) and LICENSE-MIT |
| [CocoIndex Code](https://github.com/cocoindex-io/cocoindex-code/blob/main/LICENSE) / [CocoIndex](https://github.com/cocoindex-io/cocoindex/blob/main/LICENSE) | Apache-2.0 | Development aids; Python dependencies and embedding models separate |
| Claude Code / Codex / OpenCode | CLI licenses and service terms are separate | Execute user-installed CLIs; do not bundle their binaries/auth files |

agentmemory upstream pins iii/v0.22.1. GitHub API resolved it to commit
e7de3820d1e558f3762edf95e4440552444d48d3; its LICENSE.spdx has the same AND declaration.
DataLicense CC0-1.0 covers SPDX metadata, not a permissive license for the engine.
Review actual file scope and Elastic service restrictions against the exact distribution.

Default MVP distribution is the Rust core and local state/wiki. Connect existing agentmemory
services optionally; never automatically download/bundle iii. Optional installation does not
waive license review. Use SQLite FTS5/Markdown memory if internal policy disallows it.

## Release completion gate

- Pin Cargo.lock and feature/platform resolved dependencies, sidecars, binaries, model revisions.
- Collect all direct/transitive SPDX identifiers, copyrights, license texts, and SBOM.
- Refer unknown/custom/GPL/AGPL/Elastic conditions to internal policy review.
- Apply MIT/Apache license, NOTICE, and change requirements from actual texts.
- Separately inspect legacy Graphify MIT code, model weights, installers, CLI service terms.
- Complete a release candidate only when the reviewed inventory matches the bundled inventory.

Cargo.lock now exists. A [host-target declaration inventory](rust-dependencies.md) now covers 33 packages.
The initial macOS release binary links only libiconv/libSystem; no dynamic SQLite library.
Full source/license-text and multi-platform distribution review remain **not performed**.
