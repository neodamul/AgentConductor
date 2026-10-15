# Bounded source and test modules

DD-41 traces R-35 through BG-05, SF-12, C-04, NFR-07, QAS-12 and DDR-04/DDR-06.
A compiled layout regression first failed on eight files over 500 lines and three
inline test bodies. Responsibility-based modules now preserve the existing public
paths and single-executable distribution. Production dependencies are unchanged.

Workflow separates execution, scheduler, permissions, approval, review, editing,
assignment and recovery. Process I/O separates control and platform containment.
Native shells separate Cocoa and WebView2; UI controller and startup dispatch have
separate modules. Private unit-test bodies use cfg/path modules under tests/unit/
without widening production APIs. Cooperative memory tests use a nested test module.
Under DD-160, the agent-tools-guide Python unittest contracts were ported to
`tests/agent_tools_guide.rs`; remaining Python contract modules are still scheduled for
incremental Rust conversion, so this is not evidence that the verification migration is done.
The MIT license and manifest contract is also in `tests/project_license.rs`; it checks
the installed-facing LICENSE, Cargo metadata and README link without invoking an interpreter.

The CI layout test checks source, test, script, example, style and markup files and
an optional build script. It rejects production test bodies. Maximum file: 485 lines.
Full macOS regression before path migration: 213 passed, 0 failed, 9 ignored across
57 suites. New canonical checkout's focused layout and format checks passed.
CI 38039813985 on 47eb2d7 completed successfully on native macOS and Windows, including build, layout, applicable tests, formatting and Clippy. This proves the module-split checkpoint, not full-product acceptance. Earlier CI 38038926444 passed Windows
tests but found two fixture pointer-argument lints, corrected in this checkpoint.

Evidence: `.agentconductor/verification/source-layout/regression.json` and
`source-manifest.json` (SHA-256 `7faf487cc8ab1f6b26b3666890af12854990f570c57d4850054ffcfb6128f232`).
[Design](../../docs/06-design-decisions.md), [layout test](../../tests/source_layout.rs),
[continuity](../../docs/12-development-continuity.md).
Graphify: 1,701 nodes / 3,420 edges, structural extraction without LLM. CocoIndex
structural API located continuation definition, recovery call and re-export with
no failed languages. Its MCP entrypoint lacks FastMCP in this host environment;
no dependency or daemon was installed. Ponytail has no callable capability in this
session; no plugin execution is claimed. Agentmemory: mem_mv25wmpa_f63ae0e2a551.
